export function NewChatPage() {
  return (
    <div className="shell-page shell-chat-page">
      <div className="shell-page-eyebrow">
        <span className="shell-eyebrow-line" />
        PRIVATE BY DEFAULT · LOCAL-FIRST
      </div>

      <section className="shell-chat-welcome" aria-labelledby="chat-welcome-title">
        <div className="shell-chat-core" aria-hidden="true">
          <span className="shell-chat-core-ring shell-chat-core-ring--outer" />
          <span className="shell-chat-core-ring shell-chat-core-ring--inner" />
          <span className="shell-chat-core-orb" />
          <span className="shell-chat-core-node" />
        </div>
        <p className="shell-section-kicker">A NEW CONVERSATION</p>
        <h1 id="chat-welcome-title">Your space.<br /><span>Your signal.</span></h1>
        <p className="shell-chat-description">
          A quieter place to think, create and get things done, powered by models running on your own machine.
        </p>
      </section>

      <section className="shell-chat-readiness" aria-label="Chat readiness">
        <div className="shell-readiness-card">
          <div className="shell-readiness-icon shell-readiness-icon--ready">01</div>
          <div>
            <h2>Desktop shell</h2>
            <p>Navigation and the native application bridge are in place.</p>
          </div>
          <span className="shell-readiness-state">READY</span>
        </div>
        <div className="shell-readiness-card">
          <div className="shell-readiness-icon">02</div>
          <div>
            <h2>Conversation engine</h2>
            <p>Message storage, streaming and queue handling are the next chat milestone.</p>
          </div>
          <span className="shell-readiness-state is-pending">NEXT</span>
        </div>
        <div className="shell-readiness-card">
          <div className="shell-readiness-icon">03</div>
          <div>
            <h2>Local model</h2>
            <p>GGUF management and llama.cpp integration follow the storage foundation.</p>
          </div>
          <span className="shell-readiness-state is-pending">PLANNED</span>
        </div>
      </section>

      <div className="shell-composer-preview" aria-label="Chat composer not yet available">
        <div className="shell-composer-placeholder">
          <span className="shell-composer-plus" aria-hidden="true">+</span>
          <span>Chat composer is the next implementation slice</span>
        </div>
        <button className="shell-composer-send" type="button" disabled aria-label="Send message">
          ↑
        </button>
      </div>
      <p className="shell-composer-note">No messages are sent in this build. The local inference pipeline is not connected yet.</p>
    </div>
  );
}
