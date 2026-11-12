use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

const GIB: u64 = 1024 * 1024 * 1024;
pub const DEFAULT_CONTEXT_LENGTH: u32 = 2_048;
pub const DEFAULT_BATCH_SIZE: u32 = 128;
pub const DEFAULT_MICRO_BATCH_SIZE: u32 = 64;

#[derive(Debug, Clone, serde::Serialize)]
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
    pub memory_limit_bytes: Option<u64>,
    pub worker_pid: Option<u32>,
    pub worker_working_set_bytes: Option<u64>,
    pub worker_private_bytes: Option<u64>,
    pub hard_limit_supported: bool,
    pub last_error_code: Option<String>,
    pub last_error: Option<String>,
    pub backend: String,
}

impl Default for RuntimeStatus {
    fn default() -> Self {
        Self {
            is_loading: false, is_loaded: false,
            loading_model_id: None, loaded_model_id: None, loaded_model_name: None,
            loaded_at: None, estimated_memory_bytes: None, context_length: None,
            batch_size: None, memory_limit_bytes: None, worker_pid: None,
            worker_working_set_bytes: None, worker_private_bytes: None,
            hard_limit_supported: cfg!(windows), last_error_code: None, last_error: None,
            backend: "llama.cpp · CPU · isolated worker".into(),
        }
    }
}

pub struct RamLimiterConfig {
    pub enabled: bool,
    pub limit_gb: f64,
}

impl Default for RamLimiterConfig {
    fn default() -> Self { Self { enabled: false, limit_gb: 1.0 } }
}

impl RamLimiterConfig {
    pub fn from_json(raw: Option<String>) -> Result<Self, String> {
        let Some(raw) = raw else { return Ok(Self::default()); };
        let value: Value = serde_json::from_str(&raw)
            .map_err(|_| "Saved RAM limiter settings are invalid JSON. Open Settings and save them again.".to_string())?;
        let enabled = value.get("enabled").and_then(Value::as_bool)
            .ok_or_else(|| "Saved RAM limiter setting is missing a valid enabled value.".to_string())?;
        let limit_gb = value.get("limitGb").and_then(Value::as_f64)
            .ok_or_else(|| "Saved RAM limiter setting is missing a valid memory ceiling.".to_string())?;
        if !limit_gb.is_finite() || !(1.0..=1024.0).contains(&limit_gb) {
            return Err("Saved RAM limiter ceiling is outside the supported range of 1 to 1024 GB.".into());
        }
        Ok(Self { enabled, limit_gb })
    }

    pub fn limit_bytes(&self) -> Option<u64> {
        self.enabled.then(|| (self.limit_gb * GIB as f64).min(u64::MAX as f64) as u64)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelResourceEstimate {
    pub model_size_bytes: u64,
    pub estimated_peak_bytes: u64,
    pub available_ram_bytes: u64,
    pub configured_limit_bytes: Option<u64>,
    pub context_length: u32,
    pub batch_size: u32,
    pub estimate_kind: String,
}

pub fn estimate_resources(
    model_size_bytes: u64,
    available_ram_bytes: u64,
    config: &RamLimiterConfig,
) -> ModelResourceEstimate {
    let estimated_peak_bytes = model_size_bytes
        .saturating_add(model_size_bytes / 10)
        .saturating_add((DEFAULT_CONTEXT_LENGTH as u64).saturating_mul(64 * 1024))
        .saturating_add((DEFAULT_BATCH_SIZE as u64).saturating_mul(2 * 1024 * 1024))
        .saturating_add(128 * 1024 * 1024);
    ModelResourceEstimate {
        model_size_bytes,
        estimated_peak_bytes,
        available_ram_bytes,
        configured_limit_bytes: config.limit_bytes(),
        context_length: DEFAULT_CONTEXT_LENGTH,
        batch_size: DEFAULT_BATCH_SIZE,
        estimate_kind: "Informational model/session estimate only. The Windows Job Object enforces the committed-private-memory ceiling.".into(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerModelInfo {
    model_id: String,
    model_name: String,
    loaded_at: String,
    estimated_memory_bytes: u64,
    context_length: u32,
    batch_size: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerReply {
    request_id: String,
    ok: bool,
    model: Option<WorkerModelInfo>,
    error: Option<String>,
    error_code: Option<String>,
}

struct WorkerFailure {
    code: String,
    message: String,
}

struct WorkerProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    memory_limit_bytes: Option<u64>,
    #[cfg(windows)]
    _job: JobObjectHandle,
}

impl WorkerProcess {
    fn spawn(memory_limit_bytes: Option<u64>) -> Result<Self, String> {
        let executable = std::env::current_exe()
            .map_err(|e| format!("Cannot locate the Sync AI executable for its inference worker: {e}"))?;
        let mut child = Command::new(executable)
            .arg("--inference-worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Cannot start the isolated inference worker: {e}"))?;

        #[cfg(windows)]
        let job = match create_worker_job(&child, memory_limit_bytes) {
            Ok(value) => value,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
        };

        #[cfg(not(windows))]
        if memory_limit_bytes.is_some() {
            let _ = child.kill();
            let _ = child.wait();
            return Err("A hard inference-worker memory ceiling is supported only on Windows.".into());
        }

        let stdin = match child.stdin.take() {
            Some(value) => value,
            None => { let _ = child.kill(); return Err("Inference-worker stdin is unavailable.".into()); }
        };
        let stdout = match child.stdout.take() {
            Some(value) => value,
            None => { let _ = child.kill(); return Err("Inference-worker stdout is unavailable.".into()); }
        };

        Ok(Self {
            child, stdin, stdout: BufReader::new(stdout), memory_limit_bytes,
            #[cfg(windows)]
            _job: job,
        })
    }

    fn request(&mut self, mut payload: Value) -> Result<WorkerReply, WorkerFailure> {
        let request_id = Uuid::new_v4().to_string();
        payload["requestId"] = json!(request_id);
        serde_json::to_writer(&mut self.stdin, &payload)
            .map_err(|e| WorkerFailure { code: "worker_ipc_error".into(), message: format!("Cannot send a command to the inference worker: {e}") })?;
        self.stdin.write_all(b"\n").and_then(|_| self.stdin.flush())
            .map_err(|e| WorkerFailure { code: "worker_ipc_error".into(), message: format!("Cannot flush the inference-worker command: {e}") })?;

        let mut line = String::new();
        let count = self.stdout.read_line(&mut line)
            .map_err(|e| WorkerFailure { code: "worker_ipc_error".into(), message: format!("Cannot read the inference-worker response: {e}") })?;
        if count == 0 {
            let exit = self.child.try_wait().ok().flatten()
                .map(|status| status.to_string()).unwrap_or_else(|| "unknown exit status".into());
            let limited = self.memory_limit_bytes.is_some();
            return Err(WorkerFailure {
                code: if limited { "memory_limit_exceeded" } else { "worker_exited" }.into(),
                message: if limited {
                    format!("The inference worker exited before replying ({exit}). Windows may have rejected a private-memory commitment at the configured ceiling. The Sync AI interface remains running.")
                } else {
                    format!("The isolated inference worker exited before replying ({exit}). The Sync AI interface remains running.")
                },
            });
        }

        let response: WorkerReply = serde_json::from_str(&line)
            .map_err(|e| WorkerFailure { code: "worker_protocol_error".into(), message: format!("The inference worker returned an invalid response: {e}") })?;
        if response.request_id != request_id {
            return Err(WorkerFailure { code: "worker_protocol_error".into(), message: "The inference-worker response did not match the active request.".into() });
        }
        if !response.ok {
            return Err(WorkerFailure {
                code: response.error_code.unwrap_or_else(|| "worker_operation_failed".into()),
                message: response.error.unwrap_or_else(|| "The inference worker rejected the request.".into()),
            });
        }
        Ok(response)
    }

    fn memory_metrics(&self) -> (Option<u64>, Option<u64>) {
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::{
                Foundation::HANDLE,
                System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX},
            };
            let mut counters: PROCESS_MEMORY_COUNTERS_EX = unsafe { std::mem::zeroed() };
            counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
            let process = self.child.as_raw_handle() as HANDLE;
            let success = unsafe {
                K32GetProcessMemoryInfo(
                    process,
                    &mut counters as *mut PROCESS_MEMORY_COUNTERS_EX as *mut _,
                    counters.cb,
                )
            };
            if success != 0 {
                return (Some(counters.WorkingSetSize as u64), Some(counters.PrivateUsage as u64));
            }
        }
        #[cfg(not(windows))]
        {
            use sysinfo::{Pid, ProcessesToUpdate, System};
            let mut system = System::new();
            let pid = Pid::from_u32(self.child.id());
            let _ = system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
            if let Some(process) = system.process(pid) {
                return (Some(process.memory()), None);
            }
        }
        (None, None)
    }
}

impl Drop for WorkerProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(windows)]
struct JobObjectHandle(isize);

#[cfg(windows)]
unsafe impl Send for JobObjectHandle {}

#[cfg(windows)]
impl Drop for JobObjectHandle {
    fn drop(&mut self) {
        use windows_sys::Win32::Foundation::CloseHandle;
        unsafe { let _ = CloseHandle(self.0 as _); }
    }
}

#[cfg(windows)]
fn create_worker_job(child: &Child, memory_limit_bytes: Option<u64>) -> Result<JobObjectHandle, String> {
    use std::{mem::size_of, os::windows::io::AsRawHandle, ptr::null};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOB_OBJECT_LIMIT_PROCESS_MEMORY, JobObjectExtendedLimitInformation,
        },
    };

    let job = unsafe { CreateJobObjectW(null(), null()) };
    if job.is_null() { return Err("Windows could not create an inference-worker Job Object.".into()); }

    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    if let Some(limit) = memory_limit_bytes {
        limits.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        limits.ProcessMemoryLimit = usize::try_from(limit)
            .map_err(|_| "The configured memory ceiling is too large for this Windows process.".to_string())?;
    }

    let configured = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION as *const _,
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };
    if configured == 0 {
        unsafe { let _ = CloseHandle(job); }
        return Err("Windows could not configure the inference-worker memory limit.".into());
    }

    let assigned = unsafe { AssignProcessToJobObject(job, child.as_raw_handle() as HANDLE) };
    if assigned == 0 {
        unsafe { let _ = CloseHandle(job); }
        return Err("Windows could not assign the inference worker to its Job Object. No model was loaded.".into());
    }
    Ok(JobObjectHandle(job as isize))
}

#[derive(Default)]
struct RuntimeSlot {
    loading_model_id: Option<String>,
    removing_model_id: Option<String>,
    worker: Option<WorkerProcess>,
    loaded_model_id: Option<String>,
    loaded_model_name: Option<String>,
    loaded_at: Option<String>,
    estimated_memory_bytes: Option<u64>,
    context_length: Option<u32>,
    batch_size: Option<u32>,
    memory_limit_bytes: Option<u64>,
    last_error_code: Option<String>,
    last_error: Option<String>,
}

pub struct ModelRemovalGuard<'a> {
    runtime: &'a RuntimeManager,
    model_id: String,
}

impl Drop for ModelRemovalGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut slot) = self.runtime.slot.lock() {
            if slot.removing_model_id.as_deref() == Some(self.model_id.as_str()) {
                slot.removing_model_id = None;
            }
        }
    }
}

#[derive(Clone)]
pub struct RuntimeManager {
    slot: Arc<Mutex<RuntimeSlot>>,
}

impl RuntimeManager {
    pub fn new() -> Self { Self { slot: Arc::new(Mutex::new(RuntimeSlot::default())) } }

    pub fn status(&self) -> Result<RuntimeStatus, String> {
        let mut slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
        let mut worker_pid = None;
        let mut worker_working_set_bytes = None;
        let mut worker_private_bytes = None;
        let mut worker_exited = false;

        if let Some(worker) = slot.worker.as_mut() {
            match worker.child.try_wait() {
                Ok(Some(_)) | Err(_) => worker_exited = true,
                Ok(None) => {
                    worker_pid = Some(worker.child.id());
                    let (working_set, private_bytes) = worker.memory_metrics();
                    worker_working_set_bytes = working_set;
                    worker_private_bytes = private_bytes;
                }
            }
        }

        if worker_exited {
            slot.worker = None;
            slot.loaded_model_id = None;
            slot.loaded_model_name = None;
            slot.loaded_at = None;
            slot.estimated_memory_bytes = None;
            slot.context_length = None;
            slot.batch_size = None;
            if slot.last_error.is_none() {
                let limited = slot.memory_limit_bytes.is_some();
                slot.last_error_code = Some(if limited { "memory_limit_exceeded" } else { "worker_exited" }.into());
                slot.last_error = Some(if limited {
                    "The inference worker exited while a hard memory ceiling was active. Windows may have rejected a private-memory commitment at the configured limit. The main Sync AI process remains available.".into()
                } else {
                    "The isolated inference worker exited unexpectedly. The main Sync AI process remains available.".into()
                });
            }
        }

        Ok(RuntimeStatus {
            is_loading: slot.loading_model_id.is_some(),
            is_loaded: slot.loaded_model_id.is_some(),
            loading_model_id: slot.loading_model_id.clone(),
            loaded_model_id: slot.loaded_model_id.clone(),
            loaded_model_name: slot.loaded_model_name.clone(),
            loaded_at: slot.loaded_at.clone(),
            estimated_memory_bytes: slot.estimated_memory_bytes,
            context_length: slot.context_length,
            batch_size: slot.batch_size,
            memory_limit_bytes: slot.memory_limit_bytes,
            worker_pid,
            worker_working_set_bytes,
            worker_private_bytes,
            hard_limit_supported: cfg!(windows),
            last_error_code: slot.last_error_code.clone(),
            last_error: slot.last_error.clone(),
            backend: "llama.cpp · CPU · isolated worker".into(),
        })
    }

    fn begin_loading(&self, model_id: &str, memory_limit_bytes: Option<u64>) -> Result<(), String> {
        let mut slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
        if slot.removing_model_id.is_some() {
            return Err("A model is being removed. Wait for that operation to finish before loading.".into());
        }
        if slot.loading_model_id.is_some() {
            return Err("A model is already being loaded.".into());
        }
        if slot.worker.is_some() || slot.loaded_model_id.is_some() {
            if slot.loaded_model_id.as_deref() == Some(model_id) {
                return Err("This model is already loaded.".into());
            }
            return Err("Unload the active model before loading a different one.".into());
        }
        slot.loading_model_id = Some(model_id.to_string());
        slot.memory_limit_bytes = memory_limit_bytes;
        slot.last_error = None;
        slot.last_error_code = None;
        Ok(())
    }

    pub fn load_model(
        &self,
        model_id: String,
        model_name: String,
        path: std::path::PathBuf,
        memory_limit_bytes: Option<u64>,
    ) -> Result<RuntimeStatus, String> {
        self.begin_loading(&model_id, memory_limit_bytes)?;
        let result = (|| {
            if !path.is_file() {
                return Err(WorkerFailure {
                    code: "model_file_missing".into(),
                    message: "The selected GGUF file is missing or has moved. Remove it from the library and register it again from its current location.".into(),
                });
            }
            let mut worker = WorkerProcess::spawn(memory_limit_bytes)
                .map_err(|message| WorkerFailure { code: if memory_limit_bytes.is_some() { "memory_limit_setup_failed" } else { "worker_start_failed" }.into(), message })?;
            let response = worker.request(json!({
                "action": "load",
                "modelId": model_id,
                "modelName": model_name,
                "path": path.to_string_lossy(),
                "contextLength": DEFAULT_CONTEXT_LENGTH,
                "batchSize": DEFAULT_BATCH_SIZE,
                "microBatchSize": DEFAULT_MICRO_BATCH_SIZE
            }))?;
            let info = response.model.ok_or_else(|| WorkerFailure {
                code: "worker_protocol_error".into(),
                message: "The inference worker loaded a model but returned no model diagnostics.".into(),
            })?;
            Ok::<_, WorkerFailure>((worker, info))
        })();

        let mut slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
        slot.loading_model_id = None;
        match result {
            Ok((worker, info)) => {
                slot.loaded_model_id = Some(info.model_id);
                slot.loaded_model_name = Some(info.model_name);
                slot.loaded_at = Some(info.loaded_at);
                slot.estimated_memory_bytes = Some(info.estimated_memory_bytes);
                slot.context_length = Some(info.context_length);
                slot.batch_size = Some(info.batch_size);
                slot.worker = Some(worker);
                slot.last_error = None;
                slot.last_error_code = None;
                drop(slot);
                self.status()
            }
            Err(failure) => {
                slot.last_error_code = Some(failure.code);
                slot.last_error = Some(failure.message.clone());
                Err(failure.message)
            }
        }
    }

    pub fn reserve_removal(&self, model_id: &str) -> Result<ModelRemovalGuard<'_>, String> {
        let mut slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
        if slot.loading_model_id.is_some() {
            return Err("Wait for the current model load to finish before removing a model.".into());
        }
        if slot.loaded_model_id.as_deref() == Some(model_id) {
            return Err("Unload this model before removing it from the library.".into());
        }
        if slot.removing_model_id.is_some() {
            return Err("Another model removal is already in progress.".into());
        }
        slot.removing_model_id = Some(model_id.to_string());
        Ok(ModelRemovalGuard { runtime: self, model_id: model_id.to_string() })
    }

    pub fn unload(&self) -> Result<RuntimeStatus, String> {
        let worker = {
            let mut slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
            if slot.loading_model_id.is_some() {
                return Err("Wait for the current model load to finish before unloading.".into());
            }
            slot.worker.take()
        };
        if let Some(mut worker) = worker {
            let _ = worker.request(json!({ "action": "unload" }));
            let _ = worker.request(json!({ "action": "shutdown" }));
            drop(worker);
        }
        let mut slot = self.slot.lock().map_err(|_| "Runtime state lock is unavailable".to_string())?;
        slot.loaded_model_id = None;
        slot.loaded_model_name = None;
        slot.loaded_at = None;
        slot.estimated_memory_bytes = None;
        slot.context_length = None;
        slot.batch_size = None;
        slot.memory_limit_bytes = None;
        slot.last_error = None;
        slot.last_error_code = None;
        drop(slot);
        self.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_limiter_setting_defaults_to_disabled() {
        assert!(!RamLimiterConfig::from_json(None).unwrap().enabled);
    }

    #[test]
    fn malformed_limiter_setting_fails_closed() {
        assert!(RamLimiterConfig::from_json(Some(r#"{"enabled":true}"#.into())).is_err());
    }

    #[test]
    fn enabled_limiter_produces_a_hard_worker_limit() {
        let config = RamLimiterConfig::from_json(Some(r#"{"enabled":true,"limitGb":2.5}"#.into())).unwrap();
        assert_eq!(config.limit_bytes(), Some((2.5 * GIB as f64) as u64));
    }

    #[test]
    fn disabling_limiter_removes_the_hard_ceiling() {
        let config = RamLimiterConfig::from_json(Some(r#"{"enabled":false,"limitGb":2}"#.into())).unwrap();
        assert_eq!(config.limit_bytes(), None);
    }

    #[test]
    fn removal_reservation_prevents_load() {
        let runtime = RuntimeManager::new();
        let _guard = runtime.reserve_removal("model-a").unwrap();
        assert!(runtime.begin_loading("model-a", None).is_err());
    }
}
