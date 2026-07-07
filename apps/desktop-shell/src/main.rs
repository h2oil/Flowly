//! Flowly Windows shell: a small always-on-top transparent bubble near the
//! top of the screen. Frameless and draggable (drag regions in the UI, plus
//! nudge-left/right commands); position persists across runs. The window
//! shrinks to just the pill during a take so it never blocks clicks over
//! other apps, and grows back for the library.

#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod engine;

use engine::{Cmd, SessionSlot};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, PhysicalPosition, State};

const LIBRARY_SIZE: (f64, f64) = (720.0, 620.0);
const TAKE_SIZE: (f64, f64) = (720.0, 200.0);

#[derive(Serialize, Deserialize)]
struct SavedPos {
    x: i32,
    y: i32,
}

fn pos_file(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("position.json"))
}

fn restore_position(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else { return };
    let saved: Option<SavedPos> = pos_file(app)
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok());
    match saved {
        Some(p) => {
            let _ = window.set_position(PhysicalPosition::new(p.x, p.y));
        }
        None => {
            // Default: top-center of the primary monitor, just below the
            // webcam. The user drags it under their camera from there.
            if let Ok(Some(monitor)) = window.primary_monitor() {
                let scale = monitor.scale_factor();
                let mw = monitor.size().width as f64 / scale;
                let x = ((mw - LIBRARY_SIZE.0) / 2.0).max(0.0);
                let _ = window.set_position(LogicalPosition::new(x, 8.0));
            }
        }
    }
}

fn save_position(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else { return };
    if let (Ok(pos), Some(path)) = (window.outer_position(), pos_file(app)) {
        let _ = std::fs::create_dir_all(path.parent().expect("has parent"));
        let _ = std::fs::write(
            path,
            serde_json::to_string(&SavedPos { x: pos.x, y: pos.y }).expect("serializes"),
        );
    }
}

#[tauri::command]
fn list_mics() -> Vec<String> {
    engine::list_mics()
}

#[tauri::command]
fn preload_model(app: AppHandle) -> Result<(), String> {
    engine::preload_model(&app)
}

#[tauri::command]
fn start_session(
    app: AppHandle,
    slot: State<SessionSlot>,
    script: String,
    mic: Option<String>,
) -> Result<(), String> {
    stop_session(slot.clone());
    let session = engine::start(app, script, mic)?;
    *slot.lock().expect("slot lock") = Some(session);
    Ok(())
}

#[tauri::command]
fn stop_session(slot: State<SessionSlot>) {
    if let Some(session) = slot.lock().expect("slot lock").take() {
        session.stop.store(true, Ordering::Relaxed);
        let _ = session.cmd_tx.send(Cmd::Stop);
    }
}

#[tauri::command]
fn jump_to(slot: State<SessionSlot>, token_index: usize) {
    if let Some(session) = slot.lock().expect("slot lock").as_ref() {
        let _ = session.cmd_tx.send(Cmd::Jump(token_index));
    }
}

/// Move the bubble horizontally (the ◀ ▶ controls).
#[tauri::command]
fn nudge(app: AppHandle, dx: i32) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(pos) = window.outer_position() {
            let _ = window.set_position(PhysicalPosition::new(pos.x + dx, pos.y));
            save_position(&app);
        }
    }
}

/// Shrink to pill-only during a take so the transparent remainder of the
/// window can't sit over (and block clicks in) other apps.
#[tauri::command]
fn resize_for_take(app: AppHandle, take: bool) {
    if let Some(window) = app.get_webview_window("main") {
        let (w, h) = if take { TAKE_SIZE } else { LIBRARY_SIZE };
        let _ = window.set_size(LogicalSize::new(w, h));
    }
}

fn main() {
    tauri::Builder::default()
        .manage(SessionSlot::default())
        .invoke_handler(tauri::generate_handler![
            list_mics,
            preload_model,
            start_session,
            stop_session,
            jump_to,
            nudge,
            resize_for_take
        ])
        .setup(|app| {
            restore_position(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Moved(_)) {
                // Cheap debounce: drags fire Moved continuously; writing a
                // ~30-byte JSON on each is still negligible, but skip while a
                // button is likely held by only saving every 250 ms.
                use std::sync::atomic::{AtomicU64, Ordering};
                use std::time::{SystemTime, UNIX_EPOCH};
                static LAST: AtomicU64 = AtomicU64::new(0);
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
                if now.saturating_sub(LAST.load(Ordering::Relaxed)) > 250 {
                    LAST.store(now, Ordering::Relaxed);
                    save_position(&window.app_handle());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Flowly");
}
