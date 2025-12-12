use crate::core::config::file_v1::FileConfigV1;
use crate::notify_on_error;
use std::env;
use std::fs;
use std::path::PathBuf;

pub struct ConfigStore {
    pub config_path: PathBuf,
    pub scripts_dir: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Self {
        let app_name = env!("CARGO_PKG_NAME");
        let config_dir = xdg_config_home(app_name);

        // Ensure standard paths
        let config_path = config_dir.join("config.json");
        let scripts_dir = config_dir.join("scripts");

        Self {
            config_path,
            scripts_dir,
        }
    }

    pub fn load(&self) -> FileConfigV1 {
        if !self.config_path.exists() {
            return FileConfigV1::default();
        }

        match fs::read_to_string(&self.config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                notify_on_error!("Config parse error: {}, using defaults", e);
                FileConfigV1::default()
            }),
            Err(e) => {
                notify_on_error!("Config read error: {}", e);
                FileConfigV1::default()
            }
        }
    }

    pub fn save(&self, config: FileConfigV1) -> Result<(), String> {
        // 1. Ensure directory exists (Fixes your previous crash risk)
        if let Some(parent) = self.config_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(format!("Failed to create config dir: {}", e));
            }
        }

        // 2. Write file
        let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;

        fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }
}

// --- Internal Helper (Ported from your code, simplified) ---
fn xdg_config_home(app_name: &str) -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(|s| PathBuf::from(s).join(app_name))
        .or_else(|| dirs_next::home_dir().map(|h| h.join(".config").join(app_name)))
        .unwrap_or_else(|| PathBuf::from(".").join(app_name)) // Fallback to CWD
}
