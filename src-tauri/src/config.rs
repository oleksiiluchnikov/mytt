use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{read, read_to_string};
use std::path::PathBuf;

const THEMES_DIR: &str = "themes";

lazy_static! {
    // static ref KEY_COMBO: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    pub static ref HOME_DIR: PathBuf = home_dir().unwrap();
    pub static ref CONFIG_DIRS: Vec<PathBuf> = config_dirs();
    pub static ref APP_NAME: String = env!("CARGO_PKG_NAME").to_string();
    pub static ref RUNTIME_DIR: PathBuf = compute_runtime_dir().unwrap();
    pub static ref DATA_DIR: PathBuf = compute_data_dir().unwrap();
    pub static ref CACHE_DIR: PathBuf = compute_cache_dir().unwrap();
}

pub fn script_on_submit() -> Option<PathBuf> {
    let path = xdg_config_home().join("on_submit.sh");
    if path.exists() {
        return Some(path);
    }

    None
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .and_then(|h| if h.is_empty() { None } else { Some(h) })
        .map(PathBuf::from)
}

pub fn xdg_config_home() -> PathBuf {
    match std::env::var_os("XDG_CONFIG_HOME").map(|s| PathBuf::from(s).join(APP_NAME.as_str())) {
        Some(p) => p,
        None => HOME_DIR.join(".config").join(APP_NAME.as_str()),
    }
}

fn config_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    dirs.push(xdg_config_home());

    #[cfg(target_os = "macos")]
    if let Some(d) = std::env::var_os("XDG_CONFIG_DIRS") {
        dirs.extend(std::env::split_paths(&d).map(|s| PathBuf::from(s).join(APP_NAME.as_str())));
    }

    dirs
}

fn compute_cache_dir() -> anyhow::Result<PathBuf> {
    if let Some(runtime) = dirs_next::cache_dir() {
        return Ok(runtime.join(APP_NAME.as_str()));
    }

    Ok(HOME_DIR.join(".local/share/").join(APP_NAME.as_str()))
}

fn compute_data_dir() -> anyhow::Result<PathBuf> {
    if let Some(runtime) = dirs_next::data_dir() {
        return Ok(runtime.join(APP_NAME.as_str()));
    }

    Ok(HOME_DIR.join(".local/share/").join(APP_NAME.as_str()))
}

fn compute_runtime_dir() -> anyhow::Result<PathBuf> {
    if let Some(runtime) = dirs_next::runtime_dir() {
        return Ok(runtime.join(APP_NAME.as_str()));
    }

    Ok(HOME_DIR.join(".local/share/").join(APP_NAME.as_str()))
}
