use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, State, WindowEvent,
};

mod commands;
mod storage;
mod inference;

use storage::Database;
use inference::runtime::RuntimeManager;

pub fn run_inference_worker() {
    inference::worker::run();
}

struct StartupGate {
    splash_pending: AtomicBool,
}

impl StartupGate {
    fn new() -> Self {
        Self {
            splash_pending: AtomicBool::new(true),
        }
    }
}

#[tauri::command]
fn consume_startup_splash(startup_gate: State<'_, StartupGate>) -> bool {
    startup_gate.splash_pending.swap(false, Ordering::AcqRel)
}

fn restore_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn create_tray_icon() -> Image<'static> {
    const SIZE: u32 = 32;
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let px = x as i32;
            let py = y as i32;
            let nearest_x = px.clamp(8, 23);
            let nearest_y = py.clamp(8, 23);
            let rounded_square = (px - nearest_x).pow(2) + (py - nearest_y).pow(2) <= 25;
            let mut color = if rounded_square {
                [16, 17, 32, 255]
            } else {
                [0, 0, 0, 0]
            };

            let bar = if (10..13).contains(&x) && (13..21).contains(&y) {
                Some([168, 121, 255, 255])
            } else if (15..18).contains(&x) && (8..24).contains(&y) {
                Some([104, 216, 235, 255])
            } else if (20..23).contains(&x) && (11..21).contains(&y) {
                Some([168, 121, 255, 255])
            } else {
                None
            };

            if let Some(bar_color) = bar {
                color = bar_color;
            }

            let offset = ((y * SIZE + x) * 4) as usize;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }

    Image::new_owned(rgba, SIZE, SIZE)
}

fn install_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let open_item = MenuItem::with_id(app, "open", "Open Sync AI", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Hide Sync AI", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Exit Sync AI", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &hide_item, &quit_item])?;

    let tray_app = app.handle().clone();

    TrayIconBuilder::new()
        .icon(create_tray_icon())
        .tooltip("Sync AI Desktop")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => restore_main_window(app),
            "hide" => hide_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(move |_tray, event| {
            let should_restore = matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            );

            if should_restore {
                restore_main_window(&tray_app);
            }
        })
        .build(app)?;

    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        // Register first: the single-instance plugin must run before other plugins.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            restore_main_window(app);
        }))
        .manage(StartupGate::new())
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            consume_startup_splash,
            commands::list_chats,
            commands::create_chat,
            commands::get_chat,
            commands::rename_chat,
            commands::delete_chat,
            commands::append_message,
            commands::update_message_diagnostics,
            commands::export_chat_backup,
            commands::import_chat_backup,
            commands::get_setting,
            commands::set_setting,
            commands::get_host_ram_info,
            commands::list_models,
            commands::import_gguf,
            commands::set_default_model,
            commands::remove_model,
            commands::get_runtime_status,
            commands::estimate_model_resources,
            commands::load_model,
            commands::unload_model
        ])
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            let database_path = app_data_dir.join("sync-ai-desktop.sqlite3");
            let database = Database::open(&database_path).map_err(std::io::Error::other)?;
            app.manage(database);
            app.manage(RuntimeManager::new());
            install_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    // Closing the window hides it; the tray menu's Exit action quits.
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Sync AI Desktop");
}
