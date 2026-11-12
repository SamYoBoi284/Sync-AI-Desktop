import { useEffect, useState } from "react";
import { getHostRamInfo, getSetting, setSetting, type HostRamInfo } from "../services/nativeBridge";
import { type PageId } from "../types/navigation";

interface RamLimiterConfig { enabled: boolean; limitGb: number; }
const KEY = "ram_limiter";
const toGb = (bytes: number) => bytes / (1024 ** 3);
const suggested = (info: HostRamInfo) => Math.max(1, Math.floor(Math.max(1, toGb(info.totalBytes) - 3)));

export function SettingsPage({ onNavigate }: { onNavigate: (page: PageId) => void }) {
  const [ram, setRam] = useState<HostRamInfo | null>(null);
  const [config, setConfig] = useState<RamLimiterConfig>({ enabled: false, limitGb: 4 });
  const [savedConfig, setSavedConfig] = useState<RamLimiterConfig>({ enabled: false, limitGb: 4 });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    let active = true;
    Promise.all([getHostRamInfo(), getSetting(KEY)])
      .then(([info, raw]) => {
        if (!active) return;
        setRam(info);
        let parsed: RamLimiterConfig | undefined;
        try { parsed = raw ? JSON.parse(raw) : undefined; } catch { /* retain safe defaults */ }
        const max = Math.max(1, Math.floor(toGb(info.totalBytes) - 2));
        const initial = {
          enabled: parsed?.enabled === true,
          limitGb: Math.min(max, Math.max(1, parsed?.limitGb ?? suggested(info))),
        };
        setConfig(initial);
        setSavedConfig(initial);
      })
      .catch(() => setMessage("Hardware information is available only in the desktop app."))
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, []);

  const persist = async () => {
    setSaving(true);
    setMessage("");
    try {
      await setSetting(KEY, JSON.stringify(config));
      setSavedConfig(config);
      setMessage("Saved. The new ceiling applies the next time a model is loaded.");
    } catch {
      setMessage("Could not save this setting in browser preview.");
    } finally {
      setSaving(false);
    }
  };

  const maxGb = ram ? Math.max(1, Math.floor(toGb(ram.totalBytes) - 2)) : 1;
  const dirty = config.enabled !== savedConfig.enabled || config.limitGb !== savedConfig.limitGb;

  return (
    <section className="shell-page shell-standard-page">
      <div className="shell-page-eyebrow"><span className="shell-eyebrow-line" />PREFERENCES AND CONTROL</div>
      <div className="shell-page-heading-row">
        <div>
          <p className="shell-section-kicker">SYNC AI CONFIGURATION</p>
          <h1>Settings<span>.</span></h1>
          <p className="shell-page-description">Configure local resource preferences and desktop behavior.</p>
        </div>
        <span className="shell-large-status"><span />SETTINGS FOUNDATION</span>
      </div>

      <div className="shell-settings-grid">
        <button className="shell-settings-card" type="button" onClick={() => onNavigate("appearance")}><span className="shell-settings-card-icon">◐</span><span><strong>Appearance</strong><small>Theme, accent color and motion preferences</small></span><span className="shell-settings-card-arrow">›</span></button>
        <button className="shell-settings-card" type="button" onClick={() => onNavigate("runtime")}><span className="shell-settings-card-icon">⌁</span><span><strong>Runtime</strong><small>Worker memory, native diagnostics and host information</small></span><span className="shell-settings-card-arrow">›</span></button>
      </div>

      <section className="shell-settings-panel">
        <div className="shell-panel-heading">
          <div><p className="shell-section-kicker">INFERENCE RESOURCE CONTROL</p><h2>RAM Limiter</h2></div>
          <span className="shell-panel-tag">OPTIONAL · OFF BY DEFAULT</span>
        </div>
        <p className="shell-settings-intro">
          Places the isolated llama.cpp worker inside a Windows Job Object with a hard private-memory commit ceiling. The limit includes model allocations and inference context, so a model that needs more than the ceiling may fail to load. The main Sync AI desktop process stays outside that limit.
        </p>
        {loading ? <p className="shell-settings-footnote">Detecting system memory…</p> : ram ? <>
          <div className="shell-ram-stats">
            <div><small>INSTALLED RAM</small><strong>{toGb(ram.totalBytes).toFixed(1)} GB</strong></div>
            <div><small>CURRENTLY AVAILABLE</small><strong>{toGb(ram.availableBytes).toFixed(1)} GB</strong></div>
            <div><small>MAXIMUM CEILING</small><strong>{maxGb} GB</strong></div>
          </div>
          <label className="shell-ram-toggle">
            <span><strong>Enable RAM Limiter</strong><small>Enforce a hard process-memory ceiling on the inference worker.</small></span>
            <input type="checkbox" checked={config.enabled} disabled={saving} onChange={e => setConfig(current => ({ ...current, enabled: e.target.checked }))} />
          </label>
          <div className={"shell-ram-slider" + (!config.enabled ? " is-disabled" : "")}>
            <div className="shell-ram-slider-heading"><label htmlFor="ram-limit">Worker memory ceiling</label><strong>{config.limitGb} GB</strong></div>
            <input id="ram-limit" type="range" min="1" max={maxGb} step="1" value={Math.min(config.limitGb, maxGb)} disabled={!config.enabled || saving} onChange={e => setConfig(current => ({ ...current, limitGb: Number(e.target.value) }))} />
            <div className="shell-ram-slider-scale"><span>1 GB</span><span>{maxGb} GB</span></div>
            <small>This limits committed private memory, not resident working-set size. Runtime diagnostics report both measurements separately.</small>
          </div>
          <button type="button" disabled={!dirty || saving} onClick={() => void persist()}>{saving ? "Saving…" : "Save RAM Limiter"}</button>
        </> : <p className="shell-settings-footnote">RAM detection requires the native desktop host.</p>}
        {message && <p className="shell-settings-feedback" role="status">{message}</p>}
      </section>

      <div className="shell-settings-grid shell-settings-secondary">
        <button className="shell-settings-card" type="button" onClick={() => onNavigate("models")}><span className="shell-settings-card-icon">◈</span><span><strong>Local models</strong><small>GGUF library, original file paths and model selection.</small></span><span className="shell-settings-card-arrow">›</span></button>
        <div className="shell-settings-card is-staged"><span className="shell-settings-card-icon">⌑</span><span><strong>Privacy and permissions</strong><small>Explicit approval controls arrive with the secure tool system.</small></span><span className="shell-settings-card-state">PLANNED</span></div>
      </div>
      <p className="shell-settings-footnote">Changing the saved ceiling does not resize a running worker. Unload and load the model again to apply a new limit. No model is loaded automatically at startup.</p>
    </section>
  );
}
