import { pageTitles, type PageId } from "../../types/navigation";

export type BridgeKind = "checking" | "connected" | "preview";

interface TopbarProps {
  activePage: PageId;
  bridgeKind: BridgeKind;
  version: string;
}

export function Topbar({ activePage, bridgeKind, version }: TopbarProps) {
  const bridgeLabel =
    bridgeKind === "connected"
      ? "RUST BRIDGE CONNECTED"
      : bridgeKind === "preview"
        ? "BROWSER PREVIEW"
        : "CONNECTING";

  return (
    <header className="shell-topbar">
      <div className="shell-breadcrumb">
        <span>SYNC AI</span>
        <span className="shell-breadcrumb-slash">/</span>
        <strong>{pageTitles[activePage].toUpperCase()}</strong>
      </div>
      <div className="shell-topbar-meta">
        <span className="shell-version-label">BUILD {version}</span>
        <span className={`shell-connection-pill is-${bridgeKind}`} aria-live="polite">
          <span className="shell-connection-dot" />
          {bridgeLabel}
        </span>
      </div>
    </header>
  );
}
