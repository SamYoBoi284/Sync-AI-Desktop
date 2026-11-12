#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|argument| argument == "--inference-worker") {
        sync_ai_desktop_lib::run_inference_worker();
        return;
    }
    sync_ai_desktop_lib::run();
}
