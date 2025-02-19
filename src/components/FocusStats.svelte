<script>
    import { timer } from '../stores/timer';
    import { formattedTime } from '../stores/timerDerived';
    import { SESSION_TYPES } from '../constants';
</script>

<div class="timer-display" class:flow={$timer.isFlowMode}>
    <h1 class="timer-label">{$formattedTime}</h1>
    <div class="session-info">
        <span class="session-type">
            {$timer.sessionType === SESSION_TYPES.WORK ? 'Work Session' :
             $timer.sessionType === SESSION_TYPES.SHORT_BREAK ? 'Short Break' : 'Long Break'}
        </span>
        {#if $timer.isFlowMode}
            <span class="flow-badge">Flow</span>
        {/if}
    </div>
    {#if $timer.suggestedNextDuration && $timer.suggestedNextDuration !== $timer.workDuration}
        <div class="suggestion">
            Suggested next duration: {Math.round($timer.suggestedNextDuration / 60)} minutes
        </div>
    {/if}
</div>

<style>
    .timer-display {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: var(--spacing-small);
        padding: var(--spacing-medium);
    }

    .timer-label {
        font-size: 2.5rem;
        font-weight: 700;
        margin: 0;
    }

    .session-info {
        display: flex;
        align-items: center;
        gap: var(--spacing-small);
    }

    .session-type {
        font-size: 1em;
        opacity: 0.8;
    }

    .flow-badge {
        background: var(--flow-color, #4CAF50);
        padding: 2px 8px;
        border-radius: 12px;
        font-size: 0.8em;
        font-weight: 500;
    }

    .suggestion {
        font-size: 0.9em;
        opacity: 0.7;
        margin-top: var(--spacing-small);
    }

    .flow .timer-label {
        color: var(--flow-color, #4CAF50);
    }
</style>
