use super::file_v1::FileConfigV1;

#[derive(Debug, Clone)]
pub struct ResolvedConfigV1 {
    pub work_default_s: u32,
    pub short_break_s: u32,
    pub long_break_s: u32,
    pub theme: String,
}

impl ResolvedConfigV1 {
    pub fn from_file(file: FileConfigV1) -> Self {
        Self {
            work_default_s: file.timer.work_default_s.unwrap_or(1500).max(60),
            short_break_s: file.timer.short_break_s.unwrap_or(300).max(60),
            long_break_s: file.timer.long_break_s.unwrap_or(900).max(60),
            theme: file.behavior.theme.unwrap_or_else(|| "dark".to_string()),
        }
    }
}
