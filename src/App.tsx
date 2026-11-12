import { useEffect, useState } from "react";
import { Sidebar } from "./components/layout/Sidebar";
import { Topbar, type BridgeKind } from "./components/layout/Topbar";
import { FeaturePreviewPage } from "./pages/FeaturePreviewPage";
import { NewChatPage } from "./pages/NewChatPage";
import { RuntimePage } from "./pages/RuntimePage";
import { HistoryPage } from "./pages/HistoryPage";
import { SettingsPage } from "./pages/SettingsPage";
import { LocalModelsPage } from "./pages/LocalModelsPage";
import { getAppInfo, type AppInfo } from "./services/nativeBridge";
import type { PageId } from "./types/navigation";
import { LoadingScreen } from "./components/startup/LoadingScreen";

interface AppProps {
  showStartupSplash: boolean;
}

function App({ showStartupSplash }: AppProps) {
  const [activePage, setActivePage] = useState<PageId>("new-chat");
  const [selectedChatId, setSelectedChatId] = useState<string | null>(null);
  const navigateToPage = (page: PageId) => { if (page === "history" || page === "new-chat") setSelectedChatId(null); setActivePage(page); };
  const openChatFromSidebar = (chatId: string) => { setSelectedChatId(chatId); setActivePage("history"); };
  const [bridgeKind, setBridgeKind] = useState<BridgeKind>("checking");
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [splashVisible, setSplashVisible] = useState(showStartupSplash);

  useEffect(() => {
    let active = true;

    const bridgeRequest = getAppInfo()
      .then((info) => {
        if (active) {
          setAppInfo(info);
          setBridgeKind("connected");
        }
      })
      .catch(() => {
        // Browser preview has no Tauri IPC host. Do not imply that the native bridge is connected.
        if (active) setBridgeKind("preview");
      });

    // A splash is process-startup presentation only. It never waits for model initialization.
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

  useEffect(() => {
    const handleShortcut = (event: KeyboardEvent) => {
      const isNewChatShortcut = (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "n";
      if (isNewChatShortcut) {
        event.preventDefault();
        navigateToPage("new-chat");
      }
    };

    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, []);

  const version = appInfo?.version ?? "0.1.0";

  return (
    <>
      <div className="shell-root" aria-hidden={splashVisible}>
        <Sidebar activePage={activePage} onNavigate={navigateToPage} onOpenChat={openChatFromSidebar} />
        <main className="shell-main">
          <Topbar activePage={activePage} bridgeKind={bridgeKind} version={version} />
          <div className="shell-page-scroll">
            {activePage === "new-chat" ? (
              <NewChatPage />
            ) : activePage === "history" ? (
              <HistoryPage key={selectedChatId ?? "history"} initialChatId={selectedChatId} />
            ) : activePage === "settings" ? (
              <SettingsPage onNavigate={navigateToPage} />
            ) : activePage === "models" ? (
              <LocalModelsPage />
            ) : activePage === "runtime" ? (
              <RuntimePage bridgeKind={bridgeKind} info={appInfo} />
            ) : (
              <FeaturePreviewPage page={activePage} />
            )}
            <footer className="shell-page-footer">
              <span>SYNC AI DESKTOP · WINDOWS FOUNDATION</span>
              <span>CHAT INFERENCE NOT CONNECTED</span>
            </footer>
          </div>
        </main>
      </div>
      {splashVisible && <LoadingScreen />}
    </>
  );
}

export default App;
