use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

const MAX_METADATA_ENTRIES: u64 = 100_000;
const MAX_TENSORS: u64 = 10_000_000;
const MAX_STRING_BYTES: u64 = 1024 * 1024;
const MAX_ARRAY_ITEMS: u64 = 10_000_000;
const DEFAULT_CONTEXT: u64 = 2_048;
const DEFAULT_BATCH: u64 = 128;
const MIB: u64 = 1024 * 1024;

#[derive(Debug, Clone, Default)]
pub struct GgufMetadata {
    pub architecture: Option<String>,
    pub model_name: Option<String>,
    pub quantization: Option<String>,
    pub layer_count: Option<u64>,
    pub attention_heads: Option<u64>,
    pub kv_heads: Option<u64>,
    pub embedding_length: Option<u64>,
    pub key_length: Option<u64>,
    pub value_length: Option<u64>,
    pub training_context_length: Option<u64>,
}

#[derive(Debug)]
enum MetaValue {
    String(String),
    Unsigned(u64),
    Signed(i64),
}

impl GgufMetadata {
    /// Reads only the GGUF header and metadata table. Tensor payloads are never
    /// allocated or copied by this parser.
    pub fn read(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("Cannot read GGUF metadata: {e}"))?;
        let file_len = file.metadata().map_err(|e| format!("Cannot inspect GGUF file: {e}"))?.len();
        if file_len < 24 {
            return Err("The selected file is too small to contain a GGUF header.".into());
        }

        let mut reader = BufReader::new(file);
        let mut magic = [0_u8; 4];
        reader.read_exact(&mut magic).map_err(|e| format!("Cannot read GGUF magic: {e}"))?;
        if &magic != b"GGUF" {
            return Err("The selected file does not contain a valid GGUF magic header.".into());
        }

        let version = read_u32(&mut reader)?;
        if !(2..=3).contains(&version) {
            return Err(format!("GGUF version {version} is not supported. Supported versions are 2 and 3."));
        }

        let tensor_count = read_u64(&mut reader)?;
        let metadata_count = read_u64(&mut reader)?;
        if tensor_count == 0 || tensor_count > MAX_TENSORS {
            return Err("GGUF tensor count is invalid or exceeds the safety limit.".into());
        }
        if metadata_count > MAX_METADATA_ENTRIES {
            return Err("GGUF metadata entry count exceeds the safety limit.".into());
        }

        let mut captured = std::collections::HashMap::<String, MetaValue>::new();
        for _ in 0..metadata_count {
            let key = read_string(&mut reader, file_len)?;
            let value_type = read_u32(&mut reader)?;
            let capture = key == "general.architecture"
                || key == "general.name"
                || key == "general.file_type"
                || key.ends_with(".block_count")
                || key.ends_with(".attention.head_count")
                || key.ends_with(".attention.head_count_kv")
                || key.ends_with(".embedding_length")
                || key.ends_with(".attention.key_length")
                || key.ends_with(".attention.value_length")
                || key.ends_with(".context_length");

            if let Some(value) = read_value(&mut reader, file_len, value_type, capture)? {
                if capture {
                    captured.insert(key, value);
                }
            }
        }

        let architecture = take_string(&captured, "general.architecture");
        if architecture.as_deref().is_none_or(str::is_empty) {
            return Err("GGUF metadata is missing general.architecture.".into());
        }
        let prefix = architecture.as_deref().unwrap_or_default();

        let suffix_value = |suffix: &str| -> Option<u64> {
            captured.iter()
                .find(|(key, _)| key.strip_prefix(prefix).is_some_and(|tail| tail == suffix))
                .and_then(|(_, value)| as_u64(value))
        };

        let file_type = captured.get("general.file_type").and_then(as_u64);
        Ok(Self {
            architecture,
            model_name: take_string(&captured, "general.name"),
            quantization: file_type.and_then(quantization_name),
            layer_count: suffix_value(".block_count"),
            attention_heads: suffix_value(".attention.head_count"),
            kv_heads: suffix_value(".attention.head_count_kv"),
            embedding_length: suffix_value(".embedding_length"),
            key_length: suffix_value(".attention.key_length"),
            value_length: suffix_value(".attention.value_length"),
            training_context_length: suffix_value(".context_length"),
        })
    }

    /// Conservative estimate for the configured CPU session. Quantized KV
    /// variants and unusual architectures are handled conservatively.
    pub fn estimate_peak_bytes(&self, model_size_bytes: u64) -> u64 {
        let weight_working_set = model_size_bytes.saturating_add(model_size_bytes / 5);
        let kv_cache = match (
            self.layer_count,
            self.kv_heads.or(self.attention_heads),
            self.embedding_length,
        ) {
            (Some(layers), Some(kv_heads), Some(embedding)) if kv_heads > 0 => {
                let heads = self.attention_heads.unwrap_or(kv_heads).max(1);
                let head_dim = self.key_length.unwrap_or(embedding / heads).max(1);
                let value_dim = self.value_length.unwrap_or(head_dim).max(1);
                DEFAULT_CONTEXT
                    .saturating_mul(layers)
                    .saturating_mul(kv_heads)
                    .saturating_mul(head_dim.saturating_add(value_dim))
                    .saturating_mul(2)
            }
            _ => model_size_bytes / 5 + 512 * MIB,
        };
        let batch_scratch = self.embedding_length
            .unwrap_or(4096)
            .saturating_mul(DEFAULT_BATCH)
            .saturating_mul(8);
        weight_working_set
            .saturating_add(kv_cache)
            .saturating_add(batch_scratch)
            .saturating_add(384 * MIB)
    }
}

fn read_value<R: Read + Seek>(
    reader: &mut R,
    file_len: u64,
    kind: u32,
    capture: bool,
) -> Result<Option<MetaValue>, String> {
    let value = match kind {
        0 | 1 => {
            if capture { Some(MetaValue::Unsigned(read_u8(reader)? as u64)) }
            else { skip(reader, file_len, 1)?; None }
        }
        2 | 3 => {
            if capture { Some(MetaValue::Unsigned(read_u16(reader)? as u64)) }
            else { skip(reader, file_len, 2)?; None }
        }
        4 => {
            if capture { Some(MetaValue::Unsigned(read_u32(reader)? as u64)) }
            else { skip(reader, file_len, 4)?; None }
        }
        5 => {
            if capture { Some(MetaValue::Signed(read_i32(reader)? as i64)) }
            else { skip(reader, file_len, 4)?; None }
        }
        6 => { skip(reader, file_len, 4)?; None }
        7 => {
            if capture { Some(MetaValue::Unsigned(read_u8(reader)? as u64)) }
            else { skip(reader, file_len, 1)?; None }
        }
        8 => {
            if capture { Some(MetaValue::String(read_string(reader, file_len)?)) }
            else {
                let len = read_u64(reader)?;
                if len > MAX_STRING_BYTES { return Err("GGUF string exceeds the safety limit.".into()); }
                skip(reader, file_len, len)?;
                None
            }
        }
        9 => { skip_array(reader, file_len)?; None }
        10 => {
            if capture { Some(MetaValue::Unsigned(read_u64(reader)?)) }
            else { skip(reader, file_len, 8)?; None }
        }
        11 => {
            if capture { Some(MetaValue::Signed(read_i64(reader)?)) }
            else { skip(reader, file_len, 8)?; None }
        }
        12 => { skip(reader, file_len, 8)?; None }
        _ => return Err(format!("GGUF metadata contains unknown value type {kind}.")),
    };
    Ok(value)
}

fn skip_array<R: Read + Seek>(reader: &mut R, file_len: u64) -> Result<(), String> {
    let element_type = read_u32(reader)?;
    let count = read_u64(reader)?;
    if count > MAX_ARRAY_ITEMS || element_type == 9 || element_type > 12 {
        return Err("GGUF metadata array exceeds the safety limit or uses an unsupported nested type.".into());
    }
    if element_type == 8 {
        for _ in 0..count {
            let len = read_u64(reader)?;
            if len > MAX_STRING_BYTES { return Err("GGUF metadata string exceeds the safety limit.".into()); }
            skip(reader, file_len, len)?;
        }
        return Ok(());
    }
    let width = match element_type {
        0 | 1 | 7 => 1_u64,
        2 | 3 => 2,
        4 | 5 | 6 => 4,
        10 | 11 | 12 => 8,
        _ => return Err("GGUF metadata array element type is invalid.".into()),
    };
    skip(reader, file_len, count.saturating_mul(width))
}

fn read_string<R: Read + Seek>(reader: &mut R, file_len: u64) -> Result<String, String> {
    let len = read_u64(reader)?;
    if len > MAX_STRING_BYTES { return Err("GGUF metadata string exceeds the safety limit.".into()); }
    ensure_remaining(reader, file_len, len)?;
    let mut bytes = vec![0_u8; len as usize];
    reader.read_exact(&mut bytes).map_err(|e| format!("Cannot read GGUF metadata string: {e}"))?;
    String::from_utf8(bytes).map_err(|_| "GGUF metadata contains a non-UTF-8 string.".into())
}

fn skip<R: Read + Seek>(reader: &mut R, file_len: u64, bytes: u64) -> Result<(), String> {
    let position = reader.stream_position()
        .map_err(|e| format!("Cannot inspect GGUF stream position: {e}"))?;
    let target = position.checked_add(bytes)
        .filter(|end| *end <= file_len)
        .ok_or_else(|| "GGUF metadata is truncated or contains an out-of-range length.".to_string())?;
    reader.seek(SeekFrom::Start(target))
        .map_err(|e| format!("Cannot skip GGUF metadata value: {e}"))?;
    Ok(())
}

fn ensure_remaining<R: Seek>(reader: &mut R, file_len: u64, bytes: u64) -> Result<(), String> {
    let position = reader.stream_position()
        .map_err(|e| format!("Cannot inspect GGUF stream position: {e}"))?;
    if position.checked_add(bytes).is_none_or(|end| end > file_len) {
        return Err("GGUF metadata is truncated or contains an out-of-range length.".into());
    }
    Ok(())
}

fn read_u8<R: Read>(r: &mut R) -> Result<u8, String> {
    let mut b = [0_u8; 1]; r.read_exact(&mut b).map_err(|e| format!("Truncated GGUF value: {e}"))?; Ok(b[0])
}
fn read_u16<R: Read>(r: &mut R) -> Result<u16, String> {
    let mut b = [0_u8; 2]; r.read_exact(&mut b).map_err(|e| format!("Truncated GGUF value: {e}"))?; Ok(u16::from_le_bytes(b))
}
fn read_u32<R: Read>(r: &mut R) -> Result<u32, String> {
    let mut b = [0_u8; 4]; r.read_exact(&mut b).map_err(|e| format!("Truncated GGUF value: {e}"))?; Ok(u32::from_le_bytes(b))
}
fn read_i32<R: Read>(r: &mut R) -> Result<i32, String> {
    let mut b = [0_u8; 4]; r.read_exact(&mut b).map_err(|e| format!("Truncated GGUF value: {e}"))?; Ok(i32::from_le_bytes(b))
}
fn read_u64<R: Read>(r: &mut R) -> Result<u64, String> {
    let mut b = [0_u8; 8]; r.read_exact(&mut b).map_err(|e| format!("Truncated GGUF value: {e}"))?; Ok(u64::from_le_bytes(b))
}
fn read_i64<R: Read>(r: &mut R) -> Result<i64, String> {
    let mut b = [0_u8; 8]; r.read_exact(&mut b).map_err(|e| format!("Truncated GGUF value: {e}"))?; Ok(i64::from_le_bytes(b))
}

fn as_u64(value: &MetaValue) -> Option<u64> {
    match value {
        MetaValue::Unsigned(value) => Some(*value),
        MetaValue::Signed(value) if *value >= 0 => Some(*value as u64),
        _ => None,
    }
}
fn take_string(values: &std::collections::HashMap<String, MetaValue>, key: &str) -> Option<String> {
    match values.get(key) { Some(MetaValue::String(value)) => Some(value.clone()), _ => None }
}

fn quantization_name(file_type: u64) -> Option<String> {
    let name = match file_type {
        0 => "F32", 1 => "F16", 2 => "Q4_0", 3 => "Q4_1",
        7 => "Q8_0", 8 => "Q5_0", 9 => "Q5_1", 10 => "Q2_K",
        11 => "Q3_K_S", 12 => "Q3_K_M", 13 => "Q3_K_L",
        14 => "Q4_K_S", 15 => "Q4_K_M", 16 => "Q5_K_S",
        17 => "Q5_K_M", 18 => "Q6_K", 19 => "IQ2_XXS",
        20 => "IQ2_XS", 21 => "Q2_K_S", 22 => "IQ3_XS",
        23 => "IQ3_XXS", 24 => "IQ1_S", 25 => "IQ4_NL",
        26 => "IQ3_S", 27 => "IQ3_M", 28 => "IQ2_S",
        29 => "IQ2_M", 30 => "IQ4_XS", 31 => "IQ1_M",
        32 => "BF16", 36 => "TQ1_0", 37 => "TQ2_0",
        38 => "MXFP4_MOE", 39 => "NVFP4", 40 => "Q1_0",
        41 => "Q2_0", 42 => "F8_E4M3",
        _ => return None,
    };
    Some(name.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write, time::{SystemTime, UNIX_EPOCH}};

    fn push_string(out: &mut Vec<u8>, value: &str) {
        out.extend_from_slice(&(value.len() as u64).to_le_bytes());
        out.extend_from_slice(value.as_bytes());
    }
    fn push_u32(out: &mut Vec<u8>, key: &str, value: u32) {
        push_string(out, key);
        out.extend_from_slice(&4_u32.to_le_bytes());
        out.extend_from_slice(&value.to_le_bytes());
    }
    fn sample_file() -> std::path::PathBuf {
        let mut bytes = b"GGUF".to_vec();
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&1_u64.to_le_bytes());
        bytes.extend_from_slice(&10_u64.to_le_bytes());
        push_string(&mut bytes, "general.architecture");
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        push_string(&mut bytes, "llama");
        push_string(&mut bytes, "general.name");
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        push_string(&mut bytes, "Test model");
        push_u32(&mut bytes, "general.file_type", 15);
        push_u32(&mut bytes, "llama.block_count", 32);
        push_u32(&mut bytes, "llama.attention.head_count", 32);
        push_u32(&mut bytes, "llama.attention.head_count_kv", 8);
        push_u32(&mut bytes, "llama.embedding_length", 4096);
        push_u32(&mut bytes, "llama.attention.key_length", 128);
        push_u32(&mut bytes, "llama.attention.value_length", 128);
        push_u32(&mut bytes, "llama.context_length", 4096);
        let name = format!("sync-gguf-test-{}.gguf", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
        let path = std::env::temp_dir().join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(&bytes).unwrap();
        path
    }

    #[test]
    fn parses_architecture_quantization_and_memory_dimensions() {
        let path = sample_file();
        let metadata = GgufMetadata::read(&path).unwrap();
        assert_eq!(metadata.architecture.as_deref(), Some("llama"));
        assert_eq!(metadata.model_name.as_deref(), Some("Test model"));
        assert_eq!(metadata.quantization.as_deref(), Some("Q4_K_M"));
        assert_eq!(metadata.layer_count, Some(32));
        assert_eq!(metadata.kv_heads, Some(8));
        assert!(metadata.estimate_peak_bytes(2 * 1024 * 1024 * 1024) > 2 * 1024 * 1024 * 1024);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_non_gguf_files() {
        let path = std::env::temp_dir().join("sync-not-a-gguf.gguf");
        fs::write(&path, b"not a model").unwrap();
        assert!(GgufMetadata::read(&path).is_err());
        let _ = fs::remove_file(path);
    }
}
