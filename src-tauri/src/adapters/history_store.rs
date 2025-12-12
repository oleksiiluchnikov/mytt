use crate::core::history::SessionRecord;
use crate::notify_on_error;
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct HistoryStore {
    pub history_path: PathBuf,
}

impl HistoryStore {
    pub fn new() -> Self {
        let app_name = env!("CARGO_PKG_NAME");
        let config_dir = xdg_config_home(app_name);
        let history_path = config_dir.join("history.json");

        Self { history_path }
    }

    pub fn load(&self) -> Vec<SessionRecord> {
        if !self.history_path.exists() {
            return Vec::new();
        }

        match fs::read_to_string(&self.history_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                notify_on_error!("History parse error: {}, using empty history", e);
                Vec::new()
            }),
            Err(e) => {
                notify_on_error!("History read error: {}", e);
                Vec::new()
            }
        }
    }

    pub fn save(&self, history: &[SessionRecord]) {
        if let Some(parent) = self.history_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(json) = serde_json::to_string_pretty(history) {
            if let Err(e) = fs::write(&self.history_path, json) {
                eprintln!("Failed to save history: {}", e);
            }
        }
    }

    pub fn append(&self, record: SessionRecord) {
        let mut history = self.load();
        history.push(record);

        // Keep only last 100 sessions to prevent unbounded growth
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }

        self.save(&history);
    }

    pub fn get_recent(&self, count: usize) -> Vec<SessionRecord> {
        let history = self.load();
        history.into_iter().rev().take(count).rev().collect()
    }
}

fn xdg_config_home(app_name: &str) -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(|s| PathBuf::from(s).join(app_name))
        .or_else(|| dirs_next::home_dir().map(|h| h.join(".config").join(app_name)))
        .unwrap_or_else(|| PathBuf::from(".").join(app_name))
}
