// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
use crate::config::script_on_submit;
#[cfg(target_os = "macos")]
use cocoa::base::id;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(target_os = "macos")]
#[macro_export]
macro_rules! nsstring_to_string {
    ($ns_string:expr) => {{
        use objc::{sel, sel_impl};
        let utf8: id = objc::msg_send![$ns_string, UTF8String];
        let string = if !utf8.is_null() {
            Some({
                std::ffi::CStr::from_ptr(utf8 as *const std::ffi::c_char)
                    .to_string_lossy()
                    .into_owned()
            })
        } else {
            None
        };
        string
    }};
}
pub struct Stopwatch {
    start_time: Option<Instant>,
    paused_duration: Duration,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            start_time: None,
            paused_duration: Duration::new(0, 0),
        }
    }

    pub fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now() - self.paused_duration);
            self.paused_duration = Duration::new(0, 0);
        }
    }

    pub fn pause(&mut self) {
        if let Some(start_time) = self.start_time {
            self.paused_duration = Instant::now() - start_time;
            self.start_time = None;
        }
    }

    pub fn resume(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now() - self.paused_duration);
        }
    }

    pub fn stop(&mut self) {
        self.start_time = None;
        self.paused_duration = Duration::new(0, 0);
    }

    pub fn elapsed(&self) -> u128 {
        if let Some(start_time) = self.start_time {
            (self.paused_duration.as_millis() + (Instant::now() - start_time).as_millis()) as u128
        } else {
            self.paused_duration.as_millis() as u128
        }
    }

    pub fn format_time(&self) -> String {
        let elapsed = self.elapsed();
        let hours = elapsed / 3600000;
        let minutes = (elapsed % 3600000) / 60000;
        let seconds = (elapsed % 60000) / 1000;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    pub fn reset(&mut self) {
        self.start_time = None;
        self.paused_duration = Duration::new(0, 0);
    }
}

#[tauri::command]
fn start_timer(stopwatch: tauri::State<Mutex<Stopwatch>>) {
    if let Ok(mut stopwatch) = stopwatch.lock() {
        stopwatch.start();
    }
}

#[tauri::command]
fn stop_timer(stopwatch: tauri::State<Mutex<Stopwatch>>) {
    if let Ok(mut stopwatch) = stopwatch.lock() {
        stopwatch.stop();
    }
}

#[tauri::command]
fn get_time(stopwatch: tauri::State<Mutex<Stopwatch>>) -> String {
    if let Ok(stopwatch) = stopwatch.lock() {
        println!("{}", stopwatch.format_time());
        return stopwatch.format_time();
    }
    "Error".to_string()
}

#[tauri::command]
fn pause_timer(stopwatch: tauri::State<Mutex<Stopwatch>>) {
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.pause();
}

#[tauri::command]
fn resume_timer(stopwatch: tauri::State<Mutex<Stopwatch>>) {
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.resume();
}

#[tauri::command]
fn reset_timer(stopwatch: tauri::State<Mutex<Stopwatch>>) {
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.reset();
}

#[tauri::command]
fn submit(stopwatch: tauri::State<Mutex<Stopwatch>>) -> Result<(), String> {
    let mut front_app_name = get_frontmost_application();
    if front_app_name.is_none() {
        front_app_name = Some("Unknown".to_string());
    }
    let mut stopwatch = stopwatch.lock().map_err(|e| e.to_string())?;
    let time = stopwatch.format_time();

    if let Some(script_on_submit) = script_on_submit() {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(script_on_submit)
            .arg(time)
            .arg(front_app_name.unwrap_or("Unknown".to_string()))
            .spawn()
            .map_err(|e| e.to_string())?;

        let _ = child.wait();
    }

    stopwatch.stop();

    Ok(())
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

pub fn main() {
    let stopwatch = Mutex::new(Stopwatch::new());
    get_frontmost_application();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(stopwatch)
        .invoke_handler(tauri::generate_handler![
            start_timer,
            stop_timer,
            get_time,
            reset_timer,
            pause_timer,
            resume_timer,
            submit,
            get_frontmost_application,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
