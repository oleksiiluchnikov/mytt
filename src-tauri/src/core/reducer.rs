use crate::adapters::hooks::HookName;
use crate::core::config::resolved_v1::ResolvedConfigV1;
use crate::core::domain::*;
use crate::core::rules::calculate_next_session;
use std::time::Instant;

pub fn reduce(
    mut state: AppState,
    event: Event,
    config: &ResolvedConfigV1,
) -> (AppState, Vec<Effect>) {
    let mut effects = Vec::new();

    match event {
        // --- START ---
        Event::Start { duration_s } => {
            let duration = duration_s.unwrap_or_else(|| match state.phase {
                Phase::Work => config.work_default_s,
                Phase::ShortBreak => config.short_break_s,
                Phase::LongBreak => config.long_break_s,
            });

            state.mode = Mode::Running {
                started_at: Instant::now(),
                planned_s: duration,
            };

            // Effect: Run hook based on phase
            let hook = match state.phase {
                Phase::Work => HookName::OnStart,
                _ => HookName::OnSkipBreak, // Or a generic OnBreakStart if we add it later
            };

            effects.push(Effect::RunHook {
                hook,
                ctx: state.to_hook_context(hook),
            });
            effects.push(Effect::EmitState);
        }

        // --- PAUSE ---
        Event::Pause => {
            if let Mode::Running {
                started_at,
                planned_s,
            } = state.mode
            {
                let elapsed = started_at.elapsed().as_millis();
                state.mode = Mode::Paused {
                    elapsed_ms: elapsed,
                    planned_s,
                };

                effects.push(Effect::RunHook {
                    hook: HookName::OnPause,
                    ctx: state.to_hook_context(HookName::OnPause),
                });
                effects.push(Effect::EmitState);
            }
        }

        // --- RESUME ---
        Event::Resume => {
            if let Mode::Paused {
                elapsed_ms,
                planned_s,
            } = state.mode
            {
                // To resume, we reset start time back by the elapsed amount
                // started_at = now - elapsed
                // Note: Domain logic usually shouldn't depend on "now" for calculation if passed in,
                // but for Instant math we generate a new anchor.
                let now = Instant::now();
                let started_at = now
                    .checked_sub(std::time::Duration::from_millis(elapsed_ms as u64))
                    .unwrap_or(now);

                state.mode = Mode::Running {
                    started_at,
                    planned_s,
                };

                effects.push(Effect::RunHook {
                    hook: HookName::OnResume,
                    ctx: state.to_hook_context(HookName::OnResume),
                });
                effects.push(Effect::EmitState);
            }
        }

        // --- STOP ---
        Event::Stop => {
            // "Stop" in legacy behavior means "Abort/Reset to Idle"
            let hook = HookName::OnStop;
            let ctx = state.to_hook_context(hook);

            state.mode = Mode::Idle;
            // Legacy behavior: stopping resets to Work phase?
            // For now, let's keep phase but reset mode.

            effects.push(Effect::RunHook { hook, ctx });
            effects.push(Effect::EmitState);
        }

        // --- TICK (System Event) ---
        Event::Tick { now: _ } => {
            if let Mode::Running {
                started_at,
                planned_s,
            } = state.mode
            {
                let elapsed = started_at.elapsed().as_millis();
                let total = planned_s as u128 * 1000;

                if elapsed >= total {
                    // ... (Your existing Time Up logic) ...

                    if state.phase == Phase::Work {
                        effects.push(Effect::Notify {
                            title: "Work Session Complete".to_string(),
                            body: "How was your focus?".to_string(),
                        });
                        state.mode = Mode::RatingPrompt {
                            last_work_planned_s: planned_s,
                            last_work_actual_ms: elapsed,
                        };
                        state.work_sessions_completed += 1;
                    } else {
                        effects.push(Effect::Notify {
                            title: "Break Over".to_string(),
                            body: "Ready to focus?".to_string(),
                        });
                        state.mode = Mode::Idle;
                        state.phase = Phase::Work;
                    }
                    effects.push(Effect::EmitState);
                } else {
                    // --- MISSING PART: UPDATE UI WHILE RUNNING ---
                    // We need to emit state so the frontend gets the new 'remaining_ms'

                    // Optimization: You can throttle this to 1Hz if you only care about seconds,
                    // but for a smooth progress bar, emitting every 100ms (10Hz) is fine locally.
                    effects.push(Effect::EmitState);
                }
            }
        }

        // --- LOG ---
        Event::Log => {
            let hook = HookName::OnLog;
            let ctx = state.to_hook_context(hook);

            // Reset logic similar to Stop, but specifically for logging
            state.mode = Mode::Idle;
            state.phase = Phase::Work; // Logging usually implies "done with task, ready for next"

            effects.push(Effect::RunHook { hook, ctx });
            effects.push(Effect::EmitState);
        }

        // --- RATE ---
        Event::Rate { rating } => {
            if let Mode::RatingPrompt {
                last_work_planned_s,
                ..
            } = state.mode
            {
                // Update streak based on rating
                if rating == FlowRating::Flow {
                    state.flow_streak += 1;
                } else {
                    state.flow_streak = 0;
                }

                // Calculate next specs
                let specs =
                    calculate_next_session(rating, last_work_planned_s, state.flow_streak, config);

                // Transition to Decision
                state.mode = Mode::Decision {
                    last_rating: rating,
                    next_work_s: specs.next_work_s,
                    break_suggestion: specs.break_suggestion,
                };

                effects.push(Effect::RunHook {
                    hook: HookName::OnRated,
                    ctx: state.to_hook_context(HookName::OnRated),
                });
                effects.push(Effect::EmitState);
            }
        }

        // --- CHOOSE BREAK ---
        Event::ChooseBreak { take_break } => {
            if let Mode::Decision {
                next_work_s,
                break_suggestion,
                ..
            } = state.mode
            {
                if take_break {
                    // Determine break duration from suggestion or config default
                    let duration = match break_suggestion {
                        BreakSuggestion::None => config.short_break_s,
                        BreakSuggestion::Optional(s)
                        | BreakSuggestion::Suggested(s)
                        | BreakSuggestion::Required(s) => s,
                    };

                    // Check if it's a "long" break (heuristic or explicit state tracking)
                    // For now, simple mapping:
                    state.phase = if duration >= config.long_break_s {
                        Phase::LongBreak
                    } else {
                        Phase::ShortBreak
                    };

                    // Go to Idle (Break) - wait for user to start it, OR auto-start?
                    // "Progressive" usually implies user starts it.
                    state.mode = Mode::Idle;
                } else {
                    // Skip break -> Go to Work Idle with new duration
                    state.phase = Phase::Work;
                    // We might want to store next_work_s as a "pending duration"
                    // but since Mode::Idle is stateless, we assume the UI will send
                    // Start { duration_s: next_work_s }.
                    // BETTER: Store it in a transient field or make Idle carry it?
                    // For now, let's reset to Idle.
                    // The UI has "next_work_s" from the Decision state,
                    // so it can pass it back in Start.
                    state.mode = Mode::Idle;
                }
                effects.push(Effect::EmitState);
            }
        }

        // --- APP CHANGED (Focus/Blur) ---
        Event::AppChanged { name } => {
            // Update state
            state.frontmost_app = Some(name.clone());

            // Effect: Run OnFocus hook
            // (We could also run OnBlur for the previous app if we tracked it,
            // but simple OnFocus is usually what users want)
            effects.push(Effect::RunHook {
                hook: HookName::OnFocus,
                ctx: state.to_hook_context(HookName::OnFocus),
            });

            effects.push(Effect::EmitState);
        }

        // Placeholder for future events
        _ => {}
    }

    (state, effects)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::hooks::HookName;
    use crate::core::config::file_v1::{FileConfigV1, TimerConfig};

    fn mock_config() -> ResolvedConfigV1 {
        ResolvedConfigV1::from_file(FileConfigV1 {
            timer: TimerConfig {
                work_default_s: Some(100),
                short_break_s: Some(50),
                long_break_s: Some(200),
            },
            ..Default::default()
        })
    }

    #[test]
    fn test_start_transition() {
        let state = AppState::default();
        let config = mock_config();

        let (new_state, effects) = reduce(state, Event::Start { duration_s: None }, &config);

        assert!(matches!(new_state.mode, Mode::Running { .. }));
        assert_eq!(effects.len(), 2); // RunHook + EmitState

        if let Effect::RunHook { hook, .. } = &effects[0] {
            assert!(matches!(hook, HookName::OnStart));
        } else {
            panic!("Expected RunHook effect");
        }
    }

    #[test]
    fn test_pause_resume() {
        let config = mock_config();
        let mut state = AppState::default();

        // Start
        let (s1, _) = reduce(state, Event::Start { duration_s: None }, &config);
        state = s1;

        // Pause
        let (s2, effects) = reduce(state, Event::Pause, &config);
        state = s2;

        assert!(matches!(state.mode, Mode::Paused { .. }));
        assert!(effects.iter().any(|e| matches!(
            e,
            Effect::RunHook {
                hook: HookName::OnPause,
                ..
            }
        )));

        // Resume
        let (s3, effects) = reduce(state, Event::Resume, &config);
        assert!(matches!(s3.mode, Mode::Running { .. }));
        assert!(effects.iter().any(|e| matches!(
            e,
            Effect::RunHook {
                hook: HookName::OnResume,
                ..
            }
        )));
    }

    #[test]
    fn test_stop_transition() {
        let config = mock_config();
        let mut state = AppState::default();
        let (s1, _) = reduce(state, Event::Start { duration_s: None }, &config);
        state = s1;

        let (final_state, effects) = reduce(state, Event::Stop, &config);

        assert!(matches!(final_state.mode, Mode::Idle));
        // Verify OnStop hook is called
        let has_stop_hook = effects.iter().any(|e| match e {
            Effect::RunHook { hook, .. } => matches!(hook, HookName::OnStop),
            _ => false,
        });
        assert!(has_stop_hook);
    }

    #[test]
    fn test_log_transition() {
        let config = mock_config();
        let state = AppState {
            mode: Mode::Paused {
                elapsed_ms: 5000,
                planned_s: 100,
            },
            ..Default::default()
        };

        let (new_state, effects) = reduce(state, Event::Log, &config);

        assert!(matches!(new_state.mode, Mode::Idle));
        assert_eq!(new_state.phase, Phase::Work); // Should reset to Work

        let has_log_hook = effects.iter().any(|e| match e {
            Effect::RunHook { hook, .. } => matches!(hook, HookName::OnLog),
            _ => false,
        });
        assert!(has_log_hook);
    }
}
