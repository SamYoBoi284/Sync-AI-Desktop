import { useEffect, useState } from "react";
import {
  createWorkspaceArtifact, deleteWorkspaceArtifact, listWorkspaceArtifacts, updateWorkspaceArtifact,
  type WorkspaceArtifact, type WorkspaceArtifactInput,
} from "../services/nativeBridge";

const blankArtifact = (): WorkspaceArtifactInput => ({
  title: "", kind: "document", content: "", includeInContext: false,
  sourceChatId: null, sourceMessageId: null,
});

export function WorkspacePage() {
  const [artifacts, setArtifacts] = useState<WorkspaceArtifact[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [draft, setDraft] = useState<WorkspaceArtifactInput>(blankArtifact());
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  async function refresh(preferredId?: string | null) {
    setLoading(true);
    try {
      const rows = await listWorkspaceArtifacts(query);
      setArtifacts(rows);
      const target = preferredId ?? selectedId;
      const selected = rows.find(item => item.id === target) ?? rows[0] ?? null;
      setSelectedId(selected?.id ?? null);
      setDraft(selected ? {
        title: selected.title, kind: selected.kind, content: selected.content,
        includeInContext: selected.includeInContext,
        sourceChatId: selected.sourceChatId, sourceMessageId: selected.sourceMessageId,
      } : blankArtifact());
      setError("");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => { void refresh(null); }, [query]);

  function startNew() {
    setSelectedId(null);
    setDraft(blankArtifact());
    setMessage("");
    setError("");
  }

  function selectArtifact(item: WorkspaceArtifact) {
    setSelectedId(item.id);
    setDraft({
      title: item.title, kind: item.kind, content: item.content,
      includeInContext: item.includeInContext,
      sourceChatId: item.sourceChatId, sourceMessageId: item.sourceMessageId,
    });
    setMessage("");
    setError("");
  }

  async function save() {
    setSaving(true);
    setMessage("");
    setError("");
    try {
      const saved = selectedId
        ? await updateWorkspaceArtifact(selectedId, draft)
        : await createWorkspaceArtifact(draft);
      await refresh(saved.id);
      setMessage("Saved to Workspace.");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSaving(false);
    }
  }

  async function remove() {
    if (!selectedId || !window.confirm("Delete this Workspace artifact?")) return;
    setSaving(true);
    try {
      await deleteWorkspaceArtifact(selectedId);
      setSelectedId(null);
      await refresh(null);
      setMessage("Artifact deleted.");
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSaving(false);
    }
  }

  return (
    <section className="shell-page shell-standard-page workspace-page">
      <div className="shell-page-eyebrow"><span className="shell-eyebrow-line" />PERSISTENT WORKSPACE</div>
      <div className="shell-page-heading-row">
        <div>
          <p className="shell-section-kicker">YOUR WORK, KEPT LOCALLY</p>
          <h1>Workspace<span>.</span></h1>
          <p className="shell-page-description">Save useful replies, drafts, code and documents in one local space. Nothing is uploaded.</p>
        </div>
        <button className="workspace-primary-button" type="button" onClick={startNew}>＋ New artifact</button>
      </div>

      {message && <p className="workspace-feedback" role="status">{message}</p>}
      {error && <p className="workspace-feedback is-error" role="alert">{error}</p>}

      <div className="workspace-layout">
        <aside className="workspace-library-panel">
          <div className="workspace-panel-heading">
            <div><span className="workspace-kicker">SAVED MATERIAL</span><strong>{artifacts.length} artifacts</strong></div>
          </div>
          <input className="workspace-search" value={query} onChange={event => setQuery(event.target.value)} placeholder="Search Workspace…" aria-label="Search Workspace" />
          {loading ? <p className="workspace-empty">Loading Workspace…</p> : artifacts.length === 0 ? (
            <p className="workspace-empty">Your saved replies and documents will appear here.</p>
          ) : (
            <div className="workspace-artifact-list">
              {artifacts.map(item => (
                <button className={"workspace-artifact-entry" + (selectedId === item.id ? " is-active" : "")} type="button" key={item.id} onClick={() => selectArtifact(item)}>
                  <span className="workspace-artifact-type">{item.kind.toUpperCase()}</span>
                  <strong>{item.title}</strong>
                  <small>{new Date(item.updatedAt).toLocaleString()}</small>
                  {item.includeInContext && <span className="workspace-context-badge">CHAT CONTEXT</span>}
                </button>
              ))}
            </div>
          )}
        </aside>

        <section className="workspace-editor-panel">
          <div className="workspace-editor-heading">
            <div><span className="workspace-kicker">{selectedId ? "EDIT ARTIFACT" : "NEW ARTIFACT"}</span><h2>{selectedId ? "Refine your work" : "Create something"}</h2></div>
            {selectedId && <button className="workspace-delete-button" type="button" onClick={() => void remove()} disabled={saving}>Delete</button>}
          </div>
          <label className="workspace-field">Title
            <input value={draft.title} maxLength={200} onChange={event => setDraft(current => ({ ...current, title: event.target.value }))} placeholder="Give this artifact a name" />
          </label>
          <label className="workspace-field">Type
            <select value={draft.kind} onChange={event => setDraft(current => ({ ...current, kind: event.target.value }))}>
              <option value="document">Document</option><option value="report">Report</option><option value="code">Code</option><option value="snippet">Snippet</option><option value="other">Other</option>
            </select>
          </label>
          <label className="workspace-field workspace-content-field">Content
            <textarea value={draft.content} onChange={event => setDraft(current => ({ ...current, content: event.target.value }))} placeholder="Write or paste content here. Markdown is welcome." spellCheck />
          </label>
          <label className="workspace-context-toggle">
            <input type="checkbox" checked={draft.includeInContext} onChange={event => setDraft(current => ({ ...current, includeInContext: event.target.checked }))} />
            <span><strong>Include in chat context</strong><small>Selected content may be added to future local prompts. It is capped and remains under your control.</small></span>
          </label>
          {draft.sourceChatId && <p className="workspace-source-note">Saved from conversation {draft.sourceChatId.slice(0, 8)}…</p>}
          <div className="workspace-editor-actions">
            <button className="workspace-primary-button" type="button" onClick={() => void save()} disabled={saving || !draft.title.trim()}>{saving ? "Saving…" : "Save artifact"}</button>
            <span>Up to 2 MiB per artifact</span>
          </div>
        </section>
      </div>
    </section>
  );
}
