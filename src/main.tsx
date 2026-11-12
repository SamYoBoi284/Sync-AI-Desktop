import React from "react";
import ReactDOM from "react-dom/client";
import { isTauri } from "@tauri-apps/api/core";
import App from "./App";
import { consumeStartupSplash } from "./services/nativeBridge";
import "./styles.css";
import "./shell.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("The Sync AI Desktop root element is missing.");
}

async function bootstrap() {
  let showStartupSplash = false;

  if (isTauri()) {
    try {
      // Rust permits the startup splash only once per application process.
      showStartupSplash = await consumeStartupSplash();
    } catch {
      // If startup IPC is unavailable, still mount the app and surface its bridge status.
      showStartupSplash = false;
    }
  }

  ReactDOM.createRoot(rootElement).render(
    <React.StrictMode>
      <App showStartupSplash={showStartupSplash} />
    </React.StrictMode>,
  );
}

void bootstrap();
