import { useEffect, useState } from "react";
import { LoadingScreen } from "./components/startup/LoadingScreen";
import { Sidebar, type PageId } from "./components/navigation/Sidebar";
import { ChatPage } from "./components/pages/ChatPage";
import { FeaturePage } from "./components/pages/FeaturePage";
import { RuntimePage } from "./components/pages/RuntimePage";
import { getAppInfo, type AppInfo } from "./services/nativeBridge";

type BridgeState =
  | { kind: "checking" }
  | { kind: "connected"; info: AppInfo }
  | { kind: "preview" };

const pageTitles: Record<PageId, { section: string; title: string }> = {
  chat: { section: "CONVERSATIONS", title: "New chat" },
  history: { section: "CONVERSATIONS", title: "History" },
  workspace: { section: "WORKSPACE", title: "Workspace" },
  tasks: { section: "WORKSPACE", title: "Tasks & Notes" },
  files: { section: "WORKSPACE", title: "File Library" },
  models: { section: "SYSTEM", title: "Local models" },
  appearance: { section: "SYSTEM", title: "Appearance" },
  runtime: { section: "SYSTEM", title: "Runtime" },
};

interface AppProps {
  showStartupSplash: boolean;
}

function App({ showStartupSplash }: AppProps) {
  const [bridge, setBridge] = useState<BridgeState>({ kind: "checking" });
  const [activePage, setActivePage] = useState<PageId>("chat");
  const [splashVisible, setSplashVisible] = useState(showStartupSplash);

  useEffect(() => {
    let active = true;

    const bridgeRequest = getAppInfo()
      .then((info) => {
        if (active) setBridge({ kind: "connected", info });
      })
      .catch(() => {
        if (active) setBridge({ kind: "preview" });
      });

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

  const appVersion = bridge.kind === "connected" ? bridge.info.version : "0.1.0";
  const page = pageTitles[activePage];

  function renderPage() {
    if (activePage === "chat") {
      return <ChatPage onNavigate={setActivePage} />;
    }

    if (activePage === "runtime") {
      return <RuntimePage bridge={bridge} />;
    }

    return <FeaturePage page={activePage} />;
  }

  return (
    <>
      <div className="app-frame" aria-hidden={splashVisible}>
        <Sidebar
          activePage={activePage}
          onNavigate={setActivePage}
          version={appVersion}
        />

        <div className="app-main">
          <header className="topbar">
            <div className="topbar-title">
              <div className="breadcrumb">
                <span>SYNC AI</span>
                <span className="breadcrumb-separator">/</span>
                <span>{page.section}</span>
              </div>
              <h1>{page.title}</h1>
            </div>

            <div className="topbar-status">
              <span className="build-label">BUILD {appVersion}</span>
              <span className={"connection-pill connection-pill--" + bridge.kind}>
                <span className="connection-dot" />
                {bridge.kind === "connected"
                  ? "RUST BRIDGE"
                  : bridge.kind === "preview"
                    ? "BROWSER PREVIEW"
                    : "CONNECTING"}
              </span>
            </div>
          </header>

          <main className={"page-scroll page-scroll--" + activePage}>
            {renderPage()}
          </main>
        </div>
      </div>

      {splashVisible && <LoadingScreen />}
    </>
  );
}

export default App;
