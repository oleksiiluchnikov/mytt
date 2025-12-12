<script lang="ts">
    import { appStore, timeDisplay } from '../stores/app';
</script>

<div class="timer-display" class:flow={$appStore.flowStreak > 0}>
    <div class="timer-label-container">
        <!-- Direct use of derived store -->
        <h1 class="timer-label">{$timeDisplay}</h1>
    </div>

    <div class="session-info">
        <span class="session-type">{$appStore.phase}</span>
        <div class="flow-info">
            <span class="flow-status">{$appStore.status}</span>
            {#if $appStore.flowStreak > 0}
                <span class="badge">{$appStore.flowStreak}x</span>
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
