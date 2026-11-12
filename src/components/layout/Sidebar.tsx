import { navigationSections, type PageId } from "../../types/navigation";

interface SidebarProps {
  activePage: PageId;
  onNavigate: (page: PageId) => void;
}

export function Sidebar({ activePage, onNavigate }: SidebarProps) {
  return (
    <aside className="shell-sidebar" aria-label="Main navigation">
      <div className="shell-brand-lockup">
        <div className="shell-brand-mark" aria-hidden="true">
          <span />
          <span />
          <span />
        </div>
        <div>
          <p className="shell-brand-name">SYNC AI</p>
          <p className="shell-brand-caption">DESKTOP SYSTEM</p>
        </div>
      </div>

      <button
        className="shell-new-chat"
        type="button"
        onClick={() => onNavigate("new-chat")}
        aria-label="Start a new chat"
      >
        <span aria-hidden="true">+</span>
        <span>New conversation</span>
        <kbd>CTRL N</kbd>
      </button>

      <div className="shell-sidebar-content">
        {navigationSections.map((section) => (
          <section className="shell-nav-section" key={section.heading}>
            <h2>{section.heading}</h2>
            <ul>
              {section.entries.map((entry) => {
                const active = activePage === entry.id;
                return (
                  <li key={entry.id}>
                    <button
                      className={`shell-nav-entry${active ? " is-active" : ""}`}
                      type="button"
                      onClick={() => onNavigate(entry.id)}
                      aria-current={active ? "page" : undefined}
                    >
                      <span className="shell-nav-glyph" aria-hidden="true">{entry.glyph}</span>
                      <span className="shell-nav-label">{entry.label}</span>
                      {entry.availability === "planned" && <span className="shell-nav-status">SOON</span>}
                    </button>
                  </li>
                );
              })}
            </ul>
          </section>
        ))}
      </div>

      <div className="shell-sidebar-footer">
        <div className="shell-local-seal">
          <span className="shell-seal-dot" />
          <span>LOCAL-FIRST</span>
        </div>
        <p>Private by default. Explicit by design.</p>
        <span className="shell-build-label">DESKTOP FOUNDATION · 0.1.0</span>
      </div>
    </aside>
  );
}
