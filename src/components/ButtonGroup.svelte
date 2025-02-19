<script lang="ts">
import { LABELS, FLOW_STATE, TIMER_STATUS, DURATION_SUGGESTIONS, FOCUS_STATES } from '../constants';
import type { TimerStatus, FlowState } from '../types/index';
import { timer } from '../stores/timer';
import { isWorkSession } from '../stores/timerDerived';
import TimerButton from './TimerButton.svelte';
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
    flowState: FlowState;
    action: string;
}>();

// Button configurations
$: flowButtons = [
    {
        action: LABELS.FLOW,
        state: FLOW_STATE.FLOW,
        label: FOCUS_STATES.FLOW.label,
        description: FOCUS_STATES.FLOW.description,
        suggestion: DURATION_SUGGESTIONS.FLOW.MIN_EXTENSION
    },
    {
        action: LABELS.FOCUSED,
        state: FLOW_STATE.FOCUSED,
        label: FOCUS_STATES.HIGHLY_FOCUSED.label,
        description: FOCUS_STATES.HIGHLY_FOCUSED.description,
        suggestion: DURATION_SUGGESTIONS.FOCUSED.INCREMENT
    },
    {
        action: LABELS.OK,
        state: FLOW_STATE.OK,
        label: FOCUS_STATES.SOMEWHAT_FOCUSED.label,
        description: FOCUS_STATES.SOMEWHAT_FOCUSED.description,
        suggestion: null
    },
    {
        action: LABELS.DISTRACTED,
        state: FLOW_STATE.DISTRACTED,
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
        case LABELS.RESUME:
            timer.start();
            break;
        case LABELS.PAUSE:
            timer.pause();
            break;
        case LABELS.STOP:
            timer.reset();
            break;
        case LABELS.SKIP_BREAK:
            timer.skipBreak();
            break;
        case LABELS.LOG:
            invoke('on_log').then(console.log);
            break;
    }
}

function handleFlowState(state: FlowState) {
    dispatch('flowState', state);
    timer.setFlowState(state);
}

// Action buttons configurations
const runningButtons: ActionButton[] = [
    { action: LABELS.PAUSE },
    { action: LABELS.STOP }
];

const pausedButtons: ActionButton[] = [
    { action: LABELS.RESUME },
    { action: LABELS.STOP }
];

const stoppedButtons: ActionButton[] = $isWorkSession
    ? [{ action: LABELS.START }]
    : $timer.isFlowMode
        ? [{ action: LABELS.START }]  // Don't show skip during flow
        : [{ action: LABELS.START }, { action: LABELS.SKIP_BREAK }];  // Show skip otherwise


// Current buttons state
$: currentButtons = $timer.showFlowPrompt
    ? flowButtons
    : $timer.status === TIMER_STATUS.RUNNING
        ? runningButtons
        : $timer.status === TIMER_STATUS.PAUSED
            ? pausedButtons
            : stoppedButtons;
</script>

<div class="buttons-container" role="group" aria-label="Timer controls">
    {#if $timer.showFlowPrompt}
        <div class="rating-prompt">
            <!-- <p>How was your focus this session?</p> -->
            <div class="flow-stats">
                <span>Flow streak: {$timer.flowStreak}</span>
            </div>
        </div>
        <div class="rating-buttons">
            {#each flowButtons as { action, state, label, description, suggestion } (action)}
                <TimerButton
                    {action}
                    label={label}
                    description={description}
                    on:click={() => {
                        if (state) {
                            handleFlowState(state);
                        }
                    }}
                />
            {/each}
        </div>
    {:else}
        <div class="action-buttons">
            {#each currentButtons as { action } (action)}
                <TimerButton
                    {action}
                    label={action}
                    on:click={() => handleAction(action)}
                />
            {/each}
        </div>
    {/if}

    <TimerButton
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

    .rating-prompt {
        text-align: center;
    }

    .rating-buttons, .action-buttons {
        display: grid;
        gap: var(--spacing-small);
        justify-content: center;
        width: 100%;
    }

    .flow-stats {
        margin-top: var(--spacing-small);
        font-size: 0.9em;
        opacity: 0.8;
    }

    /* Flow mode specific layout */
    .rating-buttons {
        grid-template-columns: repeat(2, 1fr);
        grid-template-rows: repeat(2, 1fr);
    }

    /* Action buttons layout */
    .action-buttons {
        grid-template-columns: repeat(auto-fit, minmax(90px, 1fr));
        grid-auto-rows: 44px;
    }
    .rating-buttons, .action-buttons {
        display: grid;
        gap: var(--spacing-small);
        justify-content: center;
        grid-template-columns: repeat(2, 1fr);
        width: 100%;
    }

    .flow-stats {
        margin-top: var(--spacing-small);
        font-size: 0.9em;
        opacity: 0.8;
    }

    /* For flow mode, use a 2x2 grid */
    .rating-buttons {
        grid-template-rows: repeat(2, 1fr);
    }
    .rating-buttons, .action-buttons {
        display: grid;
        gap: var(--spacing-small);
        justify-content: center;
        grid-template-columns: repeat(2, 1fr);
        width: 100%;
    }

    .flow-stats {
        margin-top: var(--spacing-small);
        font-size: 0.9em;
        opacity: 0.8;
    }

    .rating-buttons, .action-buttons {
        display: grid;
        gap: var(--spacing-small);
        justify-content: center;
        grid-template-columns: repeat(auto-fit, minmax(90px, 1fr));
        max-width: 400px;
        width: 100%;
    }

    @media (min-width: 400px) {
        .rating-buttons, .action-buttons {
            grid-template-columns: repeat(4, minmax(90px, 1fr));
        }
    }

    @media (min-width: 600px) {
        .rating-buttons, .action-buttons {
            grid-template-columns: repeat(5, minmax(90px, 1fr));
        }
    }

</style>
