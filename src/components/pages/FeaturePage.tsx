import type { PageId } from "../navigation/Sidebar";

interface FeatureDetails {
  eyebrow: string;
  title: string;
  description: string;
  status: string;
  detailTitle: string;
  details: string[];
  icon: string;
}

const featureDetails: Record<Exclude<PageId, "chat" | "runtime">, FeatureDetails> = {
  history: {
    eyebrow: "CONVERSATION ARCHIVE",
    title: "Your history, in one place.",
    description: "Saved conversations will live here once the local chat store is connected.",
    status: "STORAGE NOT IMPLEMENTED",
    detailTitle: "Planned for this area",
    details: [
      "Persistent conversations and message ordering",
      "Search, rename, archive and delete controls",
      "Compatible import and export for the Android chat backup format",
    ],
    icon: "◷",
  },
  workspace: {
    eyebrow: "PROJECT SPACE",
    title: "A home for your work.",
    description: "Workspace will collect documents and artifacts created during your sessions.",
    status: "WORKSPACE NOT IMPLEMENTED",
    detailTitle: "Planned for this area",
    details: [
      "Generated artifacts with clear source and status",
      "Local persistence and safe file export",
      "A focused view of work connected to conversations",
    ],
    icon: "▱",
  },
  tasks: {
    eyebrow: "PERSONAL ORGANIZATION",
    title: "Keep the loose ends together.",
    description: "Tasks and notes will provide a small local space for the things you need to remember.",
    status: "TASKS & NOTES NOT IMPLEMENTED",
    detailTitle: "Planned for this area",
    details: [
      "Lightweight tasks, notes and completion state",
      "Local persistence across app launches",
      "Clear separation between user notes and model-generated content",
    ],
    icon: "☷",
  },
  files: {
    eyebrow: "LOCAL DOCUMENTS",
    title: "Your files stay yours.",
    description: "The File Library will organize files you explicitly choose to add to Sync AI.",
    status: "FILE LIBRARY NOT IMPLEMENTED",
    detailTitle: "Planned for this area",
    details: [
      "User-selected imports with visible file metadata",
      "Document organization and source tracking",
      "No unrestricted model access to the filesystem",
    ],
    icon: "▤",
  },
  models: {
    eyebrow: "ON-DEVICE INTELLIGENCE",
    title: "Bring your own model.",
    description: "The desktop model manager will handle GGUF imports and the local inference lifecycle.",
    status: "NO MODEL LOADED",
    detailTitle: "Model manager roadmap",
    details: [
      "Import GGUF files and calculate SHA-256 hashes",
      "Track model metadata, duplicates and the active selection",
      "Load, unload and inspect models through the Windows runtime",
    ],
    icon: "◈",
  },
  appearance: {
    eyebrow: "PERSONALIZATION",
    title: "Make the space yours.",
    description: "Appearance controls will bring the Android identity and theme system into the desktop shell.",
    status: "APPEARANCE CONTROLS NOT IMPLEMENTED",
    detailTitle: "Planned controls",
    details: [
      "Six visual identities and eight accent colors",
      "System, light and dark appearance modes",
      "Motion intensity and reduced-motion behavior",
    ],
    icon: "✳",
  },
};

interface FeaturePageProps {
  page: Exclude<PageId, "chat" | "runtime">;
}

export function FeaturePage({ page }: FeaturePageProps) {
  const feature = featureDetails[page];

  return (
    <div className="feature-page">
      <header className="page-intro">
        <p className="eyebrow-label">{feature.eyebrow}</p>
        <h1>{feature.title}</h1>
        <p>{feature.description}</p>
      </header>

      <section className="feature-status-panel">
        <div className="feature-status-icon" aria-hidden="true">{feature.icon}</div>
        <div className="feature-status-copy">
          <span className="feature-status-kicker">CURRENT BUILD STATUS</span>
          <strong>{feature.status}</strong>
          <p>This destination is part of the desktop navigation, but its feature service has not been connected yet.</p>
        </div>
        <span className="feature-status-marker">IN DEVELOPMENT</span>
      </section>

      <section className="feature-detail-panel">
        <div className="panel-heading">
          <div>
            <p className="panel-kicker">IMPLEMENTATION NOTES</p>
            <h2>{feature.detailTitle}</h2>
          </div>
          <span className="panel-index">SYNC / 0{page === "history" ? 1 : page === "workspace" ? 2 : page === "tasks" ? 3 : page === "files" ? 4 : page === "models" ? 5 : 6}</span>
        </div>
        <div className="feature-detail-list">
          {feature.details.map((detail, index) => (
            <div className="feature-detail-row" key={detail}>
              <span className="detail-number">0{index + 1}</span>
              <span>{detail}</span>
              <span className="detail-state">PLANNED</span>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
