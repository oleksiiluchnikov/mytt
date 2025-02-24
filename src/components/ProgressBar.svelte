<script lang="ts">
    import { timerStore } from '../stores/timer';
    import { sessionStore } from '../stores/session';
    import { flowStore } from '../stores/flow';
    import { TIMER_STATUS, FLOW_STATUS } from '../constants';
    import * as utils from '../utils/time';

    let progressBarId = `progress-bar-${Math.random().toString(36).substr(2, 9)}`;

    function handleSliderChange(event: Event) {
        const target = event.target as HTMLInputElement;
        const sliderValue = parseInt(target.value, 10);

        if ($timerStore.status === TIMER_STATUS.STOPPED) {
            const maxDuration = $timerStore.preferences.maximumDuration;
            const minDuration = $timerStore.preferences.minimumDuration;
            const range = maxDuration - minDuration;
            const newDuration = Math.round(minDuration + (range * sliderValue / 100));
            updateStoppedTimer(newDuration);
        } else {
            updateRunningTimer(sliderValue);
        }
    }

    function updateStoppedTimer(newDuration: number) {
        const maxDuration = $timerStore.preferences.maximumDuration;
        const minDuration = $timerStore.preferences.minimumDuration;
        const percentage = ((newDuration - minDuration) / (maxDuration - minDuration)) * 100;

        timerStore.update(s => ({
            ...s,
            time: {
                total: newDuration,
                remaining: newDuration,
                display: utils.formatTime(newDuration)
            },
            progress: {
                ...s.progress,
                total: newDuration,
                percentage: Math.max(0, Math.min(100, percentage))
            }
        }));

        sessionStore.set({
            ...sessionStore.state,
            lastSessionDuration: newDuration,
            suggestedNextDuration: newDuration
        });
    }

    function updateRunningTimer(percentage: number) {
        const total = $timerStore.time.total;
        const remaining = Math.round((total * percentage) / 100);

        timerStore.update(s => ({
            ...s,
            time: {
                ...s.time,
                remaining,
                display: utils.formatTime(remaining)
            },
            progress: {
                ...s.progress,
                percentage: Math.max(0, Math.min(100, percentage))
            }
        }));
    }
</script>

<div
    class="progress-container"
    class:running={$timerStore.status === TIMER_STATUS.RUNNING}
    class:flow={$flowStore.status === FLOW_STATUS.FLOW}
    role="progressbar"
    aria-valuemin="0"
    aria-valuemax="100"
    aria-valuenow={$timerStore.progress.percentage}
    aria-label="Timer progress"
>
    <div class="progress-background" aria-hidden="true"></div>
    <div
        class="progress-bar"
    style:width="{$timerStore.progress.percentage}%"
    >
    </div>
    {#if $timerStore.status !== TIMER_STATUS.RUNNING}
        <input
            type="range"
            class="time-slider"
            min="0"
            max="100"
    value={$timerStore.progress.percentage}
            on:input={handleSliderChange}
            aria-labelledby={progressBarId}
        />
    {/if}
</div>

<style>
    .progress-container {
        position: relative;
        width: 100%;
        height: var(--progress-height);
        border-radius: calc(var(--border-radius) / 2);
        overflow: hidden;
        transition: background-color 0.3s ease;
    }

    .progress-container:not(.running) {
        cursor: pointer;
    }

    .progress-background {
        position: absolute;
        inset: 0;
        background-color: var(--background-color-blur);
        opacity: 0.75;
    }

    .progress-bar {
        position: absolute;
        inset: 0 auto 0 0;
        background-color: var(--progress-color);
        transform-origin: left;
    }

    .time-slider {
        position: absolute;
        inset: 0;
        margin: 0;
        opacity: 0;
        cursor: pointer;
        z-index: 3;
    }

    /* Interactions */
    @media (hover: hover) {
        .progress-container:hover .progress-bar {
            filter: brightness(1.1);
            opacity: 0.9;
        }

        .progress-container:active .progress-bar {
            filter: brightness(0.9);
        }
    }

    /* Accessibility */
    .time-slider:focus-visible {
        outline: 2px solid var(--focus-color, #007bff);
        outline-offset: 2px;
    }

    /* Animation optimizations */
    .progress-bar {
        backface-visibility: hidden;
        perspective: 1000px;
        transform: translateZ(0);
    }
</style>
