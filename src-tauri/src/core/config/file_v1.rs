use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileConfigV1 {
    pub schema_version: u8,

    #[serde(default)]
    pub timer: TimerConfig,

    #[serde(default)]
    pub behavior: BehaviorConfig,
}

impl Default for FileConfigV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            timer: TimerConfig::default(),
            behavior: BehaviorConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimerConfig {
    pub work_default_s: Option<u32>,
    pub short_break_s: Option<u32>,
    pub long_break_s: Option<u32>,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            work_default_s: Some(25 * 60),
            short_break_s: Some(5 * 60),
            long_break_s: Some(15 * 60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorConfig {
    pub theme: Option<String>,
    pub annoying_level: Option<String>,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            theme: Some("dark".to_string()),
            annoying_level: Some("high".to_string()),
        }
    }
}
