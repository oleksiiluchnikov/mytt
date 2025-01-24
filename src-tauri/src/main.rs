// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
use crate::config::get_script;
#[cfg(target_os = "macos")]
use cocoa::base::id;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Builder, Emitter, Manager, Window, WindowEvent};

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
            .body($($arg)*) // e.g. "Message"
            .show()
            .unwrap_or_else(|e| eprintln!("{}", e));
        }};
}

#[macro_export]
macro_rules! notify_on_error {
    ($($arg:tt)*) => {{
        crate::notify!(format!("Error: {}", $($arg)*));
        }};
}

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

pub struct CommandExecutor {
    sender: Sender<(String, String, String)>,
}

impl CommandExecutor {
    pub fn new() -> Self {
        let (sender, receiver) = channel();

        thread::spawn(move || {
            for (script, time, app_name) in receiver {
                let _ = Command::new("sh")
                    .arg("-c")
                    .arg(&script)
                    .arg(&time)
                    .arg(&app_name)
                    .output();
            }
        });

        Self { sender }
    }

    pub fn execute(&self, script_path: PathBuf, time: String, app_name: String) {
        let script_path = script_path.to_str().unwrap().to_string();
        let _ = self.sender.send((script_path, time, app_name));
    }
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
fn run_script(path: PathBuf) -> () {
    let handle = APP_HANDLE.get().unwrap();
    let stopwatch = handle.state::<Mutex<Stopwatch>>();
    let command_executor = handle.state::<CommandExecutor>();
    let front_app_name = get_frontmost_application().unwrap_or_else(|| "Unknown".to_string());
    let stopwatch = stopwatch.lock().unwrap();
    let time = stopwatch.format_time();
    command_executor.execute(path, time, front_app_name);
}

#[tauri::command]
fn on_log() -> Result<(), String> {
    if let Some(path) = get_script("on_log") {
        run_script(path);
    }
    on_stop();
    Ok(())
}

#[tauri::command]
fn on_start() {
    if let Some(path) = get_script("on_start") {
        run_script(path);
    }
    let handle = APP_HANDLE.get().unwrap();
    let stopwatch = handle.state::<Mutex<Stopwatch>>();
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.start();
    let state = handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();
    state.session.start_session();
}

#[tauri::command]
fn on_stop() {
    if let Some(path) = get_script("on_stop") {
        run_script(path);
    }
    let handle = APP_HANDLE.get().unwrap();
    let stopwatch = handle.state::<Mutex<Stopwatch>>();
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.stop();
    let state = handle.state::<Mutex<AppState>>();
    let mut state = state.lock().unwrap();
    state.session.start_session();
}

#[tauri::command]
fn on_resume() {
    if let Some(path) = get_script("on_resume") {
        run_script(path);
    }
    let handle = APP_HANDLE.get().unwrap();
    let stopwatch = handle.state::<Mutex<Stopwatch>>();
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.resume();
}

#[tauri::command]
fn on_skip_break() {
    if let Some(path) = get_script("on_skip_break") {
        run_script(path);
    }
    // let handle = APP_HANDLE.get().unwrap();
    // let state = handle.state::<Mutex<AppState>>();
    // let mut state = state.lock().unwrap();
    // state.session.skip_break();
    on_start();
}

#[tauri::command]
fn on_pause() {
    if let Some(path) = get_script("on_pause") {
        run_script(path);
    }
    let handle = APP_HANDLE.get().unwrap();
    let stopwatch = handle.state::<Mutex<Stopwatch>>();
    let mut stopwatch = stopwatch.lock().unwrap();
    stopwatch.pause();
}

#[tauri::command]
fn on_focus(state: tauri::State<'_, Mutex<AppState>>) {
    let front_app = state.lock().unwrap();
    let handle = APP_HANDLE.get().unwrap();
    let window = handle.get_webview_window("main").unwrap();
    window
        .emit("on_focus", Some(front_app.app_name.clone()))
        .unwrap();
}

#[tauri::command]
fn on_blur(state: tauri::State<'_, Mutex<AppState>>) {
    let mut front_app = state.lock().unwrap();
    front_app.app_name = get_frontmost_application().unwrap_or_else(|| "mytt".to_string());
    let handle = APP_HANDLE.get().unwrap();
    let window = handle.get_webview_window("main").unwrap();
    window
        .emit("on_blur", Some(front_app.app_name.clone()))
        .unwrap();
}

fn on_window_event(window: &Window, event: &WindowEvent) {
    let app_handle = window.app_handle();
    let state = app_handle.state::<Mutex<AppState>>();
    match event {
        WindowEvent::Focused(true) => {
            on_focus(state);
        }
        WindowEvent::Focused(false) => {
            on_blur(state);
        }
        _ => {}
    }
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

#[derive(Default)]
struct AppState {
    app_name: String,
    session: Session,
}

impl AppState {
    fn new() -> Self {
        Self {
            app_name: get_frontmost_application().unwrap_or_else(|| "mytt".to_string()),
            session: Session::new(),
        }
    }
}

#[derive(Default, PartialEq)]
enum SessionType {
    #[default]
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Default)]
struct Session {
    short_break_time: u64,
    long_break_time: u64,
    work_time: u64,
    work_sessions: u64,
    long_break_sessions: u64,
    current_session: SessionType,
}

impl Session {
    fn new() -> Self {
        Self {
            short_break_time: 5,
            long_break_time: 15,
            work_time: 25,
            work_sessions: 0,
            long_break_sessions: 0,
            current_session: SessionType::Work,
        }
    }

    fn start_session(&mut self) {
        match self.current_session {
            SessionType::Work => {
                // Start work session
                self.work_sessions += 1;
                if self.work_sessions % 4 == 0 {
                    self.current_session = SessionType::LongBreak;
                } else {
                    self.current_session = SessionType::ShortBreak;
                }
            }
            SessionType::ShortBreak => {
                // Start short break session
                self.current_session = SessionType::Work;
            }
            SessionType::LongBreak => {
                // Start long break session
                self.long_break_sessions += 1;
                self.current_session = SessionType::Work;
            }
        }
    }

    fn get_session_duration(&self) -> u64 {
        match self.current_session {
            SessionType::Work => self.work_time,
            SessionType::ShortBreak => self.short_break_time,
            SessionType::LongBreak => self.long_break_time,
        }
    }

    fn skip_break(&mut self) {
        match self.current_session {
            SessionType::Work => {
                self.current_session = SessionType::ShortBreak;
            }
            SessionType::ShortBreak => {
                self.current_session = SessionType::Work;
            }
            SessionType::LongBreak => {
                self.current_session = SessionType::Work;
            }
        }
    }
}

pub fn main() {
    Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            app.manage(Mutex::new(Stopwatch::new()));
            app.manage(CommandExecutor::new());
            app.manage(Mutex::new(AppState::default()));
            APP_HANDLE.set(app.handle().clone()).unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_timer,
            stop_timer,
            get_time,
            reset_timer,
            pause_timer,
            resume_timer,
            on_start,
            on_stop,
            on_pause,
            on_skip_break,
            on_resume,
            on_focus,
            on_blur,
            on_log,
            get_frontmost_application,
        ])
        .on_window_event(|window, event| {
            on_window_event(window, event);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
