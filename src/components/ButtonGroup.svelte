<script lang="ts">
import { LABELS, FLOW_STATUS, TIMER_STATUS, DURATION_SUGGESTIONS, FOCUS_STATES, BREAK_TYPE, DURATIONS } from '../constants';
import type { TimerStatus, FlowState, FlowStatus } from '../types/index';
import { timerStore } from '../stores/timer';
import { sessionStore } from '../stores/session';
import { breakStore } from '../stores/break';
import { flowStore } from '../stores/flow';
import Button from './Button.svelte';
import { createEventDispatcher } from 'svelte';
import { invoke } from '@tauri-apps/api/core';

interface FlowButton {
    action: string;
    state: FlowState;
    label: string;
    description: string;
    suggestion: number | null;
}

interface ActionButton {
    action: string;
}

const dispatch = createEventDispatcher<{
    flowState: FlowStatus;
    action: string;
}>();


// Button configurations
$: flowButtons = [
    {
        action: LABELS.FLOW,
        state: FLOW_STATUS.FLOW,
        label: FOCUS_STATES.FLOW.label,
        description: FOCUS_STATES.FLOW.description,
        suggestion: DURATION_SUGGESTIONS.FLOW.MIN_EXTENSION
    },
    {
        action: LABELS.FOCUSED,
        state: FLOW_STATUS.FOCUSED,
        label: FOCUS_STATES.HIGHLY_FOCUSED.label,
        description: FOCUS_STATES.HIGHLY_FOCUSED.description,
        suggestion: DURATION_SUGGESTIONS.FOCUSED.INCREMENT
    },
    {
        action: LABELS.OK,
        state: FLOW_STATUS.OK,
        label: FOCUS_STATES.SOMEWHAT_FOCUSED.label,
        description: FOCUS_STATES.SOMEWHAT_FOCUSED.description,
        suggestion: null
    },
    {
        action: LABELS.DISTRACTED,
        state: FLOW_STATUS.DISTRACTED,
        label: FOCUS_STATES.DISTRACTED.label,
        description: FOCUS_STATES.DISTRACTED.description,
        suggestion: -DURATION_SUGGESTIONS.DISTRACTED.REDUCTION
    }
];

// Event handlers
function handleAction(action: string) {
    dispatch('action', action);

    switch (action) {
        case LABELS.START:
            timerStore.start();
            break;
        case LABELS.RESUME:
            timerStore.resume();
            break;
        case LABELS.PAUSE:
            timerStore.pause();
            break;
        case LABELS.STOP:
            timerStore.stop();
            break;
        case LABELS.SKIP_BREAK:
            breakStore.skip();
            sessionStore.set({
                type: 'work',
                completed: $sessionStore.completed
            });
            timerStore.update(s => ({
                ...s,
                time: {
                    total: s.preferences.workDuration,
                    remaining: s.preferences.workDuration,
                    display: formatTime(s.preferences.workDuration)
                }
            }));
            timerStore.start();
            break;
        case LABELS.LOG:
            invoke('on_log').then(console.log);
            break;
    }
}

function formatTime(time: number): string {
    const minutes = Math.floor(time / 60);
    const seconds = time % 60;
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
}

function handleFlowState(status: FlowStatus) {
    dispatch('flowState', status);

    // Update flow store
    $flowStore.status = status;
    $flowStore.prompt.isActive = false;

    // Calculate next duration based on flow state
    const newDuration = timerStore.nextDuration(
        $timerStore.preferences.workDuration,
        $flowStore.status,
        $flowStore.streak
    );

    // Handle break session if needed
    switch (flowStore.shouldTakeBreak()) {
        case BREAK_TYPE.REQUIRED:
        case BREAK_TYPE.SUGGESTED:
            sessionStore.set({
                type: 'shortBreak',
                completed: $sessionStore.completed + 1,
                lastSessionDuration: $timerStore.preferences.workDuration,
                suggestedNextDuration: newDuration,
            });

            // Set up break duration based on flow status
            const breakDuration = DURATIONS.BREAKS[status.toUpperCase()];
            if (breakDuration > 0) {
                timerStore.update(s => ({
                    ...s,
                    time: {
                        total: breakDuration,
                        remaining: breakDuration,
                        display: formatTime(breakDuration)
                    },
                    progress: {
                        ...s.progress,
                        percentage: (breakDuration / s.time.total) * 100
                    }
                }));
            }
            break;
        default:
            // Continue with work session if no break needed (in flow or focused)
            sessionStore.set({
                type: 'work',
                completed: $sessionStore.completed,
                lastSessionDuration: $timerStore.preferences.workDuration,
                suggestedNextDuration: newDuration,
            });

            timerStore.update(s => ({
                ...s,
                time: {
                    total: newDuration,
                    remaining: newDuration,
                    display: formatTime(newDuration)
                },
                progress: {
                    ...s.progress,
                    percentage: (newDuration / s.time.total) * 100
                }
            }));
    }
}

// Action buttons configurations
const runningButtons: ActionButton[] = $sessionStore.type === 'work'
    ? [{ action: LABELS.PAUSE }, { action: LABELS.STOP }]
    : [{ action: LABELS.PAUSE }, { action: LABELS.STOP }, { action: LABELS.SKIP_BREAK }];

const pausedButtons: ActionButton[] = $sessionStore.type === 'work'
    ? [{ action: LABELS.RESUME }, { action: LABELS.STOP }]
    : [{ action: LABELS.RESUME }, { action: LABELS.STOP }, { action: LABELS.SKIP_BREAK }];

const stoppedButtons: ActionButton[] = $sessionStore.type === 'work'
    ? [{ action: LABELS.START }]
    : [{ action: LABELS.START }, { action: LABELS.SKIP_BREAK }];


// Current buttons state
$: currentButtons = $flowStore.prompt.isActive
    ? flowButtons
    : $timerStore.status === TIMER_STATUS.RUNNING
        ? runningButtons
        : $timerStore.status === TIMER_STATUS.PAUSED
            ? pausedButtons
            : stoppedButtons;
</script>

<div class="buttons-container" role="group" aria-label="Timer controls">
    {#if $flowStore.prompt.isActive}
        <div class="rating-buttons">
            {#each flowButtons as { action, state, label, description, suggestion } (action)}
                <Button
                    {action}
                    label={label}
                    description={``}
                    on:click={() => {
                        if (!state) {
                            return;
                        }
                        handleFlowState(state);
                    }}
                />
            {/each}
        </div>
    {:else}
        <div class="action-buttons">
            {#each currentButtons as { action } (action)}
                <Button
                    {action}
                    label={action}
                    on:click={() => handleAction(action)}
                />
            {/each}
        </div>
    {/if}
    <Button
        action={LABELS.LOG}
        label={LABELS.LOG}
        on:click={() => handleAction(LABELS.LOG)}
    />
</div>

<style>
.buttons-container {
    display: flex;
    flex-direction: column;
    width: 100%;
    gap: var(--spacing-small);
    margin-top: var(--spacing-small);
}

.rating-buttons, .action-buttons {
    display: grid;
    gap: var(--spacing-small);
    justify-content: center;
    grid-template-columns: repeat(auto-fit, minmax(90px, 1fr));
    max-width: 400px;
    width: 100%;
}

/* Flow mode specific layout */
.rating-buttons {
    display: flex;
    flex-direction: column;
    align-items: stretch;
}

@media (min-width: 400px) {
    .action-buttons {
        grid-template-columns: repeat(4, minmax(90px, 1fr));
    }
}

@media (min-width: 600px) {
    .action-buttons {
        grid-template-columns: repeat(5, minmax(90px, 1fr));
    }
}
</style>
