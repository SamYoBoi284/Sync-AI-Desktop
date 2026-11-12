import type { ReactNode } from "react";

export type PageId =
  | "chat"
  | "history"
  | "workspace"
  | "tasks"
  | "files"
  | "models"
  | "appearance"
  | "runtime";

type IconName =
  | "plus"
  | "clock"
  | "layers"
  | "check"
  | "file"
  | "cpu"
  | "sliders"
  | "activity";

interface NavigationItem {
  id: PageId;
  label: string;
  icon: IconName;
}

interface NavigationSection {
  label: string;
  items: NavigationItem[];
}

const primaryItems: NavigationItem[] = [
  { id: "chat", label: "New chat", icon: "plus" },
  { id: "history", label: "History", icon: "clock" },
];

const navigationSections: NavigationSection[] = [
  {
    label: "WORKSPACE",
    items: [
      { id: "workspace", label: "Workspace", icon: "layers" },
      { id: "tasks", label: "Tasks & Notes", icon: "check" },
      { id: "files", label: "File Library", icon: "file" },
    ],
  },
  {
    label: "SYSTEM",
    items: [
      { id: "models", label: "Local models", icon: "cpu" },
      { id: "appearance", label: "Appearance", icon: "sliders" },
      { id: "runtime", label: "Runtime", icon: "activity" },
    ],
  },
];

const iconArtwork: Record<IconName, ReactNode> = {
  plus: <path d="M12 5v14M5 12h14" />,
  clock: <><circle cx="12" cy="12" r="8.5" /><path d="M12 7v5l3 2" /></>,
  layers: <><path d="m12 3 9 5-9 5-9-5 9-5Z" /><path d="m3 12 9 5 9-5M3 16l9 5 9-5" /></>,
  check: <><rect x="4" y="4" width="16" height="16" rx="3" /><path d="m8 12 2.5 2.5L16 9" /></>,
  file: <><path d="M6 3.5h8l4 4V20a.5.5 0 0 1-.5.5h-11A.5.5 0 0 1 6 20V3.5Z" /><path d="M14 3.5V8h4M9 12h6M9 16h6" /></>,
  cpu: <><rect x="6" y="6" width="12" height="12" rx="2" /><path d="M9 9h6v6H9zM9 1.5v4M15 1.5v4M9 18.5v4M15 18.5v4M1.5 9h4M1.5 15h4M18.5 9h4M18.5 15h4" /></>,
  sliders: <><path d="M4 6h9M17 6h3M4 12h3M11 12h9M4 18h9M17 18h3" /><circle cx="15" cy="6" r="2" /><circle cx="9" cy="12" r="2" /><circle cx="15" cy="18" r="2" /></>,
  activity: <path d="M3 12h3l2.2-6 4.1 12 2.5-7 1.5 3H21" />,
};

function NavIcon({ name }: { name: IconName }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" className="nav-icon">
      {iconArtwork[name]}
    </svg>
  );
}

interface SidebarProps {
  activePage: PageId;
  onNavigate: (page: PageId) => void;
  version: string;
}

export function Sidebar({ activePage, onNavigate, version }: SidebarProps) {
  const renderItem = (item: NavigationItem) => {
    const active = activePage === item.id;
    return (
      <button
        className={"nav-item" + (active ? " nav-item--active" : "")}
        type="button"
        key={item.id}
        onClick={() => onNavigate(item.id)}
        aria-current={active ? "page" : undefined}
        title={item.label}
      >
        <span className="nav-item-icon">
          <NavIcon name={item.icon} />
        </span>
        <span className="nav-item-label">{item.label}</span>
      </button>
    );
  };

  return (
    <aside className="sidebar" aria-label="Main navigation">
      <button className="brand-lockup" type="button" onClick={() => onNavigate("chat")}>
        <span className="brand-mark" aria-hidden="true">
          <span />
          <span />
          <span />
        </span>
        <span className="brand-copy">
          <span className="brand-name">SYNC AI</span>
          <span className="brand-caption">DESKTOP SYSTEM</span>
        </span>
      </button>

      <div className="sidebar-rule" />

      <nav className="sidebar-navigation">
        <div className="nav-group nav-group--primary">
          {primaryItems.map(renderItem)}
        </div>

        {navigationSections.map((section) => (
          <div className="nav-group" key={section.label}>
            <p className="nav-group-label">{section.label}</p>
            {section.items.map(renderItem)}
          </div>
        ))}
      </nav>

      <div className="sidebar-bottom">
        <div className="local-status-card">
          <span className="local-status-mark"><span /></span>
          <span className="local-status-copy">
            <strong>LOCAL-FIRST</strong>
            <small>NO MODEL LOADED</small>
          </span>
        </div>
        <div className="sidebar-version">
          <span>SYNC AI DESKTOP</span>
          <span>BUILD {version}</span>
        </div>
      </div>
    </aside>
  );
}
