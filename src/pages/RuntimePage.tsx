import type { AppInfo } from "../services/nativeBridge";
import type { BridgeKind } from "../components/layout/Topbar";

interface RuntimePageProps {
  bridgeKind: BridgeKind;
  info: AppInfo | null;
}

export function RuntimePage({ bridgeKind, info }: RuntimePageProps) {
  const connected = bridgeKind === "connected";

  return (
    <div className="shell-page shell-standard-page">
      <div className="shell-page-eyebrow"><span className="shell-eyebrow-line" />SYSTEM INSPECTION</div>
      <div className="shell-page-heading-row">
        <div>
          <p className="shell-section-kicker">APPLICATION STATUS</p>
          <h1>Runtime <span>overview.</span></h1>
          <p className="shell-page-description">A view of the desktop host and the services currently connected to it.</p>
        </div>
        <span className={`shell-large-status${connected ? " is-connected" : ""}`}>
          <span />{connected ? "NATIVE HOST" : bridgeKind === "preview" ? "BROWSER PREVIEW" : "CONNECTING"}
        </span>
      </div>

      <section className="shell-runtime-grid" aria-label="Runtime components">
        <article className="shell-runtime-card shell-runtime-card--accent">
          <span className="shell-card-index">01 / APPLICATION</span>
          <h2>{info?.appName ?? "Sync AI Desktop"}</h2>
          <p>The Tauri process and React interface are connected through the typed command bridge.</p>
          <span className="shell-card-foot"><i className={connected ? "is-on" : ""} />{connected ? "HANDSHAKE COMPLETE" : "WAITING FOR NATIVE HOST"}</span>
        </article>
        <article className="shell-runtime-card">
          <span className="shell-card-index">02 / INFERENCE</span>
          <h2>Not connected</h2>
          <p>No GGUF model has been loaded. The Windows llama.cpp backend is not part of this build yet.</p>
          <span className="shell-card-foot"><i />INTEGRATION PENDING</span>
        </article>
        <article className="shell-runtime-card">
          <span className="shell-card-index">03 / PERMISSIONS</span>
          <h2>Restricted by default</h2>
          <p>No shell, filesystem or process-control plugin is exposed to the frontend or model.</p>
          <span className="shell-card-foot"><i className="is-safe" />NO HOST TOOLS EXPOSED</span>
        </article>
      </section>

      <section className="shell-runtime-details">
        <div className="shell-panel-heading">
          <div><p className="shell-section-kicker">NATIVE BRIDGE</p><h2>Host details</h2></div>
          <span className="shell-panel-tag">READ-ONLY METADATA</span>
        </div>
        {connected && info ? (
          <dl className="shell-details-grid">
            <div><dt>Application</dt><dd>{info.appName}</dd></div>
            <div><dt>Version</dt><dd>{info.version}</dd></div>
            <div><dt>Operating system</dt><dd>{info.operatingSystem}</dd></div>
            <div><dt>Architecture</dt><dd>{info.architecture}</dd></div>
          </dl>
        ) : (
          <p className="shell-details-note">
            {bridgeKind === "preview"
              ? "This is the browser preview. Launch the Tauri desktop app to read native host metadata."
              : "Waiting for the Rust application layer to respond."}
          </p>
        )}
      </section>
    </div>
  );
}
