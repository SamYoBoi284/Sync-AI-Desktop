import type { AppInfo } from "../../services/nativeBridge";

type BridgeState =
  | { kind: "checking" }
  | { kind: "connected"; info: AppInfo }
  | { kind: "preview" };

interface RuntimePageProps {
  bridge: BridgeState;
}

export function RuntimePage({ bridge }: RuntimePageProps) {
  const values = [
    {
      label: "Application",
      value: bridge.kind === "connected" ? bridge.info.appName : "Sync AI Desktop",
      state: bridge.kind === "connected" ? "REPORTED BY RUST" : "STATIC LABEL",
    },
    {
      label: "Operating system",
      value: bridge.kind === "connected" ? bridge.info.operatingSystem : "Unavailable",
      state: bridge.kind === "connected" ? "REPORTED BY RUST" : "NOT AVAILABLE",
    },
    {
      label: "Architecture",
      value: bridge.kind === "connected" ? bridge.info.architecture : "Unavailable",
      state: bridge.kind === "connected" ? "REPORTED BY RUST" : "NOT AVAILABLE",
    },
    {
      label: "Application version",
      value: bridge.kind === "connected" ? bridge.info.version : "0.1.0",
      state: bridge.kind === "connected" ? "REPORTED BY RUST" : "FALLBACK VALUE",
    },
  ];

  return (
    <div className="feature-page runtime-page">
      <header className="page-intro">
        <p className="eyebrow-label">SYSTEM INSPECTION</p>
        <h1>Know what is running.</h1>
        <p>Runtime diagnostics will make the desktop process and its local services easier to inspect.</p>
      </header>

      <section className="runtime-connection-panel">
        <div className={"runtime-connection-mark runtime-connection-mark--" + bridge.kind}>
          <span />
        </div>
        <div>
          <p className="panel-kicker">NATIVE APPLICATION BRIDGE</p>
          <h2>
            {bridge.kind === "connected"
              ? "Rust bridge connected"
              : bridge.kind === "preview"
                ? "Browser preview"
                : "Checking connection"}
          </h2>
          <p>
            {bridge.kind === "connected"
              ? "The frontend successfully received application metadata from the Tauri backend."
              : bridge.kind === "preview"
                ? "This view is running without the native Tauri host. Native runtime data is not available."
                : "Waiting for the application metadata command to respond."}
          </p>
        </div>
        <span className={"connection-pill connection-pill--" + bridge.kind}>
          {bridge.kind === "connected" ? "CONNECTED" : bridge.kind === "preview" ? "PREVIEW" : "CHECKING"}
        </span>
      </section>

      <section className="runtime-data-panel">
        <div className="panel-heading">
          <div>
            <p className="panel-kicker">AVAILABLE METADATA</p>
            <h2>Application details</h2>
          </div>
          <span className="panel-index">READ ONLY</span>
        </div>
        <div className="runtime-data-grid">
          {values.map((item) => (
            <div className="runtime-data-cell" key={item.label}>
              <span>{item.label}</span>
              <strong>{item.value}</strong>
              <small>{item.state}</small>
            </div>
          ))}
        </div>
      </section>

      <section className="runtime-limit-note">
        <span className="note-symbol">i</span>
        <p>
          CPU, memory, model, and process diagnostics are not exposed yet. This page reports only the metadata currently returned by the Rust command.
        </p>
      </section>
    </div>
  );
}
