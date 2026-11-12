import { useEffect, useState } from "react";
import {
  createNote, createTask, deleteNote, deleteTask, listNotes, listTasks, updateNote, updateTask,
  type NoteInput, type NoteRecord, type TaskInput, type TaskRecord,
} from "../services/nativeBridge";

type View = "tasks" | "notes";
const blankTask = (): TaskInput => ({ title: "", details: "", status: "todo", priority: "normal", dueDate: null });
const blankNote = (): NoteInput => ({ title: "", content: "", pinned: false, includeInContext: false });

export function TasksNotesPage() {
  const [view, setView] = useState<View>("tasks");
  const [tasks, setTasks] = useState<TaskRecord[]>([]);
  const [notes, setNotes] = useState<NoteRecord[]>([]);
  const [taskDraft, setTaskDraft] = useState<TaskInput>(blankTask());
  const [noteDraft, setNoteDraft] = useState<NoteInput>(blankNote());
  const [selectedNoteId, setSelectedNoteId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  async function refreshTasks() {
    const rows = await listTasks();
    setTasks(rows);
  }
  async function refreshNotes(preferredId?: string | null) {
    const rows = await listNotes();
    setNotes(rows);
    const selected = rows.find(note => note.id === (preferredId ?? selectedNoteId)) ?? rows[0] ?? null;
    setSelectedNoteId(selected?.id ?? null);
    setNoteDraft(selected ? {
      title: selected.title, content: selected.content,
      pinned: selected.pinned, includeInContext: selected.includeInContext,
    } : blankNote());
  }

  useEffect(() => {
    let active = true;
    Promise.all([listTasks(), listNotes()])
      .then(([taskRows, noteRows]) => {
        if (!active) return;
        setTasks(taskRows);
        setNotes(noteRows);
        if (noteRows.length) {
          const first = noteRows[0];
          setSelectedNoteId(first.id);
          setNoteDraft({ title: first.title, content: first.content, pinned: first.pinned, includeInContext: first.includeInContext });
        }
      })
      .catch(cause => { if (active) setError(cause instanceof Error ? cause.message : String(cause)); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, []);

  async function addTask() {
    setSaving(true); setError(""); setNotice("");
    try {
      await createTask(taskDraft);
      setTaskDraft(blankTask());
      await refreshTasks();
      setNotice("Task added.");
    } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setSaving(false); }
  }

  async function changeTask(task: TaskRecord, patch: Partial<TaskInput>) {
    try {
      await updateTask(task.id, {
        title: task.title, details: task.details, status: task.status,
        priority: task.priority, dueDate: task.dueDate, ...patch,
      });
      await refreshTasks();
    } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
  }

  async function removeTask(task: TaskRecord) {
    if (!window.confirm("Delete task “" + task.title + "”?")) return;
    try { await deleteTask(task.id); await refreshTasks(); setNotice("Task deleted."); }
    catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
  }

  function selectNote(note: NoteRecord) {
    setSelectedNoteId(note.id);
    setNoteDraft({ title: note.title, content: note.content, pinned: note.pinned, includeInContext: note.includeInContext });
    setNotice("");
  }

  function newNote() {
    setSelectedNoteId(null);
    setNoteDraft(blankNote());
    setNotice("");
  }

  async function saveNote() {
    setSaving(true); setError(""); setNotice("");
    try {
      const saved = selectedNoteId
        ? await updateNote(selectedNoteId, noteDraft)
        : await createNote(noteDraft);
      await refreshNotes(saved.id);
      setNotice("Note saved.");
    } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setSaving(false); }
  }

  async function removeNote() {
    if (!selectedNoteId || !window.confirm("Delete this note?")) return;
    setSaving(true);
    try {
      await deleteNote(selectedNoteId);
      setSelectedNoteId(null);
      await refreshNotes(null);
      setNotice("Note deleted.");
    } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setSaving(false); }
  }

  return (
    <section className="shell-page shell-standard-page tasks-notes-page">
      <div className="shell-page-eyebrow"><span className="shell-eyebrow-line" />PERSONAL ORGANIZATION</div>
      <div className="shell-page-heading-row">
        <div><p className="shell-section-kicker">SMALL THINGS, KEPT TOGETHER</p><h1>Tasks & Notes<span>.</span></h1><p className="shell-page-description">Lightweight planning and notes, stored in the same local database as your conversations.</p></div>
      </div>
      <div className="tasks-tabs" role="tablist" aria-label="Tasks and Notes">
        <button type="button" role="tab" aria-selected={view === "tasks"} className={view === "tasks" ? "is-active" : ""} onClick={() => setView("tasks")}>Tasks <span>{tasks.length}</span></button>
        <button type="button" role="tab" aria-selected={view === "notes"} className={view === "notes" ? "is-active" : ""} onClick={() => setView("notes")}>Notes <span>{notes.length}</span></button>
      </div>
      {notice && <p className="tasks-feedback" role="status">{notice}</p>}
      {error && <p className="tasks-feedback is-error" role="alert">{error}</p>}
      {loading ? <p className="tasks-empty">Loading local items…</p> : view === "tasks" ? (
        <div className="tasks-workspace">
          <form className="task-create-panel" onSubmit={event => { event.preventDefault(); void addTask(); }}>
            <div className="tasks-panel-heading"><span className="tasks-kicker">CAPTURE A TASK</span><strong>New task</strong></div>
            <label>Title<input value={taskDraft.title} onChange={event => setTaskDraft(current => ({ ...current, title: event.target.value }))} maxLength={200} placeholder="What needs doing?" required /></label>
            <label>Details<textarea value={taskDraft.details} onChange={event => setTaskDraft(current => ({ ...current, details: event.target.value }))} rows={3} placeholder="Optional details…" /></label>
            <div className="task-create-options">
              <label>Priority<select value={taskDraft.priority} onChange={event => setTaskDraft(current => ({ ...current, priority: event.target.value }))}><option value="low">Low</option><option value="normal">Normal</option><option value="high">High</option></select></label>
              <label>Due date<input type="date" value={taskDraft.dueDate ?? ""} onChange={event => setTaskDraft(current => ({ ...current, dueDate: event.target.value || null }))} /></label>
            </div>
            <button className="tasks-primary-button" type="submit" disabled={saving || !taskDraft.title.trim()}>{saving ? "Saving…" : "Add task"}</button>
          </form>
          <section className="task-list-panel">
            <div className="tasks-panel-heading"><span className="tasks-kicker">YOUR QUEUE</span><strong>{tasks.length} tasks</strong></div>
            {tasks.length === 0 ? <p className="tasks-empty">No tasks yet. One tiny item at a time.</p> : <div className="task-list">
              {tasks.map(task => (
                <article className={"task-card task-card--" + task.status} key={task.id}>
                  <div className="task-card-main">
                    <div className="task-title-line"><strong>{task.title}</strong><span className={"task-priority task-priority--" + task.priority}>{task.priority}</span></div>
                    {task.details && <p>{task.details}</p>}
                    <small>{task.dueDate ? "Due " + task.dueDate : "No due date"} · Updated {new Date(task.updatedAt).toLocaleDateString()}</small>
                  </div>
                  <div className="task-card-controls">
                    <select aria-label={"Status for " + task.title} value={task.status} onChange={event => void changeTask(task, { status: event.target.value })}>
                      <option value="todo">To do</option><option value="in-progress">In progress</option><option value="done">Done</option>
                    </select>
                    <button type="button" onClick={() => void removeTask(task)} aria-label={"Delete " + task.title}>×</button>
                  </div>
                </article>
              ))}
            </div>}
          </section>
        </div>
      ) : (
        <div className="notes-workspace">
          <aside className="notes-list-panel">
            <div className="tasks-panel-heading"><span className="tasks-kicker">LOCAL NOTES</span><button type="button" onClick={newNote}>＋ New</button></div>
            {notes.length === 0 ? <p className="tasks-empty">No notes saved yet.</p> : notes.map(note => (
              <button type="button" key={note.id} className={"note-list-entry" + (selectedNoteId === note.id ? " is-active" : "")} onClick={() => selectNote(note)}>
                <strong>{note.pinned && "◆ "}{note.title}</strong><small>{note.includeInContext ? "Included in chat context" : new Date(note.updatedAt).toLocaleDateString()}</small>
              </button>
            ))}
          </aside>
          <section className="note-editor-panel">
            <div className="tasks-panel-heading"><span className="tasks-kicker">{selectedNoteId ? "EDIT NOTE" : "NEW NOTE"}</span>{selectedNoteId && <button className="notes-delete-button" type="button" onClick={() => void removeNote()} disabled={saving}>Delete</button>}</div>
            <label>Title<input value={noteDraft.title} onChange={event => setNoteDraft(current => ({ ...current, title: event.target.value }))} maxLength={200} placeholder="Note title" /></label>
            <label className="note-content-label">Content<textarea value={noteDraft.content} onChange={event => setNoteDraft(current => ({ ...current, content: event.target.value }))} placeholder="Keep a thought, plan or reference here…" /></label>
            <label className="note-option"><input type="checkbox" checked={noteDraft.pinned} onChange={event => setNoteDraft(current => ({ ...current, pinned: event.target.checked }))} /> Pin this note</label>
            <label className="note-option"><input type="checkbox" checked={noteDraft.includeInContext} onChange={event => setNoteDraft(current => ({ ...current, includeInContext: event.target.checked }))} /> Include in future chat context</label>
            <button className="tasks-primary-button" type="button" onClick={() => void saveNote()} disabled={saving || !noteDraft.title.trim()}>{saving ? "Saving…" : "Save note"}</button>
            <p className="notes-context-hint">Only notes explicitly selected for context are added to local prompts.</p>
          </section>
        </div>
      )}
    </section>
  );
}
