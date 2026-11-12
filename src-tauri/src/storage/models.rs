use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageDiagnostics {
    pub generation_id: Option<String>,
    pub model_id: Option<String>,
    pub model_label: Option<String>,
    pub backend: Option<String>,
    pub backend_version: Option<String>,
    pub status: Option<String>,
    pub prompt_tokens: Option<u64>,
    pub generated_tokens: Option<u64>,
    pub prompt_eval_ms: Option<f64>,
    pub generation_ms: Option<f64>,
    pub tokens_per_second: Option<f64>,
    pub context_size: Option<u64>,
    pub max_context_size: Option<u64>,
    pub temperature: Option<f64>,
    pub stop_reason: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl MessageRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatSummary {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: String,
    pub chat_id: String,
    pub role: MessageRole,
    pub body: String,
    pub created_at: String,
    pub position: u64,
    pub diagnostics: Option<MessageDiagnostics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatDetail {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMessage {
    pub role: MessageRole,
    pub body: String,
    pub diagnostics: Option<MessageDiagnostics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupDocument {
    pub format: String,
    pub format_version: u32,
    pub exported_at: String,
    pub app: BackupAppInfo,
    pub chats: Vec<BackupChat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupAppInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupChat {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: Vec<BackupMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupMessage {
    pub id: String,
    pub role: MessageRole,
    pub body: String,
    pub created_at: String,
    pub position: u64,
    pub diagnostics: Option<MessageDiagnostics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub imported_chats: u64,
    pub skipped_existing_chats: u64,
}
