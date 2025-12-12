use crate::core::domain::{AppState, BreakSuggestion, Event, FlowRating, Mode, Phase};
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppStateDto {
    pub status: String, // "idle", "running", "paused", "ratingPrompt", "decision"
    pub phase: Phase,
    pub planned_duration_s: u32,
    pub remaining_ms: u128,
    pub work_sessions_completed: u32,
    pub flow_streak: u32,

    pub daily_sessions_completed: u32,
    pub daily_goal: u32,

    // Optional context fields
    pub last_rating: Option<FlowRating>,
    pub next_work_s: Option<u32>,
    pub break_suggestion: Option<BreakSuggestion>,
}

impl From<&AppState> for AppStateDto {
    fn from(state: &AppState) -> Self {
        let (status, planned, remaining, rating, next, suggestion) = match &state.mode {
            Mode::Idle => ("idle", 0, 0, None, None, None),
            Mode::Running {
                started_at,
                planned_s,
            } => {
                let elapsed = started_at.elapsed().as_millis();
                let total_ms = *planned_s as u128 * 1000;
                let rem = total_ms.saturating_sub(elapsed);
                ("running", *planned_s, rem, None, None, None)
            }
            Mode::Paused {
                elapsed_ms,
                planned_s,
            } => {
                let total_ms = *planned_s as u128 * 1000;
                let rem = total_ms.saturating_sub(*elapsed_ms);
                ("paused", *planned_s, rem, None, None, None)
            }
            Mode::RatingPrompt { .. } => ("ratingPrompt", 0, 0, None, None, None),
            Mode::Decision {
                last_rating,
                next_work_s,
                break_suggestion,
            } => (
                "decision",
                0,
                0,
                Some(*last_rating),
                Some(*next_work_s),
                Some(*break_suggestion),
            ),
        };

        Self {
            status: status.to_string(),
            phase: state.phase,
            planned_duration_s: planned,
            remaining_ms: remaining,
            work_sessions_completed: state.work_sessions_completed,
            flow_streak: state.flow_streak,
            last_rating: rating,
            next_work_s: next,
            break_suggestion: suggestion,

            daily_sessions_completed: state.daily_sessions_completed,
            daily_goal: state.daily_goal,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum IntentDto {
    Start {
        #[serde(rename = "durationS")] // Fixes potential future error here too
        duration_s: Option<u32>,
    },
    Pause,
    Resume,
    Stop,
    Rate {
        rating: FlowRating,
    },
    ChooseBreak {
        #[serde(rename = "takeBreak")] // <--- THE FIX
        take_break: bool,
    },
    SkipBreak,
    Log,
}

impl Into<Event> for IntentDto {
    fn into(self) -> Event {
        match self {
            IntentDto::Start { duration_s } => Event::Start { duration_s },
            IntentDto::Pause => Event::Pause,
            IntentDto::Resume => Event::Resume,
            IntentDto::Stop => Event::Stop,
            IntentDto::Rate { rating } => Event::Rate { rating },
            IntentDto::ChooseBreak { take_break } => Event::ChooseBreak { take_break },
            IntentDto::SkipBreak => Event::SkipBreak,
            IntentDto::Log => Event::Log,
        }
    }
}
