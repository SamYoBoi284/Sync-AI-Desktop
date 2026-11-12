export function LoadingScreen() {
  return (
    <div className="startup-screen" role="status" aria-live="polite" aria-label="Starting Sync AI Desktop">
      <div className="startup-content">
        <div className="startup-core" aria-hidden="true">
          <span className="startup-core-ring startup-core-ring--outer" />
          <span className="startup-core-ring startup-core-ring--inner" />
          <span className="startup-core-orb" />
          <span className="startup-core-spark startup-core-spark--one" />
          <span className="startup-core-spark startup-core-spark--two" />
        </div>

        <div className="startup-brand">
          <p className="startup-wordmark">SYNC AI</p>
          <p className="startup-edition">DESKTOP SYSTEM</p>
        </div>

        <div className="startup-status">
          <span className="startup-status-dot" />
          <span>INITIALIZING DESKTOP ENVIRONMENT</span>
        </div>

        <div className="startup-progress" aria-hidden="true">
          <span />
        </div>
        <p className="startup-footnote">LOCAL-FIRST · YOUR SPACE IS PREPARING</p>
      </div>
    </div>
  );
}
