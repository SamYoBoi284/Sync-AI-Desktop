import { type PageId } from "../types/navigation";
export function SettingsPage({onNavigate}:{onNavigate:(page:PageId)=>void}) {
 return <section className="shell-page shell-standard-page">
  <div className="shell-page-eyebrow"><span className="shell-eyebrow-line"/>PREFERENCES AND CONTROL</div>
  <div className="shell-page-heading-row"><div><p className="shell-section-kicker">SYNC AI CONFIGURATION</p><h1>Settings<span>.</span></h1><p className="shell-page-description">A central place for Sync’s appearance, local runtime and privacy controls.</p></div><span className="shell-large-status"><span/>SETTINGS FOUNDATION</span></div>
  <div className="shell-settings-grid">
   <button className="shell-settings-card" type="button" onClick={()=>onNavigate("appearance")}><span className="shell-settings-card-icon">◐</span><span><strong>Appearance</strong><small>Theme, accent color and motion preferences</small></span><span className="shell-settings-card-arrow">›</span></button>
   <button className="shell-settings-card" type="button" onClick={()=>onNavigate("runtime")}><span className="shell-settings-card-icon">⌁</span><span><strong>Runtime</strong><small>Application and native bridge information</small></span><span className="shell-settings-card-arrow">›</span></button>
   <div className="shell-settings-card is-staged"><span className="shell-settings-card-icon">◈</span><span><strong>Model and memory</strong><small>Model selection and optional RAM budget arrive with Phase 4</small></span><span className="shell-settings-card-state">PLANNED</span></div>
   <div className="shell-settings-card is-staged"><span className="shell-settings-card-icon">⌑</span><span><strong>Privacy and permissions</strong><small>Explicit approval controls arrive with the secure tool system</small></span><span className="shell-settings-card-state">PLANNED</span></div>
  </div><p className="shell-settings-footnote">Only destinations backed by implemented controls are interactive; planned controls are labelled rather than simulated.</p>
 </section>;
}
