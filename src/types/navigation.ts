export type PageId = "new-chat" | "history" | "workspace" | "tasks" | "library" | "models" | "appearance" | "runtime" | "settings";
export interface NavigationEntry { id: PageId; label: string; glyph: string; availability: "available" | "planned"; }
export interface NavigationSection { heading: string; entries: NavigationEntry[]; }
export const navigationSections: NavigationSection[] = [
 {heading:"WORKSPACE",entries:[{id:"workspace",label:"Workspace",glyph:"▧",availability:"planned"},{id:"tasks",label:"Tasks & Notes",glyph:"☷",availability:"planned"},{id:"library",label:"File Library",glyph:"▤",availability:"planned"}]},
 {heading:"SYSTEM",entries:[{id:"models",label:"Local models",glyph:"◈",availability:"planned"},{id:"appearance",label:"Appearance",glyph:"◐",availability:"planned"},{id:"runtime",label:"Runtime",glyph:"⌁",availability:"available"}]}
];
export const pageTitles: Record<PageId,string> = {"new-chat":"New chat",history:"History",workspace:"Workspace",tasks:"Tasks & Notes",library:"File Library",models:"Local models",appearance:"Appearance",runtime:"Runtime",settings:"Settings"};
export const plannedPageCopy: Record<Exclude<PageId,"new-chat"|"runtime"|"settings">,{eyebrow:string;title:string;description:string;detail:string}> = {
 history:{eyebrow:"CONVERSATION STORAGE",title:"Your conversations live here.",description:"Browse, search, reopen and manage locally stored conversations.",detail:"Chat storage and versioned desktop backup are implemented in Phase 3 source."},
 workspace:{eyebrow:"WORKSPACE",title:"A place for work in progress.",description:"Generated artifacts and ongoing work will have a dedicated home here.",detail:"Workspace persistence and artifact handling are planned after the chat and storage foundations."},
 tasks:{eyebrow:"PERSONAL ORGANIZATION",title:"Tasks and notes are not wired yet.",description:"This area will hold lightweight notes and task lists alongside your conversations.",detail:"The Android feature is a reference; the desktop data model and interactions still need to be implemented."},
 library:{eyebrow:"LOCAL FILES",title:"Your file library is not connected.",description:"A local library will make selected files and supported documents available to your workspace.",detail:"File access will use narrow, user-selected scopes. No unrestricted filesystem access will be exposed to the model."},
 models:{eyebrow:"LOCAL INFERENCE",title:"No model manager yet.",description:"GGUF model import, verification, selection and lifecycle controls are part of Phase 4.",detail:"The manager will track model metadata and hashes. Model files themselves will remain outside the Git repository."},
 appearance:{eyebrow:"PERSONALIZATION",title:"Appearance controls are coming.",description:"Theme, color accent and motion preferences will be brought over from the Android design system.",detail:"The desktop implementation will support system, dark and light modes, six theme identities, eight accents and reduced motion."}
};
