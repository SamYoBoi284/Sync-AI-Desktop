import { useCallback, useEffect, useState } from "react";
import { importGguf, listModels, removeModel, setDefaultModel, type ModelRecord } from "../services/nativeBridge";
function sizeLabel(bytes:number){return bytes>=1024**3?(bytes/1024**3).toFixed(2)+" GB":(bytes/1024**2).toFixed(0)+" MB";}
export function LocalModelsPage(){
 const [models,setModels]=useState<ModelRecord[]>([]),[busy,setBusy]=useState(false),[error,setError]=useState(""),[notice,setNotice]=useState("");
 const refresh=useCallback(async()=>{try{setModels(await listModels());setError("");}catch{setError("Model registry is available only inside the desktop app.");}},[]);
 useEffect(()=>{void refresh();},[refresh]);
 const add=async()=>{setBusy(true);setError("");setNotice("");try{const result=await importGguf();if(result){setNotice(result.duplicate?"That exact model is already in your library.":"Model imported and SHA-256 verified.");await refresh();}}catch(e){setError(String(e));}finally{setBusy(false);}};
 const makeDefault=async(id:string)=>{setBusy(true);try{await setDefaultModel(id);await refresh();}catch(e){setError(String(e));}finally{setBusy(false);}};
 const remove=async(model:ModelRecord)=>{if(!window.confirm("Remove "+model.name+" from Sync AI and delete its managed model file?"))return;setBusy(true);try{await removeModel(model.id);await refresh();}catch(e){setError(String(e));}finally{setBusy(false);}};
 return <section className="shell-page shell-standard-page">
  <div className="shell-page-eyebrow"><span className="shell-eyebrow-line"/>LOCAL INFERENCE LIBRARY</div>
  <div className="shell-page-heading-row"><div><p className="shell-section-kicker">PHASE 4 · STAGE 1</p><h1>Local models<span>.</span></h1><p className="shell-page-description">Import GGUF files into Sync’s managed model library. Model weights stay on this machine and are never committed to the repository.</p></div><button className="shell-model-import" type="button" disabled={busy} onClick={()=>void add()}>{busy?"WORKING…":"+ IMPORT GGUF"}</button></div>
  <div className="shell-model-library-head"><div><h2>MODEL LIBRARY</h2><span>{models.length} {models.length===1?"MODEL":"MODELS"}</span></div><span>LOCAL STORAGE ONLY</span></div>
  {error&&<p className="shell-model-message is-error" role="alert">{error}</p>}{notice&&<p className="shell-model-message" role="status">{notice}</p>}
  {models.length===0?<div className="shell-model-empty"><span>◈</span><strong>No GGUF models imported</strong><p>Import a GGUF model to add it to your local library. Sync calculates its SHA-256 hash and detects duplicates.</p><button type="button" onClick={()=>void add()} disabled={busy}>Choose a GGUF file</button></div>:<div className="shell-model-list">{models.map(model=><article className="shell-model-card" key={model.id}><div className="shell-model-icon">◈</div><div className="shell-model-info"><div className="shell-model-title-row"><h3>{model.name}</h3>{model.isDefault&&<span className="shell-model-default">DEFAULT</span>}</div><p>{model.fileName} · {sizeLabel(model.sizeBytes)}</p><small>SHA-256 · {model.sha256}</small><small>Architecture and quantization will be read from GGUF metadata in Stage 2.</small></div><div className="shell-model-actions">{!model.isDefault&&<button type="button" disabled={busy} onClick={()=>void makeDefault(model.id)}>Set default</button>}<button type="button" disabled={busy} onClick={()=>void remove(model)}>Remove</button></div></article>)}</div>}
  <p className="shell-settings-footnote">Imported files are copied into Sync’s application data directory. Duplicate files are detected by SHA-256. Import does not load or execute a model.</p>
 </section>;
}
