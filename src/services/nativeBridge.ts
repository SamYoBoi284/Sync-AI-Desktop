import { Channel, invoke } from "@tauri-apps/api/core";

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
  isLoading: boolean; isLoaded: boolean; isGenerating: boolean; loadingModelId: string | null;
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


export interface GenerationEvent {
  event: "token" | "complete" | "error";
  generationId: string;
  token: string | null;
}
export interface GenerationResult {
  text: string;
  diagnostics: MessageDiagnostics;
}
export function generateChat(
  prompt: string,
  generationId: string,
  onEvent: (event: GenerationEvent) => void,
): Promise<GenerationResult> {
  const channel = new Channel<GenerationEvent>();
  channel.onmessage = onEvent;
  return invoke<GenerationResult>("generate_chat", { prompt, generationId, onEvent: channel });
}
export function cancelGeneration(generationId: string): Promise<void> {
  return invoke<void>("cancel_generation", { generationId });
}

export function appendAssistantMessageAfter(
  chatId: string,
  userMessageId: string,
  message: { role: "assistant"; body: string; diagnostics: MessageDiagnostics },
): Promise<ChatMessage> {
  return invoke<ChatMessage>("append_assistant_message_after", { chatId, userMessageId, message });
}


export type ArtifactKind = "document" | "code" | "snippet" | "report" | "other";
export interface WorkspaceArtifact {
  id: string; title: string; kind: ArtifactKind; content: string;
  sourceChatId: string | null; sourceMessageId: string | null;
  includeInContext: boolean; createdAt: string; updatedAt: string;
}
export interface WorkspaceArtifactInput {
  title: string; kind: ArtifactKind; content: string; includeInContext: boolean;
  sourceChatId?: string | null; sourceMessageId?: string | null;
}
export interface WorkspaceContextEntry { kind: string; title: string; content: string; }
export function listWorkspaceArtifacts(query = ""): Promise<WorkspaceArtifact[]> {
  return invoke<WorkspaceArtifact[]>("list_workspace_artifacts", { query: query || null });
}
export function getWorkspaceArtifact(artifactId: string): Promise<WorkspaceArtifact> {
  return invoke<WorkspaceArtifact>("get_workspace_artifact", { artifactId });
}
export function createWorkspaceArtifact(artifact: WorkspaceArtifactInput): Promise<WorkspaceArtifact> {
  return invoke<WorkspaceArtifact>("create_workspace_artifact", { artifact });
}
export function updateWorkspaceArtifact(artifactId: string, artifact: WorkspaceArtifactInput): Promise<WorkspaceArtifact> {
  return invoke<WorkspaceArtifact>("update_workspace_artifact", { artifactId, artifact });
}
export function deleteWorkspaceArtifact(artifactId: string): Promise<void> {
  return invoke<void>("delete_workspace_artifact", { artifactId });
}
export function listWorkspaceContext(): Promise<WorkspaceContextEntry[]> {
  return invoke<WorkspaceContextEntry[]>("list_workspace_context");
}

export type TaskStatus = "todo" | "in-progress" | "done";
export type TaskPriority = "low" | "normal" | "high";
export interface TaskRecord {
  id: string; title: string; details: string; status: TaskStatus; priority: TaskPriority;
  dueDate: string | null; createdAt: string; updatedAt: string;
}
export interface TaskInput {
  title: string; details: string; status: TaskStatus; priority: TaskPriority; dueDate: string | null;
}
export function listTasks(query = ""): Promise<TaskRecord[]> {
  return invoke<TaskRecord[]>("list_tasks", { query: query || null });
}
export function createTask(task: TaskInput): Promise<TaskRecord> {
  return invoke<TaskRecord>("create_task", { task });
}
export function updateTask(taskId: string, task: TaskInput): Promise<TaskRecord> {
  return invoke<TaskRecord>("update_task", { taskId, task });
}
export function deleteTask(taskId: string): Promise<void> {
  return invoke<void>("delete_task", { taskId });
}

export interface NoteRecord {
  id: string; title: string; content: string; pinned: boolean; includeInContext: boolean;
  createdAt: string; updatedAt: string;
}
export interface NoteInput { title: string; content: string; pinned: boolean; includeInContext: boolean; }
export function listNotes(query = ""): Promise<NoteRecord[]> {
  return invoke<NoteRecord[]>("list_notes", { query: query || null });
}
export function createNote(note: NoteInput): Promise<NoteRecord> {
  return invoke<NoteRecord>("create_note", { note });
}
export function updateNote(noteId: string, note: NoteInput): Promise<NoteRecord> {
  return invoke<NoteRecord>("update_note", { noteId, note });
}
export function deleteNote(noteId: string): Promise<void> {
  return invoke<void>("delete_note", { noteId });
}

export interface LibraryFileRecord {
  id: string; fileName: string; extension: string | null; sourcePath: string;
  sizeBytes: number; sha256: string; addedAt: string; isAvailable: boolean;
}
export interface LibraryImportResult { file: LibraryFileRecord; duplicate: boolean; }
export function listLibraryFiles(): Promise<LibraryFileRecord[]> {
  return invoke<LibraryFileRecord[]>("list_library_files");
}
export function importLibraryFile(): Promise<LibraryImportResult | null> {
  return invoke<LibraryImportResult | null>("import_library_file");
}
export function relinkLibraryFile(fileId: string): Promise<LibraryFileRecord | null> {
  return invoke<LibraryFileRecord | null>("relink_library_file", { fileId });
}
export function removeLibraryFile(fileId: string): Promise<void> {
  return invoke<void>("remove_library_file", { fileId });
}
