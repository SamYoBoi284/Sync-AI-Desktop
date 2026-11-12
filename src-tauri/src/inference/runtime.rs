use std::sync::Mutex;

use chrono::{SecondsFormat, Utc};
use llama_cpp::{LlamaModel, LlamaParams, LlamaSession, SessionParams};
use serde::Serialize;

use super::gguf::GgufMetadata;

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * 1024 * 1024;
pub const DEFAULT_CONTEXT_LENGTH: u32 = 2_048;
pub const DEFAULT_BATCH_SIZE: u32 = 128;
pub const DEFAULT_MICRO_BATCH_SIZE: u32 = 64;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub is_loading: bool,
    pub is_loaded: bool,
    pub loading_model_id: Option<String>,
    pub loaded_model_id: Option<String>,
    pub loaded_model_name: Option<String>,
    pub loaded_at: Option<String>,
    pub estimated_memory_bytes: Option<u64>,
    pub context_length: Option<u32>,
    pub batch_size: Option<u32>,
    pub backend: String,
}

impl Default for RuntimeStatus {
    fn default() -> Self {
        Self {
            is_loading: false,
            is_loaded: false,
            loading_model_id: None,
            loaded_model_id: None,
            loaded_model_name: None,
            loaded_at: None,
            estimated_memory_bytes: None,
            context_length: None,
            batch_size: None,
            backend: "llama.cpp · CPU".into(),
        }
    }
}

pub struct LoadedModel {
    // Field order is intentional: drop the native session before the model.
    _session: LlamaSession,
    _model: LlamaModel,
    pub(crate) model_id: String,
    pub(crate) model_name: String,
    pub(crate) loaded_at: String,
    pub(crate) estimated_memory_bytes: u64,
    pub(crate) context_length: u32,
    pub(crate) batch_size: u32,
}

#[derive(Default)]
struct RuntimeSlot {
    loading_model_id: Option<String>,
    loaded: Option<LoadedModel>,
}

pub struct RuntimeManager {
    slot: Mutex<RuntimeSlot>,
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self { slot: Mutex::new(RuntimeSlot::default()) }
    }

    pub fn status(&self) -> Result<RuntimeStatus, String> {
        let slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
        let mut status = RuntimeStatus::default();
        if let Some(id) = &slot.loading_model_id {
            status.is_loading = true;
            status.loading_model_id = Some(id.clone());
        }
        if let Some(loaded) = &slot.loaded {
            status.is_loaded = true;
            status.loaded_model_id = Some(loaded.model_id.clone());
            status.loaded_model_name = Some(loaded.model_name.clone());
            status.loaded_at = Some(loaded.loaded_at.clone());
            status.estimated_memory_bytes = Some(loaded.estimated_memory_bytes);
            status.context_length = Some(loaded.context_length);
            status.batch_size = Some(loaded.batch_size);
        }
        Ok(status)
    }

    pub fn begin_loading(&self, model_id: &str) -> Result<(), String> {
        let mut slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
        if slot.loading_model_id.is_some() {
            return Err("A model is already being loaded.".into());
        }
        if let Some(loaded) = &slot.loaded {
            if loaded.model_id == model_id {
                return Err("This model is already loaded.".into());
            }
            return Err("Unload the active model before loading a different one.".into());
        }
        slot.loading_model_id = Some(model_id.to_string());
        Ok(())
    }

    pub fn finish_loading(&self, model: Option<LoadedModel>) -> Result<(), String> {
        let mut slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
        slot.loading_model_id = None;
        if let Some(model) = model {
            if slot.loaded.is_some() {
                return Err("A model became active while another load was completing.".into());
            }
            slot.loaded = Some(model);
        }
        Ok(())
    }

    pub fn is_in_use(&self, model_id: &str) -> Result<bool, String> {
        let slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
        Ok(slot.loading_model_id.as_deref() == Some(model_id)
            || slot.loaded.as_ref().is_some_and(|model| model.model_id == model_id))
    }

    pub fn unload(&self) -> Result<(), String> {
        let removed = {
            let mut slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
            if slot.loading_model_id.is_some() {
                return Err("Wait for the current model load to finish before unloading.".into());
            }
            slot.loaded.take()
        };
        drop(removed);
        Ok(())
    }
}

pub struct RamLimiterConfig {
    pub enabled: bool,
    pub limit_gb: f64,
}

impl Default for RamLimiterConfig {
    fn default() -> Self {
        Self { enabled: false, limit_gb: 1.0 }
    }
}

impl RamLimiterConfig {
    pub fn from_json(raw: Option<String>) -> Result<Self, String> {
        let Some(raw) = raw else { return Ok(Self::default()); };
        let value: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|_| "Saved RAM limiter settings are invalid JSON. Open Settings and save the limiter configuration again.".to_string())?;
        let enabled = value.get("enabled").and_then(serde_json::Value::as_bool)
            .ok_or_else(|| "Saved RAM limiter setting is missing a valid enabled value.".to_string())?;
        let limit_gb = value.get("limitGb").and_then(serde_json::Value::as_f64)
            .ok_or_else(|| "Saved RAM limiter setting is missing a valid memory budget.".to_string())?;
        if !limit_gb.is_finite() || !(1.0..=1024.0).contains(&limit_gb) {
            return Err("Saved RAM limiter budget is outside the supported range of 1 to 1024 GB.".into());
        }
        Ok(Self { enabled, limit_gb })
    }

    pub fn budget_bytes(&self) -> Option<u64> {
        self.enabled.then(|| (self.limit_gb * GIB as f64).min(u64::MAX as f64) as u64)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelResourceEstimate {
    pub model_size_bytes: u64,
    pub estimated_peak_bytes: u64,
    pub available_ram_bytes: u64,
    pub configured_budget_bytes: Option<u64>,
    pub context_length: u32,
    pub batch_size: u32,
    pub fits_configured_budget: bool,
    pub fits_current_availability: bool,
    pub estimate_kind: String,
}

pub fn estimate_resources(
    model_size_bytes: u64,
    metadata: &GgufMetadata,
    available_ram_bytes: u64,
    config: &RamLimiterConfig,
) -> ModelResourceEstimate {
    let estimated_peak_bytes = metadata.estimate_peak_bytes(model_size_bytes);
    let configured_budget_bytes = config.budget_bytes();
    let safety_reserve = (available_ram_bytes / 8).max(GIB);
    ModelResourceEstimate {
        model_size_bytes,
        estimated_peak_bytes,
        available_ram_bytes,
        configured_budget_bytes,
        context_length: DEFAULT_CONTEXT_LENGTH,
        batch_size: DEFAULT_BATCH_SIZE,
        fits_configured_budget: configured_budget_bytes.is_none_or(|budget| estimated_peak_bytes <= budget),
        fits_current_availability: estimated_peak_bytes.saturating_add(safety_reserve) <= available_ram_bytes,
        estimate_kind: "Conservative GGUF metadata estimate; llama.cpp native session sizing is checked again during load.".into(),
    }
}

pub fn validate_preflight(estimate: &ModelResourceEstimate) -> Result<(), String> {
    if !estimate.fits_configured_budget {
        let budget = estimate.configured_budget_bytes.unwrap_or_default() / GIB;
        let needed = estimate.estimated_peak_bytes / GIB;
        return Err(format!(
            "Model load blocked by RAM Limiter. Estimated inference memory is about {needed} GB, above your configured {budget} GB budget. Increase the budget in Settings or choose a smaller model."
        ));
    }
    if !estimate.fits_current_availability {
        let reserve = (estimate.available_ram_bytes / 8).max(GIB);
        let needed = estimate.estimated_peak_bytes.saturating_add(reserve);
        return Err(format!(
            "Model load blocked because current available RAM is too low. The estimate plus a system safety reserve needs about {} GB; Windows currently reports {} GB available.",
            needed / GIB,
            estimate.available_ram_bytes / GIB
        ));
    }
    Ok(())
}

pub fn load_native_model(
    path: std::path::PathBuf,
    model_id: String,
    model_name: String,
    model_size_bytes: u64,
    config: RamLimiterConfig,
    available_ram_bytes_before_load: u64,
) -> Result<LoadedModel, String> {
    let metadata = GgufMetadata::read(&path)?;
    let initial = estimate_resources(model_size_bytes, &metadata, available_ram_bytes_before_load, &config);
    validate_preflight(&initial)?;

    let mut model_params = LlamaParams::default();
    model_params.n_gpu_layers = 0;
    model_params.use_mmap = true;
    model_params.use_mlock = false;

    let model = LlamaModel::load_from_file(&path, model_params)
        .map_err(|error| format!("llama.cpp could not load this GGUF model: {error:?}"))?;

    let mut session_params = SessionParams::default();
    session_params.n_ctx = DEFAULT_CONTEXT_LENGTH;
    session_params.n_batch = DEFAULT_BATCH_SIZE;
    session_params.n_ubatch = DEFAULT_MICRO_BATCH_SIZE;

    // This native estimate is computed with the actual loaded model and the
    // exact session parameters that will be allocated.
    let session_usage = model.estimate_session_size(&session_params);
    let session_bytes = (session_usage.host_memory as u64)
        .saturating_add(session_usage.device_memory as u64);
    let native_estimate = model_size_bytes
        .saturating_add(model_size_bytes / 10)
        .saturating_add(session_bytes)
        .saturating_add(256 * MIB);

    if let Some(budget) = config.budget_bytes() {
        if native_estimate > budget {
            return Err(format!(
                "Model load blocked after native sizing. llama.cpp estimates about {} GB for model weights and the 2K context session, above your configured {} GB budget. The model has been released from memory.",
                native_estimate / GIB,
                budget / GIB
            ));
        }
    }

    let mut system = sysinfo::System::new();
    system.refresh_memory();
    let available_after_weights = system.available_memory();
    let safety_reserve = (system.total_memory() / 8).max(GIB);
    if session_bytes.saturating_add(safety_reserve) > available_after_weights {
        return Err(
            "Model weights loaded, but the native context session was not allocated because available RAM fell below the system safety reserve. The model has been released.".into()
        );
    }

    let session = model.create_session(session_params)
        .map_err(|error| format!("llama.cpp could not allocate the configured context session: {error:?}"))?;

    Ok(LoadedModel {
        _session: session,
        _model: model,
        model_id,
        model_name,
        loaded_at: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        estimated_memory_bytes: native_estimate.max(initial.estimated_peak_bytes),
        context_length: DEFAULT_CONTEXT_LENGTH,
        batch_size: DEFAULT_BATCH_SIZE,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_limiter_setting_defaults_to_disabled() {
        let config = RamLimiterConfig::from_json(None).unwrap();
        assert!(!config.enabled);
        assert!(config.budget_bytes().is_none());
    }

    #[test]
    fn malformed_limiter_setting_fails_closed() {
        assert!(RamLimiterConfig::from_json(Some(r#"{"enabled":true}"#.into())).is_err());
    }

    #[test]
    fn budget_bytes_are_applied_only_when_enabled() {
        let config = RamLimiterConfig::from_json(Some(r#"{"enabled":true,"limitGb":3}"#.into())).unwrap();
        assert_eq!(config.budget_bytes(), Some(3 * GIB));
    }

    #[test]
    fn preflight_refuses_models_over_the_configured_budget() {
        let estimate = ModelResourceEstimate {
            model_size_bytes: 3 * GIB,
            estimated_peak_bytes: 5 * GIB,
            available_ram_bytes: 8 * GIB,
            configured_budget_bytes: Some(4 * GIB),
            context_length: DEFAULT_CONTEXT_LENGTH,
            batch_size: DEFAULT_BATCH_SIZE,
            fits_configured_budget: false,
            fits_current_availability: true,
            estimate_kind: "test".into(),
        };
        assert!(validate_preflight(&estimate).unwrap_err().contains("RAM Limiter"));
    }

    #[test]
    fn preflight_refuses_models_that_leave_no_system_reserve() {
        let estimate = ModelResourceEstimate {
            model_size_bytes: 2 * GIB,
            estimated_peak_bytes: 3 * GIB,
            available_ram_bytes: 3 * GIB,
            configured_budget_bytes: None,
            context_length: DEFAULT_CONTEXT_LENGTH,
            batch_size: DEFAULT_BATCH_SIZE,
            fits_configured_budget: true,
            fits_current_availability: false,
            estimate_kind: "test".into(),
        };
        assert!(validate_preflight(&estimate).unwrap_err().contains("currently available RAM"));
    }
}
}
