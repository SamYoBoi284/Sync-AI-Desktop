import { useEffect, useState } from "react";
import { getHostRamInfo, getSetting, setSetting, type HostRamInfo } from "../services/nativeBridge";
import { type PageId } from "../types/navigation";
interface RamLimiterConfig { enabled: boolean; limitGb: number; }
const KEY = "ram_limiter";
const toGb=(bytes:number)=>bytes/(1024**3);
const suggested=(info:HostRamInfo)=>Math.max(1,Math.floor(Math.max(1,toGb(info.totalBytes)-3)));
export function SettingsPage({onNavigate}:{onNavigate:(page:PageId)=>void}) {
 const [ram,setRam]=useState<HostRamInfo|null>(null),[config,setConfig]=useState<RamLimiterConfig>({enabled:false,limitGb:4}),[loading,setLoading]=useState(true),[saving,setSaving]=useState(false),[message,setMessage]=useState("");
 useEffect(()=>{let active=true;Promise.all([getHostRamInfo(),getSetting(KEY)]).then(([info,raw])=>{if(!active)return;setRam(info);let parsed:RamLimiterConfig|undefined;try{parsed=raw?JSON.parse(raw):undefined;}catch{}const max=Math.max(1,Math.floor(toGb(info.totalBytes)));setConfig({enabled:parsed?.enabled===true,limitGb:Math.min(max,Math.max(1,parsed?.limitGb??suggested(info)))});}).catch(()=>setMessage("Hardware information is available only in the desktop app.")).finally(()=>{if(active)setLoading(false);});return()=>{active=false;};},[]);
 const persist=async(next:RamLimiterConfig)=>{setConfig(next);setSaving(true);setMessage("");try{await setSetting(KEY,JSON.stringify(next));setMessage("Saved locally.");}catch{setMessage("Could not save this setting in browser preview.");}finally{setSaving(false);}};
 const maxGb=ram?Math.max(1,Math.floor(toGb(ram.totalBytes)-2)):1;
 return <section className="shell-page shell-standard-page">
  <div className="shell-page-eyebrow"><span className="shell-eyebrow-line"/>PREFERENCES AND CONTROL</div>
  <div className="shell-page-heading-row"><div><p className="shell-section-kicker">SYNC AI CONFIGURATION</p><h1>Settings<span>.</span></h1><p className="shell-page-description">Configure local resource preferences and desktop behavior.</p></div><span className="shell-large-status"><span/>SETTINGS FOUNDATION</span></div>
  <div className="shell-settings-grid">
   <button className="shell-settings-card" type="button" onClick={()=>onNavigate("appearance")}><span className="shell-settings-card-icon">◐</span><span><strong>Appearance</strong><small>Theme, accent color and motion preferences</small></span><span className="shell-settings-card-arrow">›</span></button>
   <button className="shell-settings-card" type="button" onClick={()=>onNavigate("runtime")}><span className="shell-settings-card-icon">⌁</span><span><strong>Runtime</strong><small>Application and native bridge information</small></span><span className="shell-settings-card-arrow">›</span></button>
  </div>
  <section className="shell-settings-panel">
   <div className="shell-panel-heading"><div><p className="shell-section-kicker">RESOURCE MANAGEMENT</p><h2>RAM Limiter</h2></div><span className="shell-panel-tag">OPTIONAL · OFF BY DEFAULT</span></div>
   <p className="shell-settings-intro">Sync detects your machine’s RAM automatically. Enable the limiter to set a managed memory budget for local inference.</p>
   {loading?<p className="shell-settings-footnote">Detecting system memory…</p>:ram?<>
    <div className="shell-ram-stats"><div><small>INSTALLED RAM</small><strong>{toGb(ram.totalBytes).toFixed(1)} GB</strong></div><div><small>CURRENTLY AVAILABLE</small><strong>{toGb(ram.availableBytes).toFixed(1)} GB</strong></div><div><small>MAX INFERENCE BUDGET</small><strong>{maxGb} GB</strong></div></div>
    <label className="shell-ram-toggle"><span><strong>Enable RAM Limiter</strong><small>Block a model load when its estimated inference memory exceeds this budget.</small></span><input type="checkbox" checked={config.enabled} disabled={saving} onChange={e=>void persist({...config,enabled:e.target.checked})}/></label>
    <div className={"shell-ram-slider"+(!config.enabled?" is-disabled":"")}><div className="shell-ram-slider-heading"><label htmlFor="ram-limit">Inference memory budget</label><strong>{config.limitGb} GB</strong></div><input id="ram-limit" type="range" min="1" max={maxGb} step="1" value={Math.min(config.limitGb,maxGb)} disabled={!config.enabled||saving} onChange={e=>setConfig({...config,limitGb:Number(e.target.value)})} onMouseUp={()=>void persist(config)} onTouchEnd={()=>void persist(config)} onKeyUp={()=>void persist(config)}/><div className="shell-ram-slider-scale"><span>1 GB</span><span>{maxGb} GB</span></div><small>Leaves room for Windows and other applications. This is an application-level pre-load guard, not an OS-enforced process cap.</small></div>
   </>:<p className="shell-settings-footnote">RAM detection requires the native desktop host.</p>}
   {message&&<p className="shell-settings-feedback" role="status">{message}</p>}
  </section>
  <div className="shell-settings-grid shell-settings-secondary"><div className="shell-settings-card is-staged"><span className="shell-settings-card-icon">◈</span><span><strong>Model and memory</strong><small>GGUF registry and model lifecycle are being added in Phase 4.</small></span><span className="shell-settings-card-state">IN PROGRESS</span></div><div className="shell-settings-card is-staged"><span className="shell-settings-card-icon">⌑</span><span><strong>Privacy and permissions</strong><small>Explicit approval controls arrive with the secure tool system.</small></span><span className="shell-settings-card-state">PLANNED</span></div></div>
  <p className="shell-settings-footnote">The selected budget is saved in local SQLite and checked when a model is loaded. Lowering the budget does not retroactively unload an active model.</p>
 </section>;
}
