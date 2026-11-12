import { useEffect, useState } from "react";
import type { AppInfo, RuntimeStatus } from "../services/nativeBridge";
import { getRuntimeStatus } from "../services/nativeBridge";
import type { BridgeKind } from "../components/layout/Topbar";

interface RuntimePageProps { bridgeKind: BridgeKind; info: AppInfo | null; }
const bytesLabel = (bytes: number | null | undefined) => bytes == null
  ? "—"
  : bytes >= 1024 ** 3
    ? (bytes / 1024 ** 3).toFixed(2) + " GB"
    : (bytes / 1024 ** 2).toFixed(0) + " MB";

export function RuntimePage({ bridgeKind, info }: RuntimePageProps) {
  const connected = bridgeKind === "connected";
  const [runtime, setRuntime] = useState<RuntimeStatus | null>(null);
  const [runtimeError, setRuntimeError] = useState("");

  useEffect(() => {
    let active = true;
    const refresh = () => getRuntimeStatus().then(value => {
      if (active) { setRuntime(value); setRuntimeError(""); }
    }).catch(error => { if (active) setRuntimeError(String(error)); });
    void refresh();
    const handle = window.setInterval(refresh, 1500);
    return () => { active = false; window.clearInterval(handle); };
  }, []);

  return (
    <div className="shell-page shell-standard-page">
      <div className="shell-page-eyebrow"><span className="shell-eyebrow-line" />SYSTEM INSPECTION</div>
      <div className="shell-page-heading-row">
        <div><p className="shell-section-kicker">APPLICATION STATUS</p><h1>Runtime <span>diagnostics.</span></h1><p className="shell-page-description">The desktop host and isolated inference-worker resource usage.</p></div>
        <span className={"shell-large-status" + (connected ? " is-connected" : "")}><span />{connected ? "NATIVE HOST" : bridgeKind === "preview" ? "BROWSER PREVIEW" : "CONNECTING"}</span>
      </div>

      <section className="shell-runtime-grid" aria-label="Runtime components">
        <article className="shell-runtime-card shell-runtime-card--accent">
          <span className="shell-card-index">01 / APPLICATION</span><h2>{info?.appName ?? "Sync AI Desktop"}</h2>
          <p>The Tauri process and React interface stay outside the inference worker's memory ceiling.</p>
          <span className="shell-card-foot"><i className={connected ? "is-on" : ""} />{connected ? "HOST CONNECTED" : "WAITING FOR NATIVE HOST"}</span>
        </article>
        <article className="shell-runtime-card">
          <span className="shell-card-index">02 / INFERENCE WORKER</span><h2>{runtime?.isLoaded ? runtime.loadedModelName : runtime?.isLoading ? "Loading model…" : "No model loaded"}</h2>
          <p>{runtime?.backend ?? "Waiting for the native inference manager."}</p>
          <span className="shell-card-foot"><i className={runtime?.isLoaded ? "is-on" : ""} />{runtime?.isLoaded ? "WORKER ACTIVE" : runtime?.isLoading ? "STARTING" : "IDLE"}</span>
        </article>
        <article className="shell-runtime-card">
          <span className="shell-card-index">03 / MEMORY ENFORCEMENT</span><h2>{runtime?.memoryLimitBytes ? bytesLabel(runtime.memoryLimitBytes) : "No hard ceiling"}</h2>
          <p>{runtime?.memoryLimitBytes ? "Windows Job Object process committed-memory limit." : "The worker has no configured hard ceiling at present."}</p>
          <span className="shell-card-foot"><i className={runtime?.hardLimitSupported ? "is-safe" : ""} />{runtime?.hardLimitSupported ? "WINDOWS JOB OBJECT SUPPORT" : "HARD LIMIT UNAVAILABLE"}</span>
        </article>
      </section>

      <section className="shell-runtime-details">
        <div className="shell-panel-heading"><div><p className="shell-section-kicker">WORKER RESOURCE TELEMETRY</p><h2>Memory diagnostics</h2></div><span className="shell-panel-tag">REFRESHES EVERY 1.5 SEC</span></div>
        {runtimeError ? <p className="shell-details-note">{runtimeError}</p> : runtime ? <dl className="shell-details-grid">
          <div><dt>Configured hard ceiling</dt><dd>{bytesLabel(runtime.memoryLimitBytes)}</dd></div>
          <div><dt>Worker resident working set</dt><dd>{bytesLabel(runtime.workerWorkingSetBytes)}</dd></div>
          <div><dt>Worker private committed memory</dt><dd>{bytesLabel(runtime.workerPrivateBytes)}</dd></div>
          <div><dt>Worker process ID</dt><dd>{runtime.workerPid ?? "Not running"}</dd></div>
          <div><dt>Loaded model</dt><dd>{runtime.loadedModelName ?? "None"}</dd></div>
          <div><dt>Context / batch</dt><dd>{runtime.contextLength ? runtime.contextLength.toLocaleString() + " / " + runtime.batchSize : "—"}</dd></div>
        </dl> : <p className="shell-details-note">Waiting for worker telemetry.</p>}
        <p className="shell-details-note">The Job Object ceiling applies to committed private memory. Resident working set measures pages currently resident in RAM; it is not the enforcement counter. GGUF weights are loaded without file-backed memory mapping so their model allocations are inside the worker's private-memory accounting.</p>
      </section>

      {runtime?.lastError && <section className="shell-runtime-details" aria-live="polite">
        <div className="shell-panel-heading"><div><p className="shell-section-kicker">LAST WORKER ERROR</p><h2>{runtime.lastErrorCode ?? "Runtime error"}</h2></div><span className="shell-panel-tag">DIAGNOSTIC</span></div>
        <p className="shell-model-message is-error" role="alert">{runtime.lastError}</p>
      </section>}

      <section className="shell-runtime-details">
        <div className="shell-panel-heading"><div><p className="shell-section-kicker">NATIVE BRIDGE</p><h2>Host details</h2></div><span className="shell-panel-tag">READ-ONLY METADATA</span></div>
        {connected && info ? <dl className="shell-details-grid">
          <div><dt>Application</dt><dd>{info.appName}</dd></div><div><dt>Version</dt><dd>{info.version}</dd></div>
          <div><dt>Operating system</dt><dd>{info.operatingSystem}</dd></div><div><dt>Architecture</dt><dd>{info.architecture}</dd></div>
        </dl> : <p className="shell-details-note">{bridgeKind === "preview" ? "This is the browser preview. Launch the Tauri desktop app to read native host metadata." : "Waiting for the Rust application layer to respond."}</p>}
      </section>
    </div>
  );
}
