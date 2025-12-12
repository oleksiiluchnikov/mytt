use crate::core::domain::{AppState, FlowRating};
use crate::core::history::SessionRecord;
use chrono::{DateTime, Local, Timelike};
use std::time::SystemTime;

pub struct SmartSuggestions;

impl SmartSuggestions {
    pub fn suggest_start_duration(
        state: &AppState,
        recent_sessions: &[SessionRecord],
    ) -> Vec<DurationOption> {
        let current_time = Local::now();
        let suggested = Self::calculate_suggested_duration(current_time, state, recent_sessions);

        // Always provide 3 options: shorter, suggested, longer
        vec![
            DurationOption {
                duration_s: Self::calculate_short_option(suggested),
                label: Self::format_duration(Self::calculate_short_option(suggested)),
                description: "Quick start".to_string(),
                is_suggested: false,
            },
            DurationOption {
                duration_s: suggested,
                label: Self::format_duration(suggested),
                description: Self::get_suggestion_reason(current_time, state, recent_sessions),
                is_suggested: true,
            },
            DurationOption {
                duration_s: Self::calculate_long_option(suggested),
                label: Self::format_duration(Self::calculate_long_option(suggested)),
                description: "Challenge".to_string(),
                is_suggested: false,
            },
        ]
    }

    fn calculate_suggested_duration(
        current_time: DateTime<Local>,
        state: &AppState,
        recent_sessions: &[SessionRecord],
    ) -> u32 {
        // Priority 1: Morning hours - start gentle
        if current_time.hour() >= 6 && current_time.hour() < 10 {
            return 300; // 5 min
        }

        // Priority 2: After long break - ease back in
        if let Some(last_ended) = state.last_session_ended {
            if let Ok(duration) = SystemTime::now().duration_since(last_ended) {
                if duration.as_secs() > 2 * 3600 {
                    // More than 2 hours ago
                    return 600; // 10 min
                }
            }
        }

        // Priority 3: High flow streak - build on momentum
        if state.flow_streak >= 3 {
            let last_duration = recent_sessions
                .last()
                .map(|s| s.actual_duration_s())
                .unwrap_or(900);
            return (last_duration + 300).min(2400); // +5m, cap at 40m
        }

        // Priority 4: Recent distraction pattern - reset to basics
        if recent_sessions.len() >= 2 {
            let recent_distracted = recent_sessions
                .iter()
                .rev()
                .take(2)
                .all(|s| matches!(s.focus_rating, FlowRating::Distracted));

            if recent_distracted {
                return 180; // 3 min - very low barrier
            }
        }

        // Priority 5: First session of the day
        if state.daily_sessions_completed == 0 {
            return 300; // 5 min - fresh start
        }

        // Priority 6: Building focus (Ok or Focused recently)
        if let Some(last_session) = recent_sessions.last() {
            match last_session.focus_rating {
                FlowRating::Flow => return (last_session.actual_duration_s() + 600).min(2400),
                FlowRating::Focused => return (last_session.actual_duration_s() + 300).min(1800),
                FlowRating::Ok => return last_session.planned_duration_s.max(300),
                FlowRating::Distracted => return 300,
            }
        }

        // Default: Progressive start
        300
    }

    fn get_suggestion_reason(
        current_time: DateTime<Local>,
        state: &AppState,
        recent_sessions: &[SessionRecord],
    ) -> String {
        if current_time.hour() >= 6 && current_time.hour() < 10 {
            return "Morning warm-up".to_string();
        }

        if state.flow_streak >= 3 {
            return format!("Building on {} flow streak", state.flow_streak);
        }

        if recent_sessions.len() >= 2 {
            let recent_distracted = recent_sessions
                .iter()
                .rev()
                .take(2)
                .all(|s| matches!(s.focus_rating, FlowRating::Distracted));
            if recent_distracted {
                return "Fresh restart".to_string();
            }
        }

        if state.daily_sessions_completed == 0 {
            return "First session".to_string();
        }

        if let Some(last_session) = recent_sessions.last() {
            match last_session.focus_rating {
                FlowRating::Flow => return "Riding the flow".to_string(),
                FlowRating::Focused => return "Steady progress".to_string(),
                FlowRating::Ok => return "Maintaining pace".to_string(),
                FlowRating::Distracted => return "Gentle restart".to_string(),
            }
        }

        "Suggested".to_string()
    }

    fn calculate_short_option(suggested: u32) -> u32 {
        (suggested / 2).max(120).min(300) // Half of suggested, min 2m, max 5m
    }

    fn calculate_long_option(suggested: u32) -> u32 {
        (suggested * 2).max(600).min(3000) // Double suggested, min 10m, max 50m
    }

    fn format_duration(seconds: u32) -> String {
        let minutes = seconds / 60;
        if minutes < 60 {
            format!("{}m", minutes)
        } else {
            let hours = minutes / 60;
            let remaining_mins = minutes % 60;
            if remaining_mins == 0 {
                format!("{}h", hours)
            } else {
                format!("{}h {}m", hours, remaining_mins)
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DurationOption {
    pub duration_s: u32,
    pub label: String,
    pub description: String,
    pub is_suggested: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::{AppState, FlowRating, Phase};

    #[test]
    fn test_morning_suggestion() {
        let state = AppState::default();
        let sessions = vec![];

        let suggestions = SmartSuggestions::suggest_start_duration(&state, &sessions);

        // Should have 3 options
        assert_eq!(suggestions.len(), 3);

        // Middle option should be suggested
        assert!(suggestions[1].is_suggested);
    }
    #[test]
    fn test_first_session_of_day_gives_five_minutes() {
        let mut state = AppState::default();
        state.daily_sessions_completed = 0;
        let sessions = vec![];
        let suggestions = SmartSuggestions::suggest_start_duration(&state, &sessions);
        assert_eq!(suggestions[1].duration_s, 300);
        assert_eq!(suggestions[1].description, "First session");
    }

    #[test]
    fn test_recent_focused_increases_duration() {
        let mut state = AppState::default();
        state.flow_streak = 1;
        let last_session =
            SessionRecord::new(600, 600_000, FlowRating::Focused, "work".to_string());
        let sessions = vec![last_session];
        let suggestions = SmartSuggestions::suggest_start_duration(&state, &sessions);
        assert_eq!(suggestions[1].duration_s, 900); // 600 + 300, capped at 1800
        assert_eq!(suggestions[1].description, "Steady progress");
    }

    #[test]
    fn test_recent_ok_uses_planned_duration() {
        let mut state = AppState::default();
        let last_session = SessionRecord::new(420, 420_000, FlowRating::Ok, "work".to_string());
        let sessions = vec![last_session];
        let suggestions = SmartSuggestions::suggest_start_duration(&state, &sessions);
        assert_eq!(suggestions[1].duration_s, 420);
        assert_eq!(suggestions[1].description, "Maintaining pace");
    }

    #[test]
    fn test_recent_flow_caps_duration() {
        let mut state = AppState::default();
        let last_session = SessionRecord::new(2000, 2000_000, FlowRating::Flow, "work".to_string());
        let sessions = vec![last_session];
        let suggestions = SmartSuggestions::suggest_start_duration(&state, &sessions);
        // 2000 + 600 = 2600, but should be capped at 2400
        assert_eq!(suggestions[1].duration_s, 2400);
        assert_eq!(suggestions[1].description, "Riding the flow");
    }

    #[test]
    fn test_long_break_eases_back_in() {
        use std::time::{Duration, SystemTime};
        let mut state = AppState::default();
        state.last_session_ended = Some(SystemTime::now() - Duration::from_secs(3 * 3600));
        let sessions = vec![];
        let suggestions = SmartSuggestions::suggest_start_duration(&state, &sessions);
        assert_eq!(suggestions[1].duration_s, 600); // 10 min
        assert_eq!(suggestions[1].description, "Suggested");
    }
}
