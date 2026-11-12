import { useEffect, useMemo, useState } from "react";
import { listChats, type ChatSummary } from "../../services/nativeBridge";
import { navigationSections, type PageId } from "../../types/navigation";
interface SidebarProps { activePage: PageId; onNavigate: (page: PageId) => void; onOpenChat: (chatId: string) => void; }
function dateGroup(value: string): string {
 const date=new Date(value); if(Number.isNaN(date.getTime())) return "OLDER";
 const now=new Date(),today=new Date(now.getFullYear(),now.getMonth(),now.getDate()).getTime();
 const then=new Date(date.getFullYear(),date.getMonth(),date.getDate()).getTime(),days=Math.floor((today-then)/86400000);
 if(days<=0)return "TODAY";if(days===1)return "YESTERDAY";if(days<7)return "PREVIOUS 7 DAYS";if(days<30)return "PREVIOUS 30 DAYS";return "OLDER";
}
export function Sidebar({activePage,onNavigate,onOpenChat}:SidebarProps) {
 const [query,setQuery]=useState(""),[recentChats,setRecentChats]=useState<ChatSummary[]>([]),[historyError,setHistoryError]=useState(false);
 useEffect(()=>{let current=true;const timeout=window.setTimeout(()=>{void listChats(query.trim(),40,0).then(rows=>{if(current){setRecentChats(rows);setHistoryError(false);}}).catch(()=>{if(current)setHistoryError(true);});},140);return()=>{current=false;window.clearTimeout(timeout);};},[query]);
 const groups=useMemo(()=>{const map=new Map<string,ChatSummary[]>();recentChats.forEach(chat=>{const key=dateGroup(chat.updatedAt),items=map.get(key)??[];items.push(chat);map.set(key,items);});return Array.from(map.entries());},[recentChats]);
 return <aside className="shell-sidebar" aria-label="Main navigation">
  <div className="shell-brand-lockup"><div className="shell-brand-mark" aria-hidden="true"><span/><span/><span/></div><div><p className="shell-brand-name">SYNC AI</p><p className="shell-brand-caption">DESKTOP SYSTEM</p></div></div>
  <button className="shell-new-chat" type="button" onClick={()=>onNavigate("new-chat")} aria-label="Start a new chat"><span aria-hidden="true">+</span><span>New chat</span><kbd>CTRL N</kbd></button>
  <div className="shell-sidebar-content">
   <section className="shell-recent-section" aria-label="Recent conversations">
    <div className="shell-recent-heading"><h2>RECENT CHATS</h2><button type="button" onClick={()=>onNavigate("history")}>SEE ALL</button></div>
    <label className="shell-chat-search"><span aria-hidden="true">⌕</span><input value={query} onChange={e=>setQuery(e.target.value)} placeholder="Search conversations" aria-label="Search recent conversations"/>{query&&<button type="button" onClick={()=>setQuery("")} aria-label="Clear search">×</button>}</label>
    {historyError?<p className="shell-recent-empty">History is unavailable in browser preview.</p>:groups.length===0?<p className="shell-recent-empty">{query?"No matching conversations.":"Your chats will appear here."}</p>:groups.map(([heading,chats])=><div className="shell-recent-group" key={heading}><h3>{heading}</h3>{chats.slice(0,8).map(chat=><button className="shell-recent-chat" type="button" key={chat.id} title={chat.title} onClick={()=>onOpenChat(chat.id)}><span className="shell-recent-chat-mark" aria-hidden="true">◌</span><span>{chat.title}</span></button>)}</div>)}
   </section>
   {navigationSections.map(section=><section className="shell-nav-section" key={section.heading}><h2>{section.heading}</h2><ul>{section.entries.map(entry=>{const active=activePage===entry.id;return <li key={entry.id}><button className={"shell-nav-entry"+(active?" is-active":"")} type="button" onClick={()=>onNavigate(entry.id)} aria-current={active?"page":undefined}><span className="shell-nav-glyph" aria-hidden="true">{entry.glyph}</span><span className="shell-nav-label">{entry.label}</span>{entry.availability==="planned"&&<span className="shell-nav-status">SOON</span>}</button></li>;})}</ul></section>)}
  </div>
  <div className="shell-sidebar-footer"><button className={"shell-settings-button"+(activePage==="settings"?" is-active":"")} type="button" onClick={()=>onNavigate("settings")} aria-current={activePage==="settings"?"page":undefined}><span aria-hidden="true">⚙</span><span>Settings</span><span className="shell-settings-chevron">›</span></button><div className="shell-local-seal"><span className="shell-seal-dot"/><span>LOCAL-FIRST</span></div><p>Private by default. Explicit by design.</p><span className="shell-build-label">DESKTOP FOUNDATION · 0.1.0</span></div>
 </aside>;
}
