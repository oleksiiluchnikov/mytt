use crate::adapters::hooks::HookName;
use serde::{Deserialize, Serialize};
use std::time::Instant; // We reuse the existing HookName

// --- 1. Core State Enums ---

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FlowRating {
    Distracted,
    Ok,
    Focused,
    Flow,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BreakSuggestion {
    None,
    Optional(u32),  // Seconds
    Suggested(u32), // Seconds
    Required(u32),  // Seconds
}

// --- 2. Runtime Mode ---

#[derive(Debug, Clone, PartialEq)]
// We don't derive Serialize for Mode directly because Instant isn't serializable.
// We will map this to a DTO for the frontend later.
pub enum Mode {
    Idle,
    Running {
        started_at: Instant,
        planned_s: u32,
    },
    Paused {
        elapsed_ms: u128,
        planned_s: u32,
    },
    RatingPrompt {
        last_work_planned_s: u32,
        last_work_actual_ms: u128,
    },
    Decision {
        last_rating: FlowRating,
        next_work_s: u32,
        break_suggestion: BreakSuggestion,
    },
}

// --- 3. The Root Domain State ---

#[derive(Debug, Clone)]
pub struct AppState {
    pub phase: Phase,
    pub mode: Mode,

    // Persistent counters
    pub work_sessions_completed: u32,
    pub flow_streak: u32,

    // Config / Context
    pub frontmost_app: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            phase: Phase::Work,
            mode: Mode::Idle,
            work_sessions_completed: 0,
            flow_streak: 0,
            frontmost_app: None,
        }
    }
}

// --- 4. Inputs (Events) ---

#[derive(Debug, Clone)]
pub enum Event {
    Start { duration_s: Option<u32> },
    Pause,
    Resume,
    Stop,
    Tick { now: Instant },
    TimeUp,
    Rate { rating: FlowRating },
    ChooseBreak { take_break: bool },
    SkipBreak,
    Log,
    AppChanged { name: String },
}

// --- 5. Outputs (Effects) ---

#[derive(Debug, Clone)]
pub enum Effect {
    EmitState,
    Notify {
        title: String,
        body: String,
    },
    RunHook {
        hook: HookName,
        ctx: crate::adapters::hooks::HookContext,
    },
    PersistConfig,
}

use crate::adapters::hooks::HookContext;

impl AppState {
    pub fn to_hook_context(&self, hook: HookName) -> HookContext {
        let (planned, actual) = match &self.mode {
            Mode::Running {
                planned_s,
                started_at,
            } => (
                Some(*planned_s as u64),
                Some(started_at.elapsed().as_millis()),
            ),
            Mode::Paused {
                planned_s,
                elapsed_ms,
            } => (Some(*planned_s as u64), Some(*elapsed_ms)),
            Mode::RatingPrompt {
                last_work_planned_s,
                last_work_actual_ms,
            } => (
                Some(*last_work_planned_s as u64),
                Some(*last_work_actual_ms),
            ),
            _ => (None, None),
        };

        HookContext {
            hook: hook.as_str().to_string(),
            phase: format!("{:?}", self.phase).to_lowercase(),
            rating: None, // Filled by reducer for Rate events
            planned_duration_s: planned,
            actual_duration_ms: actual,
            frontmost_app: self.frontmost_app.clone(),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::domain::*;
    use crate::ipc::dto::AppStateDto;
    use std::time::{Duration, Instant};

    #[test]
    fn test_default_state() {
        let state = AppState::default();
        assert_eq!(state.phase, Phase::Work);
        assert!(matches!(state.mode, Mode::Idle));
    }

    #[test]
    fn test_running_dto_calculation() {
        let start = Instant::now() - Duration::from_secs(10);
        let state = AppState {
            mode: Mode::Running {
                started_at: start,
                planned_s: 60,
            },
            ..Default::default()
        };

        let dto: AppStateDto = (&state).into();
        assert_eq!(dto.status, "running");
        assert_eq!(dto.planned_duration_s, 60);
        // 60s total - 10s elapsed = 50s remaining (50,000ms)
        // Allow tiny delta for test execution time
        assert!(dto.remaining_ms <= 50_000);
        assert!(dto.remaining_ms > 49_000);
    }

    #[test]
    fn test_decision_dto_serialization() {
        let state = AppState {
            mode: Mode::Decision {
                last_rating: FlowRating::Flow,
                next_work_s: 1200,
                break_suggestion: BreakSuggestion::None,
            },
            ..Default::default()
        };

        let dto: AppStateDto = (&state).into();
        let json = serde_json::to_string(&dto).unwrap();

        assert!(json.contains("\"status\":\"decision\""));
        assert!(json.contains("\"lastRating\":\"flow\""));
        assert!(json.contains("\"breakSuggestion\":\"none\""));
    }
}
