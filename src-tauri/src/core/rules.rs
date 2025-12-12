use crate::core::config::resolved_v1::ResolvedConfigV1;
use crate::core::domain::{BreakSuggestion, FlowRating};

pub struct NextSessionSpecs {
    pub next_work_s: u32,
    pub break_suggestion: BreakSuggestion,
}

pub fn calculate_next_session(
    rating: FlowRating,
    last_work_s: u32,
    streak: u32,
    config: &ResolvedConfigV1,
) -> NextSessionSpecs {
    // 1. Calculate next work duration
    // Simple progressive logic:
    // Distracted -> decrease or reset to min
    // Ok -> keep same or small increase
    // Focused -> increase
    // Flow -> large increase

    let delta = match rating {
        FlowRating::Distracted => -300, // -5 min
        FlowRating::Ok => 0,            // Maintain
        FlowRating::Focused => 300,     // +5 min
        FlowRating::Flow => 600,        // +10 min
    };

    // Apply delta, clamping to [120s, work_default * 2] or some config max
    let new_duration = (last_work_s as i32 + delta)
        .max(120) // Minimum 2m
        .min(3600 * 2) // Max 2h hard cap for now
        as u32;

    // 2. Calculate break suggestion
    // Flow -> None (skip)
    // Focused -> Optional Short
    // Ok -> Suggested Short
    // Distracted -> Suggested Long or Short

    let suggestion = match rating {
        FlowRating::Flow => {
            if streak >= 4 {
                BreakSuggestion::Suggested(config.long_break_s)
            } else {
                BreakSuggestion::None
            }
        }
        FlowRating::Focused => BreakSuggestion::Optional(config.short_break_s),
        FlowRating::Ok => BreakSuggestion::Suggested(config.short_break_s),
        FlowRating::Distracted => BreakSuggestion::Required(config.short_break_s),
    };

    NextSessionSpecs {
        next_work_s: new_duration,
        break_suggestion: suggestion,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::file_v1::FileConfigV1;

    fn mock_config() -> ResolvedConfigV1 {
        ResolvedConfigV1::from_file(FileConfigV1::default())
    }

    #[test]
    fn test_flow_increases_duration() {
        let config = mock_config();
        let specs = calculate_next_session(FlowRating::Flow, 1500, 0, &config);
        assert!(specs.next_work_s > 1500);
        assert!(matches!(specs.break_suggestion, BreakSuggestion::None));
    }

    #[test]
    fn test_distracted_decreases_duration() {
        let config = mock_config();
        let specs = calculate_next_session(FlowRating::Distracted, 1500, 0, &config);
        assert!(specs.next_work_s < 1500);
        assert!(matches!(
            specs.break_suggestion,
            BreakSuggestion::Required(_)
        ));
    }
}
