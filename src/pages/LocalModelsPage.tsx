import { useCallback, useEffect, useState } from "react";
import {
  estimateModelResources, getRuntimeStatus, importGguf, listModels, loadModel,
  removeModel, setDefaultModel, unloadModel, type ModelRecord,
  type ModelResourceEstimate, type RuntimeStatus,
} from "../services/nativeBridge";

function sizeLabel(bytes: number) {
  return bytes >= 1024 ** 3 ? (bytes / 1024 ** 3).toFixed(2) + " GB" : (bytes / 1024 ** 2).toFixed(0) + " MB";
}

export function LocalModelsPage() {
  const [models, setModels] = useState<ModelRecord[]>([]);
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null);
  const [estimates, setEstimates] = useState<Record<string, ModelResourceEstimate>>({});
  const [busy, setBusy] = useState(false);
  const [busyModel, setBusyModel] = useState<string | null>(null);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");

  const refresh = useCallback(async () => {
    try {
      const [nextModels, nextRuntime] = await Promise.all([listModels(), getRuntimeStatus()]);
      setModels(nextModels);
      setRuntime(nextRuntime);
      if (nextRuntime.lastError) setError(nextRuntime.lastError);
    } catch {
      setError("Model registry and runtime status are available only inside the desktop app.");
    }
  }, []);

  useEffect(() => { void refresh(); }, [refresh]);

  const add = async () => {
    setBusy(true); setError(""); setNotice("");
    try {
      const result = await importGguf();
      if (result) {
        setNotice(result.duplicate
          ? "That exact model is already registered. Its original file location has been refreshed."
          : "GGUF registered. The file stays at its original location; Sync does not copy or move it.");
        setEstimates({});
        await refresh();
      }
    } catch (e) { setError(String(e)); }
    finally { setBusy(false); }
  };

  const makeDefault = async (id: string) => {
    setBusyModel(id); setError(""); setNotice("");
    try { await setDefaultModel(id); await refresh(); }
    catch (e) { setError(String(e)); }
    finally { setBusyModel(null); }
  };

  const inspect = async (id: string) => {
    setBusyModel(id); setError(""); setNotice("");
    try {
      const estimate = await estimateModelResources(id);
      setEstimates(current => ({ ...current, [id]: estimate }));
      setNotice("Informational estimate calculated. The Windows worker limit, not this estimate, enforces the ceiling.");
    } catch (e) { setError(String(e)); }
    finally { setBusyModel(null); }
  };

  const load = async (id: string) => {
    setBusyModel(id); setError(""); setNotice("");
    try {
      // Model replacement is an explicit user action. Stop the current worker
      // first, then load the selected model into a fresh limited worker.
      if (runtime?.isLoaded) await unloadModel();
      const next = await loadModel(id);
      setRuntime(next);
      setNotice("Selected model loaded into the isolated llama.cpp worker.");
    } catch (e) { setError(String(e)); await refresh(); }
    finally { setBusyModel(null); }
  };

  const unload = async () => {
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await unloadModel();
      setRuntime(next);
      setNotice("The active model and inference worker were unloaded.");
    } catch (e) { setError(String(e)); }
    finally { setBusy(false); }
  };

  const remove = async (model: ModelRecord) => {
    const confirmed = window.confirm(
      "Remove " + model.name + " from the Sync AI library? The original GGUF file will remain untouched."
    );
    if (!confirmed) return;
    setBusyModel(model.id); setError(""); setNotice("");
    try {
      await removeModel(model.id);
      setEstimates(current => { const next = { ...current }; delete next[model.id]; return next; });
      setNotice("Removed from Sync AI. The original model file was not deleted or moved.");
      await refresh();
    } catch (e) { setError(String(e)); }
    finally { setBusyModel(null); }
  };

  const isAnyBusy = busy || busyModel !== null || Boolean(runtime?.isLoading);

  return (
    <section className="shell-page shell-standard-page">
      <div className="shell-page-eyebrow"><span className="shell-eyebrow-line" />LOCAL INFERENCE LIBRARY</div>
      <div className="shell-page-heading-row">
        <div>
          <p className="shell-section-kicker">PHASE 4 · MODEL MANAGEMENT</p>
          <h1>Local models<span>.</span></h1>
          <p className="shell-page-description">Register GGUF files by their original path. Importing never copies, moves or deletes the source file.</p>
        </div>
        <button className="shell-model-import" type="button" disabled={isAnyBusy} onClick={() => void add()}>{busy ? "WORKING…" : "+ IMPORT GGUF"}</button>
      </div>

      <section className="shell-settings-panel">
        <div className="shell-panel-heading">
          <div><p className="shell-section-kicker">ISOLATED NATIVE RUNTIME</p><h2>{runtime?.isLoading ? "Loading model…" : runtime?.isLoaded ? runtime.loadedModelName : "No model loaded"}</h2></div>
          <span className="shell-panel-tag">{runtime?.backend ?? "llama.cpp · CPU · isolated worker"}</span>
        </div>
        {runtime?.isLoading ? <p className="shell-settings-footnote">Loading {runtime.loadingModelId}. The worker is starting outside the main interface process.</p>
          : runtime?.isLoaded ? <>
            <div className="shell-ram-stats">
              <div><small>CONTEXT</small><strong>{runtime.contextLength?.toLocaleString()} TOKENS</strong></div>
              <div><small>BATCH</small><strong>{runtime.batchSize}</strong></div>
              <div><small>WORKER PRIVATE COMMIT</small><strong>{runtime.workerPrivateBytes ? sizeLabel(runtime.workerPrivateBytes) : "—"}</strong></div>
            </div>
            <button type="button" disabled={isAnyBusy} onClick={() => void unload()}>Unload model</button>
          </> : <p className="shell-settings-footnote">Model loading is manual. A model that exceeds the worker's configured private-memory ceiling may fail to load without taking down the Sync interface.</p>}
      </section>

      <div className="shell-model-library-head">
        <div><h2>MODEL LIBRARY</h2><span>{models.length} {models.length === 1 ? "MODEL" : "MODELS"}</span></div>
        <span>ORIGINAL FILES · LOCAL REGISTRY</span>
      </div>
      {error && <p className="shell-model-message is-error" role="alert">{error}</p>}
      {notice && <p className="shell-model-message" role="status">{notice}</p>}

      {models.length === 0 ? <div className="shell-model-empty">
        <span>◈</span><strong>No GGUF models registered</strong>
        <p>Choose a GGUF file. Sync validates its header, calculates SHA-256 and stores its original absolute path in the local registry.</p>
        <button type="button" onClick={() => void add()} disabled={isAnyBusy}>Choose a GGUF file</button>
      </div> : <div className="shell-model-list">
        {models.map(model => {
          const estimate = estimates[model.id];
          const active = runtime?.loadedModelId === model.id;
          return <article className="shell-model-card" key={model.id}>
            <div className="shell-model-icon">◈</div>
            <div className="shell-model-info">
              <div className="shell-model-title-row">
                <h3>{model.name}</h3>
                {model.isDefault && <span className="shell-model-default">DEFAULT</span>}
                {active && <span className="shell-model-default">LOADED</span>}
                {!model.isAvailable && <span className="shell-model-default is-error">MISSING</span>}
              </div>
              <p>{model.fileName} · {sizeLabel(model.sizeBytes)}</p>
              <small>{model.architecture ?? "Unknown architecture"} · {model.quantization ?? "Unknown quantization"}</small>
              <small className={!model.isAvailable ? "is-error" : ""}>{model.filePath}</small>
              <small>SHA-256 · {model.sha256}</small>
              {estimate && <small>Estimated peak: {sizeLabel(estimate.estimatedPeakBytes)} · {estimate.configuredLimitBytes ? "configured worker ceiling " + sizeLabel(estimate.configuredLimitBytes) : "worker ceiling disabled"}</small>}
              {!model.isAvailable && <small className="is-error">This registered path is unavailable. Select the file again to relink it.</small>}
            </div>
            <div className="shell-model-actions">
              {!model.isDefault && <button type="button" disabled={isAnyBusy || !model.isAvailable} onClick={() => void makeDefault(model.id)}>Set default</button>}
              <button type="button" disabled={isAnyBusy || !model.isAvailable || active} onClick={() => void inspect(model.id)}>Estimate</button>
              {!active ? <button type="button" disabled={isAnyBusy || !model.isAvailable} onClick={() => void load(model.id)}>{runtime?.isLoaded ? "Load instead" : "Load"}</button>
                : <button type="button" disabled={isAnyBusy} onClick={() => void unload()}>Unload</button>}
              <details className="shell-model-menu">
                <summary aria-label={"More actions for " + model.name}>⋯</summary>
                <div>
                  {!model.isDefault && <button type="button" disabled={isAnyBusy || !model.isAvailable} onClick={() => void makeDefault(model.id)}>Make default</button>}
                  {!active && <button type="button" disabled={isAnyBusy || !model.isAvailable} onClick={() => void load(model.id)}>{runtime?.isLoaded ? "Load instead of active model" : "Load model"}</button>}
                  {active && <button type="button" disabled={isAnyBusy} onClick={() => void unload()}>Remove active selection</button>}
                  <button type="button" disabled={isAnyBusy || active} onClick={() => void remove(model)}>Remove from Sync AI</button>
                </div>
              </details>
            </div>
          </article>;
        })}
      </div>}
      <p className="shell-settings-footnote">Older managed-copy entries remain readable for compatibility. Removing any library entry deletes only its registry record; it never deletes the corresponding GGUF file.</p>
    </section>
  );
}
