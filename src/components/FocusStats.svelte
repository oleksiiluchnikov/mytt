<script lang="ts">
import { timerStore } from '../stores/timer';
import { sessionStore } from '../stores/session';
import { flowStore } from '../stores/flow';
import { FLOW_STATUS } from '../constants';
// camelCase to label
function camelCaseToLabel(str: string): string {
    return str.replace(/([A-Z])/g, ' $1').toLowerCase().trim();
}
</script>

<div class="timer-display" class:flow={$flowStore.status === FLOW_STATUS.FLOW}>
    <div class="timer-label-container">
        <h1 class="timer-label minutes">{$timerStore.time.display.slice(0, 2)}</h1>
        <h1 class="timer-label-separator">:</h1>
        <h1 class="timer-label seconds">{$timerStore.time.display.slice(3, 5)}</h1>
    </div>
    <div class="session-info">
        <span class="session-type" style:color={$sessionStore.type.toLowerCase().includes('break') ? 'var(--break-text-color)' : 'inherit'}>{camelCaseToLabel($sessionStore.type)}</span>
        <div class="flow-info">
        <span class="flow-status">{$flowStore.status}</span>
        {#if $flowStore.status === FLOW_STATUS.FLOW}
            <span class="badge">{$flowStore.streak}</span>
        {/if}
        </div>
    </div>
</div>

<style>
.timer-display {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-small);
}

.timer-label-container,
.flow-info {
    display: flex;
    align-items: center;
    gap: var(--spacing-small);
}

.timer-label-container {
    flex-direction: row;
    gap: 0.1em;
}

.timer-label {
    font-size: 2.6rem;
    font-weight: 900;
    margin: 0;
    text-align: center;
    font-variant-numeric: tabular-nums;
}

.seconds.timer-label {
    align-items: flex-end;
}

.minutes.timer-label {
    align-items: flex-start;
}

.timer-label-separator {
    font-size: 2.6rem;
    font-weight: 900;
    margin: 0;
    line-height: 1;
    width: 0.3em;
    text-align: center;
    display: inline-block;
}

.session-info {
    display: flex;
    align-items: center;
    margin: auto;
    width: 100%;
    height: var(--session-info-height);
    justify-content: space-between;
}

.session-type,
.flow-status {
    color: var(--text-color);
    font-size: var(--font-size-small);
    opacity: 0.8;
}

.session-type {
    align-self: flex-start;
    font-weight: 500;
}

.flow-status {
    align-self: flex-end;
}

.badge {
    background: var(--flow-color, #4CAF50);
    padding: 0px 4px;
    border-radius: calc(var(--border-radius) / 2);
    font-size: var(--font-size-small);
    font-weight: 500;
}

</style>
