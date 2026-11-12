export type PageId = "new-chat" | "history" | "workspace" | "tasks" | "library" | "models" | "appearance" | "runtime" | "settings";
export interface NavigationEntry { id: PageId; label: string; glyph: string; availability: "available" | "planned"; }
export interface NavigationSection { heading: string; entries: NavigationEntry[]; }
export const navigationSections: NavigationSection[] = [
 {heading:"WORKSPACE",entries:[{id:"workspace",label:"Workspace",glyph:"▧",availability:"available"},{id:"tasks",label:"Tasks & Notes",glyph:"☷",availability:"available"},{id:"library",label:"File Library",glyph:"▤",availability:"available"}]},
 {heading:"SYSTEM",entries:[{id:"models",label:"Local models",glyph:"◈",availability:"available"},{id:"appearance",label:"Appearance",glyph:"◐",availability:"planned"},{id:"runtime",label:"Runtime",glyph:"⌁",availability:"available"}]}
];
export const pageTitles: Record<PageId,string> = {"new-chat":"New chat",history:"History",workspace:"Workspace",tasks:"Tasks & Notes",library:"File Library",models:"Local models",appearance:"Appearance",runtime:"Runtime",settings:"Settings"};
export const plannedPageCopy: Record<Exclude<PageId,"new-chat"|"runtime"|"settings">,{eyebrow:string;title:string;description:string;detail:string}> = {
 history:{eyebrow:"CONVERSATION STORAGE",title:"Your conversations live here.",description:"Browse, search, reopen and manage locally stored conversations.",detail:"Chat storage and versioned desktop backup are implemented in Phase 3 source."},
 workspace:{eyebrow:"WORKSPACE",title:"A place for work in progress.",description:"Save assistant replies, drafts, code and documents in a persistent local workspace.",detail:"Artifacts can be selected explicitly as context for future local conversations."},
 tasks:{eyebrow:"PERSONAL ORGANIZATION",title:"Keep the small things in view.",description:"Manage lightweight tasks and notes alongside your conversations.",detail:"Tasks, notes, due dates, pinning and opt-in chat context are stored locally in SQLite."},
 library:{eyebrow:"LOCAL FILES",title:"A local index of your files.",description:"Register selected files, track their original locations and detect when they move.",detail:"The library stores paths and SHA-256 fingerprints only. It does not read file contents into chat or expose unrestricted filesystem access."},
 
 appearance:{eyebrow:"PERSONALIZATION",title:"Appearance controls are coming.",description:"Theme, color accent and motion preferences will be brought over from the Android design system.",detail:"The desktop implementation will support system, dark and light modes, six theme identities, eight accents and reduced motion."}
};
