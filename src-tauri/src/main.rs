// src-tauri/src/main.rs

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adapters;
mod core;
mod engine;
mod ipc;

#[cfg(target_os = "macos")]
use cocoa::base::id;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

use std::sync::OnceLock;
use tauri::{AppHandle, Builder, Emitter, Manager, Window, WindowEvent};

// Imports
use crate::adapters::config_store::ConfigStore;
use crate::adapters::hooks::HookRunner;
use crate::core::domain::Event;
use crate::engine::runtime::Runtime;
use crate::ipc::commands::*;

lazy_static::lazy_static! {
    pub static ref APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
}

#[macro_export]
macro_rules! notify {
    ($($arg:tt)*) => {{
        use tauri_plugin_notification::NotificationExt;
        let app_handle = crate::APP_HANDLE.get().unwrap();
        app_handle
            .notification()
            .builder()
            .title("mytt")
            .body($($arg)*)
            .show()
            .unwrap_or_else(|e| eprintln!("{}", e));
        }};
}

#[macro_export]
macro_rules! notify_on_error {
    ($($arg:tt)*) => {{
        crate::notify!(format!($($arg)*));
    }};
}

#[cfg(target_os = "macos")]
#[macro_export]
macro_rules! nsstring_to_string {
    ($ns_string:expr) => {{
        use objc::{sel, sel_impl};
        let utf8: id = objc::msg_send![$ns_string, UTF8String];
        if !utf8.is_null() {
            Some({
                std::ffi::CStr::from_ptr(utf8 as *const std::ffi::c_char)
                    .to_string_lossy()
                    .into_owned()
            })
        } else {
            None
        }
    }};
}

// --- Commands ---

// Polling compat for old UI components
#[tauri::command]
fn get_time(runtime: tauri::State<Runtime>) -> String {
    let dto = crate::ipc::commands::get_state(runtime);
    let total_seconds = dto.remaining_ms / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[tauri::command]
fn get_frontmost_application() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let shared_workspace: id = unsafe { msg_send![class!(NSWorkspace), sharedWorkspace] };
        let frontmost_app: id = unsafe { msg_send![shared_workspace, frontmostApplication] };
        let app_name: id = unsafe { msg_send![frontmost_app, localizedName] };
        unsafe { nsstring_to_string!(app_name) }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

// --- OS / Window Event Adapters ---

#[tauri::command]
fn on_focus(runtime: tauri::State<Runtime>) {
    // 1. Fetch current app name
    if let Some(name) = get_frontmost_application() {
        // 2. Notify Runtime (Reducer will update state and run hooks)
        runtime.emit(Event::AppChanged { name: name.clone() });

        // 3. Notify Frontend (UI update)
        if let Some(handle) = APP_HANDLE.get() {
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.emit("on_focus", Some(name));
            }
        }
    }
}

#[tauri::command]
fn on_blur(runtime: tauri::State<Runtime>) {
    // Just re-check frontmost app on blur?
    // Or just emit event. Logic is same as focus: "Something changed, check who is on top".
    on_focus(runtime);
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    let runtime = window.app_handle().state::<Runtime>();

    match event {
        WindowEvent::Focused(true) => on_focus(runtime),
        WindowEvent::Focused(false) => on_blur(runtime),
        _ => {}
    }
}

pub fn main() {
    Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            // Adapters
            app.manage(HookRunner::new());
            app.manage(ConfigStore::new());

            // Engine
            let runtime = Runtime::new(app.handle().clone());
            app.manage(runtime);

            APP_HANDLE.set(app.handle().clone()).unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Core API
            intent,
            get_state,
            get_config,
            set_config,
            // Legacy Compat
            get_time,
            // OS Integration
            on_focus,
            on_blur,
            get_frontmost_application,
            // IPC
            get_duration_suggestions,
            get_session_history,
        ])
        .on_window_event(|window, event| {
            on_window_event(window, event);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
