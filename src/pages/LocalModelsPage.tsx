import { useCallback, useEffect, useState } from "react";
import {
  estimateModelResources, getRuntimeStatus, importGguf, listModels, loadModel,
  removeModel, setDefaultModel, unloadModel, type ModelRecord,
  type ModelResourceEstimate, type RuntimeStatus,
} from "../services/nativeBridge";

function sizeLabel(bytes: number) {
  return bytes >= 1024 ** 3
    ? (bytes / 1024 ** 3).toFixed(2) + " GB"
    : (bytes / 1024 ** 2).toFixed(0) + " MB";
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
      setError("");
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
          ? "That exact model is already in your library. Its GGUF metadata has been refreshed."
          : "Model imported, SHA-256 verified, and GGUF metadata parsed.");
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
      setEstimates((current) => ({ ...current, [id]: estimate }));
      setNotice("Pre-load memory estimate calculated for a 2,048-token context and 128-token batch.");
    } catch (e) { setError(String(e)); }
    finally { setBusyModel(null); }
  };

  const load = async (id: string) => {
    setBusyModel(id); setError(""); setNotice("");
    setRuntime((current) => current ? {
      ...current, isLoading: true, loadingModelId: id,
    } : {
      isLoading: true, isLoaded: false, loadingModelId: id,
      loadedModelId: null, loadedModelName: null, loadedAt: null,
      estimatedMemoryBytes: null, contextLength: null, batchSize: null,
      backend: "llama.cpp · CPU",
    });
    try {
      const next = await loadModel(id);
      setRuntime(next);
      setNotice("Model and 2K context session loaded into the native llama.cpp runtime.");
    } catch (e) { setError(String(e)); await refresh(); }
    finally { setBusyModel(null); }
  };

  const unload = async () => {
    setBusy(true); setError(""); setNotice("");
    try {
      const next = await unloadModel();
      setRuntime(next);
      setNotice("The active model and context session were unloaded.");
    } catch (e) { setError(String(e)); }
    finally { setBusy(false); }
  };

  const remove = async (model: ModelRecord) => {
    if (!window.confirm("Remove " + model.name + " from Sync AI and delete its managed model file?")) return;
    setBusyModel(model.id); setError(""); setNotice("");
    try {
      await removeModel(model.id);
      setEstimates((current) => { const next = { ...current }; delete next[model.id]; return next; });
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
          <p className="shell-section-kicker">PHASE 4 · STAGE 2</p>
          <h1>Local models<span>.</span></h1>
          <p className="shell-page-description">Manage GGUF files and load a local CPU inference runtime. Models are never loaded automatically at startup.</p>
        </div>
        <button className="shell-model-import" type="button" disabled={isAnyBusy} onClick={() => void add()}>
          {busy ? "WORKING…" : "+ IMPORT GGUF"}
        </button>
      </div>

      <section className="shell-settings-panel">
        <div className="shell-panel-heading">
          <div>
            <p className="shell-section-kicker">NATIVE RUNTIME</p>
            <h2>{runtime?.isLoading ? "Loading model…" : runtime?.isLoaded ? runtime.loadedModelName : "No model loaded"}</h2>
          </div>
          <span className="shell-panel-tag">{runtime?.backend ?? "llama.cpp · CPU"}</span>
        </div>
        {runtime?.isLoading ? (
          <p className="shell-settings-footnote">Loading {runtime.loadingModelId}. Sync is checking the model and preparing its native context.</p>
        ) : runtime?.isLoaded ? (
          <>
            <div className="shell-ram-stats">
              <div><small>CONTEXT</small><strong>{runtime.contextLength?.toLocaleString()} TOKENS</strong></div>
              <div><small>BATCH</small><strong>{runtime.batchSize}</strong></div>
              <div><small>ESTIMATED MEMORY</small><strong>{runtime.estimatedMemoryBytes ? sizeLabel(runtime.estimatedMemoryBytes) : "—"}</strong></div>
            </div>
            <button type="button" disabled={isAnyBusy} onClick={() => void unload()}>Unload model</button>
          </>
        ) : (
          <p className="shell-settings-footnote">Loading is manual. The runtime uses CPU-only inference with a 2,048-token context and 128-token batch. Chat generation connects in Stage 3.</p>
        )}
      </section>

      <div className="shell-model-library-head">
        <div><h2>MODEL LIBRARY</h2><span>{models.length} {models.length === 1 ? "MODEL" : "MODELS"}</span></div>
        <span>LOCAL STORAGE ONLY</span>
      </div>
      {error && <p className="shell-model-message is-error" role="alert">{error}</p>}
      {notice && <p className="shell-model-message" role="status">{notice}</p>}

      {models.length === 0 ? (
        <div className="shell-model-empty">
          <span>◈</span><strong>No GGUF models imported</strong>
          <p>Import a valid GGUF file. Sync checks its header, reads model metadata, calculates SHA-256 and detects duplicates.</p>
          <button type="button" onClick={() => void add()} disabled={isAnyBusy}>Choose a GGUF file</button>
        </div>
      ) : (
        <div className="shell-model-list">
          {models.map((model) => {
            const estimate = estimates[model.id];
            const active = runtime?.loadedModelId === model.id;
            const loading = runtime?.loadingModelId === model.id;
            return (
              <article className="shell-model-card" key={model.id}>
                <div className="shell-model-icon">◈</div>
                <div className="shell-model-info">
                  <div className="shell-model-title-row">
                    <h3>{model.name}</h3>
                    {model.isDefault && <span className="shell-model-default">DEFAULT</span>}
                    {active && <span className="shell-model-default">LOADED</span>}
                  </div>
                  <p>{model.fileName} · {sizeLabel(model.sizeBytes)}</p>
                  <small>{model.architecture ?? "Unknown architecture"} · {model.quantization ?? "Unknown quantization"}</small>
                  <small>SHA-256 · {model.sha256}</small>
                  {estimate && (
                    <small className={estimate.fitsConfiguredBudget && estimate.fitsCurrentAvailability ? "" : "is-error"}>
                      Estimated peak: {sizeLabel(estimate.estimatedPeakBytes)} · 2K context · {estimate.fitsConfiguredBudget ? "within configured budget" : "over RAM budget"} · {estimate.fitsCurrentAvailability ? "available RAM check passed" : "not enough currently available RAM"}
                    </small>
                  )}
                </div>
                <div className="shell-model-actions">
                  {!model.isDefault && <button type="button" disabled={isAnyBusy} onClick={() => void makeDefault(model.id)}>Set default</button>}
                  <button type="button" disabled={isAnyBusy} onClick={() => void inspect(model.id)}>Check memory</button>
                  {!active && <button type="button" disabled={isAnyBusy || Boolean(runtime?.isLoaded)} onClick={() => void load(model.id)}>{loading || busyModel === model.id ? "Working…" : "Load"}</button>}
                  <button type="button" disabled={isAnyBusy || active} onClick={() => void remove(model)}>Remove</button>
                </div>
              </article>
            );
          })}
        </div>
      )}
      <p className="shell-settings-footnote">GGUF metadata is read from the file header. The RAM Limiter blocks loads whose conservative estimate exceeds your configured budget. Sync also checks currently available system RAM and a safety reserve. This is a managed inference guard, not an operating-system hard cap.</p>
    </section>
  );
}
