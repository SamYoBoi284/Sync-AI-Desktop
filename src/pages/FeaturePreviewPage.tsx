import { plannedPageCopy, type PageId } from "../types/navigation";

type PlannedPageId = Exclude<PageId, "new-chat" | "runtime" | "settings">;

interface FeaturePreviewPageProps {
  page: PlannedPageId;
}

export function FeaturePreviewPage({ page }: FeaturePreviewPageProps) {
  const content = plannedPageCopy[page];

  return (
    <div className="shell-page shell-standard-page">
      <div className="shell-page-eyebrow"><span className="shell-eyebrow-line" />{content.eyebrow}</div>
      <section className="shell-preview-panel">
        <div className="shell-preview-symbol" aria-hidden="true">⌁</div>
        <span className="shell-preview-label">FEATURE NOT IMPLEMENTED</span>
        <h1>{content.title}</h1>
        <p className="shell-preview-description">{content.description}</p>
        <div className="shell-preview-divider" />
        <p className="shell-preview-detail">{content.detail}</p>
      </section>
    </div>
  );
}
