import { useEffect, useState } from "react";
import { getAppInfo, type AppInfo } from "./services/nativeBridge";
import { LoadingScreen } from "./components/startup/LoadingScreen";

type BridgeState =
  | { kind: "checking" }
  | { kind: "connected"; info: AppInfo }
  | { kind: "preview" };

const navigation = [
  { heading: "CONVERSATIONS", entries: ["New chat", "History"] },
  { heading: "WORKSPACE", entries: ["Workspace", "Tasks & Notes", "File Library"] },
  { heading: "SYSTEM", entries: ["Local models", "Appearance", "Runtime"] },
];

interface AppProps {
  showStartupSplash: boolean;
}

function App({ showStartupSplash }: AppProps) {
  const [bridge, setBridge] = useState<BridgeState>({ kind: "checking" });
  const [splashVisible, setSplashVisible] = useState(showStartupSplash);

  useEffect(() => {
    let active = true;

    const bridgeRequest = getAppInfo()
      .then((info) => {
        if (active) setBridge({ kind: "connected", info });
      })
      .catch(() => {
        // Browser preview has no Tauri IPC host. Do not pretend the Rust bridge is connected.
        if (active) setBridge({ kind: "preview" });
      });

    // Keep the branded splash visible briefly, but never wait for a local model to load.
    const minimumSplashTime = showStartupSplash
      ? new Promise<void>((resolve) => window.setTimeout(resolve, 850))
      : Promise.resolve();

    Promise.all([bridgeRequest, minimumSplashTime]).finally(() => {
      if (active) setSplashVisible(false);
    });

    return () => {
      active = false;
    };
  }, [showStartupSplash]);

  const version = bridge.kind === "connected" ? bridge.info.version : "0.1.0";

  return (
    <>
      <main className="app-shell" aria-hidden={splashVisible}>
      <aside className="sidebar" aria-label="Main navigation preview">
        <div className="brand-lockup">
          <div className="brand-mark" aria-hidden="true">
            <span />
            <span />
            <span />
          </div>
          <div>
            <p className="brand-name">SYNC AI</p>
            <p className="brand-caption">DESKTOP SYSTEM</p>
          </div>
        </div>

        <div className="sidebar-divider" />

        <div className="sidebar-content">
          {navigation.map((section) => (
            <section className="nav-section" key={section.heading}>
              <h2>{section.heading}</h2>
              <ul>
                {section.entries.map((entry) => (
                  <li className="nav-entry" aria-disabled="true" key={entry}>
                    <span className="nav-entry-indicator" aria-hidden="true" />
                    <span>{entry}</span>
                    <span className="nav-entry-status">PLANNED</span>
                  </li>
                ))}
              </ul>
            </section>
          ))}
        </div>

        <div className="sidebar-footer">
          <div className="local-seal">
            <span className="seal-dot" />
            <span>LOCAL-FIRST</span>
          </div>
          <p>Built around explicit user control.</p>
        </div>
      </aside>

      <section className="main-panel">
        <header className="topbar">
          <div className="breadcrumb">
            <span>SYNC AI</span>
            <span className="breadcrumb-slash">/</span>
            <strong>FOUNDATION</strong>
          </div>
          <div className="topbar-meta">
            <span className="version-label">BUILD {version}</span>
            <span className={"connection-pill connection-pill--" + bridge.kind} aria-live="polite">
              <span className="connection-dot" />
              {bridge.kind === "connected"
                ? "RUST BRIDGE CONNECTED"
                : bridge.kind === "preview"
                  ? "BROWSER PREVIEW"
                  : "CONNECTING"}
            </span>
          </div>
        </header>

        <div className="content-scroll">
          <div className="content-wrap">
            <div className="eyebrow">
              <span className="eyebrow-line" />
              WINDOWS-FIRST · OFFLINE-FIRST
            </div>

            <section className="hero">
              <div className="hero-copy">
                <p className="hero-kicker">A NEW DESKTOP INSTANCE</p>
                <h1>
                  Your space.
                  <br />
                  <span>Your signal.</span>
                </h1>
                <p className="hero-description">
                  Sync AI is being rebuilt for desktop with local inference,
                  familiar identity, and a strict boundary around your computer.
                </p>
              </div>

              <div className="core-stage" aria-label="Sync Core, ready">
                <div className="core-halo core-halo--outer" />
                <div className="core-halo core-halo--inner" />
                <div className="core-orbit">
                  <span className="orbit-node" />
                </div>
                <div className="core-orb">
                  <span className="core-highlight" />
                </div>
                <div className="core-caption">
                  <span className="core-caption-dot" />
                  CORE · READY
                </div>
              </div>
            </section>

            <section className="status-grid" aria-label="Foundation status">
              <article className="status-card status-card--primary">
                <div className="card-heading">
                  <span className="card-index">01</span>
                  <span className="card-tag">APPLICATION LAYER</span>
                </div>
                <h2>Desktop foundation</h2>
                <p>
                  Tauri 2, React, TypeScript and Vite are connected as the
                  initial application structure.
                </p>
                <div className="card-footer">
                  <span className="status-led status-led--active" />
                  {bridge.kind === "connected"
                    ? "NATIVE HANDSHAKE VERIFIED"
                    : bridge.kind === "preview"
                      ? "FRONTEND PREVIEW ONLY"
                      : "CHECKING NATIVE BRIDGE"}
                </div>
              </article>

              <article className="status-card">
                <div className="card-heading">
                  <span className="card-index">02</span>
                  <span className="card-tag">LOCAL INFERENCE</span>
                </div>
                <h2>Not connected yet</h2>
                <p>
                  GGUF model management and the Windows llama.cpp bridge come
                  in a later implementation phase.
                </p>
                <div className="card-footer">
                  <span className="status-led" />
                  INTEGRATION PENDING
                </div>
              </article>

              <article className="status-card">
                <div className="card-heading">
                  <span className="card-index">03</span>
                  <span className="card-tag">HOST PERMISSIONS</span>
                </div>
                <h2>Nothing has access</h2>
                <p>
                  No shell, process-control or general filesystem tools are
                  exposed to the frontend or model.
                </p>
                <div className="card-footer">
                  <span className="status-led status-led--safe" />
                  RESTRICTED BY DEFAULT
                </div>
              </article>
            </section>

            <section className="runtime-panel">
              <div className="runtime-heading">
                <div>
                  <p className="hero-kicker">RUNTIME INSPECTION</p>
                  <h2>Native bridge</h2>
                </div>
                <span className="runtime-badge">
                  {bridge.kind === "connected" ? "CONNECTED" : "NOT CONNECTED"}
                </span>
              </div>

              {bridge.kind === "connected" ? (
                <dl className="runtime-details">
                  <div>
                    <dt>Application</dt>
                    <dd>{bridge.info.appName}</dd>
                  </div>
                  <div>
                    <dt>Operating system</dt>
                    <dd>{bridge.info.operatingSystem}</dd>
                  </div>
                  <div>
                    <dt>Architecture</dt>
                    <dd>{bridge.info.architecture}</dd>
                  </div>
                  <div>
                    <dt>Version</dt>
                    <dd>{bridge.info.version}</dd>
                  </div>
                </dl>
              ) : (
                <p className="runtime-note">
                  {bridge.kind === "preview"
                    ? "This page is running without the Tauri host. Launch the desktop app with the Tauri development command to verify the Rust bridge."
                    : "Waiting for the Rust application layer to respond."}
                </p>
              )}
            </section>

            <footer className="page-footer">
              <span>SYNC AI DESKTOP · FOUNDATION BUILD</span>
              <span>CHAT · MODELS · STORAGE · TOOLS ARE NOT IMPLEMENTED</span>
            </footer>
          </div>
        </div>
      </section>
      </main>
      {splashVisible && <LoadingScreen />}
    </>
  );
}

export default App;
