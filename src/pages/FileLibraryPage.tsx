import { useEffect, useState } from "react";
import {
  importLibraryFile, listLibraryFiles, relinkLibraryFile, removeLibraryFile,
  type LibraryFileRecord,
} from "../services/nativeBridge";

function formatBytes(value: number): string {
  if (value < 1024) return value + " B";
  const units = ["KB", "MB", "GB", "TB"];
  let size = value / 1024;
  let index = 0;
  while (size >= 1024 && index < units.length - 1) { size /= 1024; index += 1; }
  return size.toFixed(size >= 10 ? 1 : 2) + " " + units[index];
}

export function FileLibraryPage() {
  const [files, setFiles] = useState<LibraryFileRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  async function refresh() {
    setLoading(true);
    try { setFiles(await listLibraryFiles()); setError(""); }
    catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setLoading(false); }
  }

  useEffect(() => { void refresh(); }, []);

  async function addFile() {
    setImporting(true); setError(""); setNotice("");
    try {
      const result = await importLibraryFile();
      if (result) setNotice(result.duplicate ? "That file was already known. Its path was refreshed." : "File registered in your local library.");
      await refresh();
    } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setImporting(false); }
  }

  async function relink(file: LibraryFileRecord) {
    setBusyId(file.id); setError(""); setNotice("");
    try {
      const result = await relinkLibraryFile(file.id);
      if (result) setNotice("File path updated.");
      await refresh();
    } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setBusyId(null); }
  }

  async function remove(file: LibraryFileRecord) {
    if (!window.confirm("Remove “" + file.fileName + "” from the library? The original file will not be deleted.")) return;
    setBusyId(file.id);
    try { await removeLibraryFile(file.id); await refresh(); setNotice("Library entry removed. Original file untouched."); }
    catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
    finally { setBusyId(null); }
  }

  return (
    <section className="shell-page shell-standard-page file-library-page">
      <div className="shell-page-eyebrow"><span className="shell-eyebrow-line" />USER-SELECTED LOCAL FILES</div>
      <div className="shell-page-heading-row">
        <div><p className="shell-section-kicker">A LOCAL INDEX, NOT A CLOUD DRIVE</p><h1>File Library<span>.</span></h1><p className="shell-page-description">Register files you choose, track their paths and detect when they move. Originals stay where they are.</p></div>
        <button className="library-primary-button" type="button" onClick={() => void addFile()} disabled={importing}>{importing ? "Checking file…" : "＋ Add file"}</button>
      </div>
      <div className="library-safety-note"><strong>File access boundary</strong><span>Sync currently stores file metadata and hashes only. It does not read library contents into chat. Any future AI file operation will require the separate, one-time approval flow.</span></div>
      {notice && <p className="library-feedback" role="status">{notice}</p>}
      {error && <p className="library-feedback is-error" role="alert">{error}</p>}
      <div className="library-list-heading"><div><span className="library-kicker">REGISTERED FILES</span><strong>{files.length} entries</strong></div><span>MAXIMUM 512 MiB PER FILE</span></div>
      {loading ? <p className="library-empty">Checking your file registry…</p> : files.length === 0 ? (
        <div className="library-empty-state"><span aria-hidden="true">▤</span><strong>Your File Library is empty.</strong><p>Add a file using the native picker. Sync stores its canonical location and SHA-256 fingerprint, not a second copy of the file.</p><button type="button" onClick={() => void addFile()} disabled={importing}>Choose a file</button></div>
      ) : (
        <div className="library-file-list">
          {files.map(file => (
            <article className="library-file-card" key={file.id}>
              <div className="library-file-icon">{(file.extension ?? "FILE").slice(0, 4).toUpperCase()}</div>
              <div className="library-file-main">
                <div className="library-file-title"><strong>{file.fileName}</strong><span className={file.isAvailable ? "is-available" : "is-missing"}>{file.isAvailable ? "AVAILABLE" : "MISSING"}</span></div>
                <p>{formatBytes(file.sizeBytes)} · Added {new Date(file.addedAt).toLocaleDateString()}</p>
                <small>{file.sourcePath}</small>
                <code>SHA-256 · {file.sha256}</code>
              </div>
              <div className="library-file-actions">
                {!file.isAvailable && <button type="button" disabled={busyId === file.id} onClick={() => void relink(file)}>{busyId === file.id ? "Checking…" : "Relink"}</button>}
                <button type="button" className="is-danger" disabled={busyId === file.id} onClick={() => void remove(file)}>Remove</button>
              </div>
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
