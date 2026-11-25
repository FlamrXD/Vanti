#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod utils;

use dirs::download_dir;
use notify_rust::Notification;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::TrayIconBuilder;
use utils::filechecker::watch_added_files;
use utils::hashd::{check_file_hash, download_and_load_hashes};

slint::include_modules!();

fn get_downloads_folder() -> Option<PathBuf> {
    let path = download_dir()?;
    println!("Downloads folder: {}", path.display());
    Some(path)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;


    ui.window().on_close_requested(|| {
        slint::CloseRequestResponse::HideWindow
    });

    let protection_enabled = Arc::new(AtomicBool::new(ui.get_protection_enabled()));

    let ui_handle_for_settings = ui.as_weak();
    ui.on_settings_clicked(move || {
        println!("Settings button clicked!");
        let _ = ui_handle_for_settings.upgrade_in_event_loop(|ui| {
            let _ = ui.window().show();
        });
    });

    // Keep the atomic boolean synced whenever the user toggles protection in the UI.
    // Note: Ensure you have a callback or trigger in your .slint file (e.g., `callback protection-toggled(bool);`)
    // If your .slint property changes directly without a callback, see the note below.
    let protection_flag_for_ui = Arc::clone(&protection_enabled);
    let ui_handle_for_toggle = ui.as_weak();
    // Periodically or via callback sync; if you add `callback toggle_protection(bool)` in Slint:
    // ui.on_toggle_protection(move |enabled| {
    //     protection_flag_for_ui.store(enabled, Ordering::Relaxed);
    // });

    let ui_handle = ui.as_weak();
    let ui_handle_for_notification = ui.as_weak();
    let protection_flag_for_watcher = Arc::clone(&protection_enabled);

    // Background thread for file protection
    thread::spawn(move || {
        println!("Initializing hashes...");
        let malicious_hashes = match download_and_load_hashes() {
            Ok(hashes) => hashes,
            Err(e) => {
                eprintln!("Failed to load hashes: {e}");
                return;
            }
        };

        let Some(download_fold) = get_downloads_folder() else {
            return;
        };

        let _ = watch_added_files(&download_fold, true, move |path| {
            if !protection_flag_for_watcher.load(Ordering::Relaxed) {
                return;
            }

            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(ext, "crdownload" | "part" | "tmp") {
                    return;
                }
            }

            if let Ok(hash) = check_file_hash(&path) {
                if malicious_hashes.contains(&hash) {
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown file")
                        .to_string();

                    // Send desktop notification
                    let ui_handle_clone = ui_handle_for_notification.clone();
                    let notification_result = Notification::new()
                        .summary("Vanti Security Alert")
                        .body(&format!("Threat Neutralized: {}", file_name))
                        .appname("Vanti-RS")
                        .show();

                    if let Ok(mut handle) = notification_result {
                        handle.wait_for_action(move |_action| {
                            let _ = ui_handle_clone.upgrade_in_event_loop(|ui| {
                                let _ = ui.window().show();
                            });
                        });
                    }

                    let _ = ui_handle.upgrade_in_event_loop(move |ui| {
                        let current_count = ui.get_alert_count();
                        ui.set_alert_count(current_count + 1);
                        ui.set_alert_text(format!("Threat Neutralized: {}", file_name).into());
                        let _ = ui.window().show();
                    });
                }
            }
        });
    });

    // sys tray
    let tray_menu = Menu::new();
    let show_item = MenuItem::new("Show Settings", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let _ = tray_menu.append(&show_item);
    let _ = tray_menu.append(&quit_item);

    let show_id = show_item.id().clone();
    let quit_id = quit_item.id().clone();

    let icon = load_tray_icon();

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Vanti-RS Protection")
        .with_icon(icon)
        .build()?;

    let ui_handle_for_tray = ui.as_weak();
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        if event.id == show_id {
            let _ = ui_handle_for_tray.upgrade_in_event_loop(|ui| {
                let _ = ui.window().show();
            });
        } else if event.id == quit_id {
            let _ = slint::quit_event_loop();
        }
    }));
    slint::run_event_loop_until_quit()?;

    Ok(())
}

fn load_tray_icon() -> tray_icon::Icon {
    let icon_bytes = include_bytes!("../ui/assets/tmpico.png");

    let img = image::load_from_memory(icon_bytes)
        .expect("Failed to decode tray icon image")
        .into_rgba8();

    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    tray_icon::Icon::from_rgba(rgba, width, height)
        .expect("Failed to create tray icon from RGBA data")
}