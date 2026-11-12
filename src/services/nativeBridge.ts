import { invoke } from "@tauri-apps/api/core";

export interface AppInfo { appName: string; version: string; operatingSystem: string; architecture: string; }
export async function getAppInfo(): Promise<AppInfo> { return invoke<AppInfo>("get_app_info"); }
export async function consumeStartupSplash(): Promise<boolean> { return invoke<boolean>("consume_startup_splash"); }

export type MessageRole = "system" | "user" | "assistant";
export interface MessageDiagnostics {
  generationId?: string | null; modelId?: string | null; modelLabel?: string | null;
  backend?: string | null; backendVersion?: string | null; status?: string | null;
  promptTokens?: number | null; generatedTokens?: number | null; promptEvalMs?: number | null;
  generationMs?: number | null; tokensPerSecond?: number | null; contextSize?: number | null;
  maxContextSize?: number | null; temperature?: number | null; stopReason?: string | null;
  errorCode?: string | null; errorMessage?: string | null; startedAt?: string | null; finishedAt?: string | null;
}
export interface ChatSummary { id: string; title: string; createdAt: string; updatedAt: string; messageCount: number; }
export interface ChatMessage { id: string; chatId: string; role: MessageRole; body: string; createdAt: string; position: number; diagnostics: MessageDiagnostics | null; }
export interface ChatDetail { id: string; title: string; createdAt: string; updatedAt: string; messages: ChatMessage[]; }
export interface BackupDocument {
  format: "sync-ai-desktop-backup"; formatVersion: 1; exportedAt: string;
  app: { name: string; version: string };
  chats: Array<{ id: string; title: string; createdAt: string; updatedAt: string;
    messages: Array<{ id: string; role: MessageRole; body: string; createdAt: string; position: number; diagnostics: MessageDiagnostics | null }> }>;
}
export interface ImportSummary { importedChats: number; skippedExistingChats: number; }

export function listChats(query = "", limit = 100, offset = 0): Promise<ChatSummary[]> { return invoke<ChatSummary[]>("list_chats", { query, limit, offset }); }
export function createChat(title?: string): Promise<ChatDetail> { return invoke<ChatDetail>("create_chat", { title: title ?? null }); }
export function getChat(chatId: string): Promise<ChatDetail> { return invoke<ChatDetail>("get_chat", { chatId }); }
export function renameChat(chatId: string, title: string): Promise<void> { return invoke<void>("rename_chat", { chatId, title }); }
export function deleteChat(chatId: string): Promise<void> { return invoke<void>("delete_chat", { chatId }); }
export function exportChatBackup(): Promise<BackupDocument> { return invoke<BackupDocument>("export_chat_backup"); }
export function importChatBackup(backup: BackupDocument): Promise<ImportSummary> { return invoke<ImportSummary>("import_chat_backup", { backup }); }
export function getSetting(key: string): Promise<string | null> { return invoke<string | null>("get_setting", { key }); }
export function setSetting(key: string, valueJson: string): Promise<void> { return invoke<void>("set_setting", { key, valueJson }); }
export function appendMessage(chatId: string, message: { role: MessageRole; body: string; diagnostics?: MessageDiagnostics | null }): Promise<ChatMessage> { return invoke<ChatMessage>("append_message", { chatId, message }); }
export function updateMessageDiagnostics(chatId: string, messageId: string, diagnostics: MessageDiagnostics): Promise<void> { return invoke<void>("update_message_diagnostics", { chatId, messageId, diagnostics }); }

export interface HostRamInfo { totalBytes: number; availableBytes: number; }
export interface ModelRecord {
  id: string; name: string; fileName: string; sizeBytes: number; sha256: string;
  architecture: string | null; quantization: string | null; importedAt: string; isDefault: boolean;
  sourcePath: string | null; filePath: string; isAvailable: boolean;
}
export interface ModelImportResult { model: ModelRecord; duplicate: boolean; }
export function getHostRamInfo(): Promise<HostRamInfo> { return invoke<HostRamInfo>("get_host_ram_info"); }
export function listModels(): Promise<ModelRecord[]> { return invoke<ModelRecord[]>("list_models"); }
export function importGguf(): Promise<ModelImportResult | null> { return invoke<ModelImportResult | null>("import_gguf"); }
export function setDefaultModel(modelId: string): Promise<void> { return invoke<void>("set_default_model", { modelId }); }
export function removeModel(modelId: string): Promise<void> { return invoke<void>("remove_model", { modelId }); }

export interface RuntimeStatus {
  isLoading: boolean; isLoaded: boolean; loadingModelId: string | null;
  loadedModelId: string | null; loadedModelName: string | null; loadedAt: string | null;
  estimatedMemoryBytes: number | null; contextLength: number | null; batchSize: number | null;
  memoryLimitBytes: number | null; workerPid: number | null;
  workerWorkingSetBytes: number | null; workerPrivateBytes: number | null;
  hardLimitSupported: boolean; lastErrorCode: string | null; lastError: string | null; backend: string;
}
export interface ModelResourceEstimate {
  modelSizeBytes: number; estimatedPeakBytes: number; availableRamBytes: number;
  configuredLimitBytes: number | null; contextLength: number; batchSize: number; estimateKind: string;
}
export function getRuntimeStatus(): Promise<RuntimeStatus> { return invoke<RuntimeStatus>("get_runtime_status"); }
export function estimateModelResources(modelId: string): Promise<ModelResourceEstimate> { return invoke<ModelResourceEstimate>("estimate_model_resources", { modelId }); }
export function loadModel(modelId: string): Promise<RuntimeStatus> { return invoke<RuntimeStatus>("load_model", { modelId }); }
export function unloadModel(): Promise<RuntimeStatus> { return invoke<RuntimeStatus>("unload_model"); }
