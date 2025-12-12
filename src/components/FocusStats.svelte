<script lang="ts">
    import { appStore, timeDisplay } from '../stores/app';

    // Helper to format phase nicely (e.g. "shortBreak" -> "Short Break")
    $: phaseDisplay = $appStore.phase.replace(/([A-Z])/g, ' $1').toLowerCase();
</script>

<div class="timer-display" class:flow={$appStore.flowStreak > 0}>
    <!-- Time -->
    <div class="timer-label-container">
        <h1 class="timer-label">{$timeDisplay}</h1>
    </div>

    <!-- Daily Progress (Centered, small) -->
    <div class="daily-progress">
        {#if $appStore.dailySessionsCompleted !== undefined}
            <span class="progress-text">Day: {$appStore.dailySessionsCompleted} / {$appStore.dailyGoal}</span>
        {/if}
    </div>

    <!-- Session Info Row -->
    <div class="session-info">
        <span class="session-type">{phaseDisplay}</span>

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
        width: 100%;
        /* Removed gap here to control spacing manually via margins */
    }

    .timer-label-container {
        display: flex;
        flex-direction: row;
        align-items: center;
        justify-content: center;
    }

    .timer-label {
        font-size: 2.6rem;
        font-weight: 900;
        margin: 0;
        text-align: center;
        font-variant-numeric: tabular-nums;
        line-height: 1; /* Tighter line height */
    }

    .daily-progress {
        font-size: 0.7rem;
        opacity: 0.5;
        margin-top: 4px;
        margin-bottom: 8px; /* Push session info down slightly */
        height: 14px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .session-info {
        display: flex;
        align-items: center;
        width: 100%;
        height: var(--session-info-height);
        justify-content: space-between;
        padding: 0 4px; /* Slight padding from edges */
    }

    .session-type,
    .flow-status {
        color: var(--text-color);
        font-size: var(--font-size-small);
        opacity: 0.8;
        text-transform: capitalize;
    }

    .session-type {
        font-weight: 500;
    }

    .flow-info {
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .badge {
        background: var(--flow-color, #4CAF50);
        color: rgba(0,0,0,0.9);
        padding: 1px 5px;
        border-radius: 4px;
        font-size: 0.65rem;
        font-weight: 700;
        line-height: 1;
    }
</style>
