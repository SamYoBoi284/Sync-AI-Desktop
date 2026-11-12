import { invoke } from "@tauri-apps/api/core";

export interface AppInfo {
  appName: string;
  version: string;
  operatingSystem: string;
  architecture: string;
}

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}


export async function consumeStartupSplash(): Promise<boolean> {
  return invoke<boolean>("consume_startup_splash");
}


export type MessageRole = "system" | "user" | "assistant";

export interface MessageDiagnostics {
  generationId?: string | null;
  modelId?: string | null;
  modelLabel?: string | null;
  backend?: string | null;
  backendVersion?: string | null;
  status?: string | null;
  promptTokens?: number | null;
  generatedTokens?: number | null;
  promptEvalMs?: number | null;
  generationMs?: number | null;
  tokensPerSecond?: number | null;
  contextSize?: number | null;
  maxContextSize?: number | null;
  temperature?: number | null;
  stopReason?: string | null;
  errorCode?: string | null;
  errorMessage?: string | null;
  startedAt?: string | null;
  finishedAt?: string | null;
}

export interface ChatSummary {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  messageCount: number;
}

export interface ChatMessage {
  id: string;
  chatId: string;
  role: MessageRole;
  body: string;
  createdAt: string;
  position: number;
  diagnostics: MessageDiagnostics | null;
}

export interface ChatDetail {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  messages: ChatMessage[];
}

export interface BackupDocument {
  format: "sync-ai-desktop-backup";
  formatVersion: 1;
  exportedAt: string;
  app: { name: string; version: string };
  chats: Array<{
    id: string;
    title: string;
    createdAt: string;
    updatedAt: string;
    messages: Array<{
      id: string;
      role: MessageRole;
      body: string;
      createdAt: string;
      position: number;
      diagnostics: MessageDiagnostics | null;
    }>;
  }>;
}

export interface ImportSummary {
  importedChats: number;
  skippedExistingChats: number;
}

export function listChats(query = "", limit = 100, offset = 0): Promise<ChatSummary[]> {
  return invoke("list_chats", { query, limit, offset });
}

export function createChat(title?: string): Promise<ChatDetail> {
  return invoke("create_chat", { title: title ?? null });
}

export function getChat(chatId: string): Promise<ChatDetail> {
  return invoke("get_chat", { chatId });
}

export function renameChat(chatId: string, title: string): Promise<void> {
  return invoke("rename_chat", { chatId, title });
}

export function deleteChat(chatId: string): Promise<void> {
  return invoke("delete_chat", { chatId });
}

export function exportChatBackup(): Promise<BackupDocument> {
  return invoke("export_chat_backup");
}

export function importChatBackup(backup: BackupDocument): Promise<ImportSummary> {
  return invoke("import_chat_backup", { backup });
}
