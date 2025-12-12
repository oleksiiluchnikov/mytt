use crate::notify_on_error;
use serde::Serialize;
use std::process::Command;
use std::thread;

#[derive(Debug, Clone, Copy)]
pub enum HookName {
    OnStart,
    OnStop,
    OnLog,
    OnPause,
    OnResume,
    OnSkipBreak,
    OnRated,
    OnFocus,
    OnBlur,
    OnWindowEvent,
}

impl HookName {
    pub fn as_str(&self) -> &'static str {
        match self {
            HookName::OnStart => "on_start",
            HookName::OnStop => "on_stop",
            HookName::OnLog => "on_log",
            HookName::OnPause => "on_pause",
            HookName::OnResume => "on_resume",
            HookName::OnSkipBreak => "on_skip_break",
            HookName::OnRated => "on_rated",
            HookName::OnFocus => "on_focus",
            HookName::OnBlur => "on_blur",
            HookName::OnWindowEvent => "on_window_event",
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct HookContext {
    pub hook: String,
    pub phase: String,
    pub rating: Option<String>,
    pub planned_duration_s: Option<u64>,
    pub actual_duration_ms: Option<u128>,
    pub frontmost_app: Option<String>,
    pub timestamp_ms: u128,
}

pub struct HookRunner {
    sender: std::sync::mpsc::Sender<(std::path::PathBuf, HookContext)>,
}

impl HookRunner {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();

        thread::spawn(move || {
            for (script_path, ctx) in receiver {
                let ctx_json = match serde_json::to_string(&ctx) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Failed to serialize HookContext: {}", e);
                        continue;
                    }
                };

                let result = Command::new(&script_path).arg(&ctx_json).output();

                if let Err(err) = result {
                    eprintln!("Hook execution failed: {}", err);
                }
            }
        });

        Self { sender }
    }

    pub fn run_hook(&self, script_path: std::path::PathBuf, ctx: HookContext) {
        if !script_path.exists() {
            notify_on_error!(
                "Hook script not found: {} ({:?})",
                script_path.display(),
                ctx
            );
            return;
        }

        if let Err(e) = self.sender.send((script_path, ctx)) {
            notify_on_error!("Failed to queue hook execution: {:?}", e);
        }
    }
}
