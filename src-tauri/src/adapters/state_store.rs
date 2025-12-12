use crate::core::domain::{AppState, Mode, Phase};
use crate::notify_on_error;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

// DTO for the file on disk (excludes runtime-only fields like 'mode')
#[derive(Debug, Serialize, Deserialize)]
struct StateFile {
    work_sessions_completed: u32,
    flow_streak: u32,
    daily_sessions_completed: u32,
    daily_goal: u32,
    last_date: String,
    // We optionally persist phase to resume roughly where we left off
    phase: Phase,
}

impl Default for StateFile {
    fn default() -> Self {
        // Create a temporary default AppState to borrow defaults from
        let def = AppState::default();
        Self {
            work_sessions_completed: def.work_sessions_completed,
            flow_streak: def.flow_streak,
            daily_sessions_completed: def.daily_sessions_completed,
            daily_goal: def.daily_goal,
            last_date: def.last_date,
            phase: def.phase,
        }
    }
}

pub struct StateStore {
    pub state_path: PathBuf,
}

impl StateStore {
    pub fn new() -> Self {
        let app_name = env!("CARGO_PKG_NAME");
        let config_dir = xdg_config_home(app_name);
        let state_path = config_dir.join("state.json");

        Self { state_path }
    }

    pub fn load(&self) -> AppState {
        let file_data = if self.state_path.exists() {
            match fs::read_to_string(&self.state_path) {
                Ok(content) => serde_json::from_str::<StateFile>(&content).unwrap_or_else(|e| {
                    notify_on_error!("State parse error: {}, resetting state", e);
                    StateFile::default()
                }),
                Err(e) => {
                    notify_on_error!("State read error: {}", e);
                    StateFile::default()
                }
            }
        } else {
            StateFile::default()
        };

        // Map File DTO -> Domain Entity
        AppState {
            phase: file_data.phase,
            mode: Mode::Idle, // Always reset mode to Idle on cold boot
            work_sessions_completed: file_data.work_sessions_completed,
            flow_streak: file_data.flow_streak,
            daily_sessions_completed: file_data.daily_sessions_completed,
            daily_goal: file_data.daily_goal,
            last_date: file_data.last_date,
            last_session_ended: None,
            frontmost_app: None, // Will be populated by on_focus event
        }
    }

    pub fn save(&self, state: &AppState) {
        let file_data = StateFile {
            work_sessions_completed: state.work_sessions_completed,
            flow_streak: state.flow_streak,
            daily_sessions_completed: state.daily_sessions_completed,
            daily_goal: state.daily_goal,
            last_date: state.last_date.clone(),
            phase: state.phase,
        };

        if let Some(parent) = self.state_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(&file_data) {
            if let Err(e) = fs::write(&self.state_path, json) {
                eprintln!("Failed to save state: {}", e);
            }
        }
    }
}

// Helper to resolve config path (same as in config_store.rs)
fn xdg_config_home(app_name: &str) -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(|s| PathBuf::from(s).join(app_name))
        .or_else(|| dirs_next::home_dir().map(|h| h.join(".config").join(app_name)))
        .unwrap_or_else(|| PathBuf::from(".").join(app_name))
}
