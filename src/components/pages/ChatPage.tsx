import type { PageId } from "../navigation/Sidebar";

interface ChatPageProps {
  onNavigate: (page: PageId) => void;
}

function SyncCoreGraphic() {
  return (
    <div className="chat-core-graphic" aria-hidden="true">
      <span className="chat-core-ring chat-core-ring--outer" />
      <span className="chat-core-ring chat-core-ring--middle" />
      <span className="chat-core-ring chat-core-ring--inner" />
      <span className="chat-core-orbit-dot" />
      <span className="chat-core-orb">
        <span />
      </span>
    </div>
  );
}

export function ChatPage({ onNavigate }: ChatPageProps) {
  return (
    <div className="chat-page">
      <div className="chat-toolbar">
        <div className="model-state">
          <span className="model-state-indicator" />
          <span className="model-state-copy">
            <strong>NO MODEL LOADED</strong>
            <small>LOCAL INFERENCE IS NOT CONNECTED</small>
          </span>
        </div>
        <button className="quiet-button" type="button" onClick={() => onNavigate("models")}>
          <span className="button-plus">+</span>
          Manage models
        </button>
      </div>

      <section className="chat-welcome" aria-labelledby="chat-welcome-title">
        <SyncCoreGraphic />
        <p className="eyebrow-label">YOUR PRIVATE WORKSPACE</p>
        <h1 id="chat-welcome-title">
          A quieter space
          <br />
          <span>to think clearly.</span>
        </h1>
        <p className="chat-welcome-description">
          Your desktop companion is taking shape. Load a local GGUF model to
          begin on-device conversations once the inference runtime is connected.
        </p>
        <button
          className="primary-button"
          type="button"
          onClick={() => onNavigate("models")}
        >
          Explore local models
          <span aria-hidden="true">↗</span>
        </button>
      </section>

      <section className="chat-composer-card" aria-label="Message composer preview">
        <div className="composer-disabled-input" aria-disabled="true">
          <span>Load a local model before starting a conversation...</span>
          <span className="composer-lock" aria-hidden="true">⌑</span>
        </div>
        <div className="composer-bottom">
          <span className="composer-hint">
            <span className="composer-hint-dot" />
            CHAT GENERATION WILL BE AVAILABLE AFTER MODEL INTEGRATION
          </span>
          <button className="send-button" type="button" disabled aria-label="Send message">
            <span aria-hidden="true">↑</span>
          </button>
        </div>
      </section>

      <p className="chat-privacy-note">
        Designed for local-first conversations. Chat generation and message storage are not active in this build.
      </p>
    </div>
  );
}
