import { useEffect, useState } from "react";
import {
  createChat,
  deleteChat,
  exportChatBackup,
  getChat,
  importChatBackup,
  listChats,
  renameChat,
  type ChatDetail,
  type ChatSummary,
  type MessageDiagnostics,
} from "../services/nativeBridge";

function formatDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}

function DiagnosticDetails({ diagnostics }: { diagnostics: MessageDiagnostics }) {
  const fields: Array<[string, string | number | null | undefined]> = [
    ["Status", diagnostics.status],
    ["Model", diagnostics.modelLabel ?? diagnostics.modelId],
    ["Backend", diagnostics.backend],
    ["Backend version", diagnostics.backendVersion],
    ["Prompt tokens", diagnostics.promptTokens],
    ["Generated tokens", diagnostics.generatedTokens],
    ["Prompt evaluation", diagnostics.promptEvalMs == null ? null : `${diagnostics.promptEvalMs} ms`],
    ["Generation time", diagnostics.generationMs == null ? null : `${diagnostics.generationMs} ms`],
    ["Tokens per second", diagnostics.tokensPerSecond],
    ["Context size", diagnostics.contextSize],
    ["Maximum context", diagnostics.maxContextSize],
    ["Temperature", diagnostics.temperature],
    ["Stop reason", diagnostics.stopReason],
    ["Error code", diagnostics.errorCode],
    ["Error details", diagnostics.errorMessage],
    ["Started", diagnostics.startedAt],
    ["Finished", diagnostics.finishedAt],
    ["Generation ID", diagnostics.generationId],
  ];

  return (
    <details className="history-diagnostics">
      <summary>Message diagnostics</summary>
      <dl>
        {fields.filter(([, value]) => value !== null && value !== undefined && value !== "").map(([label, value]) => (
          <div className="history-diagnostic-row" key={label}>
            <dt>{label}</dt>
            <dd>{String(value)}</dd>
          </div>
        ))}
      </dl>
    </details>
  );
}

export function HistoryPage() {
  const [chats, setChats] = useState<ChatSummary[]>([]);
  const [selected, setSelected] = useState<ChatDetail | null>(null);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  async function refresh(preferredId?: string) {
    setLoading(true);
    try {
      const rows = await listChats(query);
      setChats(rows);
      const target = preferredId ?? selected?.id;
      if (target && rows.some((chat) => chat.id === target)) {
        setSelected(await getChat(target));
      } else if (rows.length > 0) {
        setSelected(await getChat(rows[0].id));
      } else {
        setSelected(null);
      }
      setError(null);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    // Initial load and explicit search submissions are handled by the query effect below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query]);

  async function handleCreate() {
    setBusy(true);
    try {
      const chat = await createChat();
      await refresh(chat.id);
      setNotice("Conversation created.");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function handleRename() {
    if (!selected) return;
    const nextTitle = window.prompt("Rename conversation", selected.title);
    if (nextTitle === null || !nextTitle.trim()) return;
    setBusy(true);
    try {
      await renameChat(selected.id, nextTitle.trim());
      await refresh(selected.id);
      setNotice("Conversation renamed.");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function handleDelete() {
    if (!selected || !window.confirm(`Delete “${selected.title}”? This also deletes its messages.`)) return;
    setBusy(true);
    try {
      await deleteChat(selected.id);
      await refresh();
      setNotice("Conversation deleted.");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function handleExport() {
    setBusy(true);
    try {
      const backup = await exportChatBackup();
      const blob = new Blob([JSON.stringify(backup, null, 2)], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = `sync-ai-desktop-backup-${new Date().toISOString().slice(0, 10)}.json`;
      anchor.click();
      URL.revokeObjectURL(url);
      setNotice(`Exported ${backup.chats.length} conversation(s).`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function handleImport(file?: File) {
    if (!file) return;
    setBusy(true);
    try {
      if (file.size > 50 * 1024 * 1024) throw new Error("Backup exceeds the 50 MiB size limit.");
      const backup = JSON.parse(await file.text());
      const result = await importChatBackup(backup);
      await refresh();
      setNotice(`Imported ${result.importedChats}; skipped ${result.skippedExistingChats} existing conversation(s).`);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="shell-page history-page">
      <div className="shell-page-eyebrow"><span className="shell-eyebrow-line" />LOCAL CONVERSATIONS</div>
      <div className="history-toolbar">
        <div>
          <h1>History</h1>
          <p>Your conversations, stored on this device.</p>
        </div>
        <div className="history-actions">
          <button type="button" onClick={handleCreate} disabled={busy}>New chat</button>
          <button type="button" onClick={handleExport} disabled={busy}>Export backup</button>
          <label className="history-file-button">
            Import backup
            <input type="file" accept=".json,application/json" disabled={busy} onChange={(event) => {
              const file = event.currentTarget.files?.[0];
              void handleImport(file);
              event.currentTarget.value = "";
            }} />
          </label>
        </div>
      </div>
      {notice && <p className="history-notice" role="status">{notice}</p>}
      {error && <p className="history-error" role="alert">{error}</p>}
      <div className="history-layout">
        <aside className="history-list-panel">
          <input className="history-search" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search conversations…" aria-label="Search conversations" />
          {loading ? <p className="history-empty">Loading history…</p> : chats.length === 0 ? <p className="history-empty">No conversations found.</p> : (
            <ul className="history-chat-list">
              {chats.map((chat) => (
                <li key={chat.id}>
                  <button className={`history-chat-entry${selected?.id === chat.id ? " is-selected" : ""}`} onClick={() => void getChat(chat.id).then(setSelected)} type="button">
                    <strong>{chat.title}</strong>
                    <span>{chat.messageCount} messages</span>
                    <time>{formatDate(chat.updatedAt)}</time>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </aside>
        <article className="history-detail-panel">
          {!selected ? <p className="history-empty">Select a conversation to view its messages.</p> : (
            <>
              <header className="history-detail-header">
                <div>
                  <h2>{selected.title}</h2>
                  <p>Created {formatDate(selected.createdAt)}</p>
                </div>
                <div className="history-actions">
                  <button type="button" onClick={handleRename} disabled={busy}>Rename</button>
                  <button type="button" onClick={handleDelete} disabled={busy}>Delete</button>
                </div>
              </header>
              <div className="history-messages">
                {selected.messages.map((message) => (
                  <section className={`history-message history-message--${message.role}`} key={message.id}>
                    <div className="history-message-role">{message.role}</div>
                    <p>{message.body}</p>
                    {message.role === "assistant" && message.diagnostics && <DiagnosticDetails diagnostics={message.diagnostics} />}
                  </section>
                ))}
                {selected.messages.length === 0 && <p className="history-empty">This conversation has no messages yet.</p>}
              </div>
            </>
          )}
        </article>
      </div>
    </section>
  );
}
