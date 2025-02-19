<script lang="ts">
import { timer } from '../stores/timer';
import { progress } from '../stores/timerDerived';
import { FLOW_THRESHOLDS } from '../constants';

export let status: string;
export let progressColor: string;
export let frontmostApp: string;

$: isFlowMode = $timer.isFlowMode;
$: isRunning = status === 'running';
$: progressPercentage = 100 - ($progress.percentage || 0);

function handleSliderChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const value = 100 - parseInt(target.value, 10);
    const total = $progress.total;
    const remaining = Math.round((total * value) / 100);
    timer.setRemainingTime(remaining);
}
</script>

<div class="progress-container"
     class:running={isRunning}
     class:flow={isFlowMode}
     class:flow-streak={$timer.flowStreak >= FLOW_THRESHOLDS.FLOW_STREAK_THRESHOLD}>
    <div class="progress-background"></div>
    <div class="progress-bar"
             style:width="{100 - progressPercentage}%"
             style:background-color={progressColor}
             style:transform="translateZ(0)">
        {#if $timer.flowStreak > 0}
            <div class="flow-indicator">
                Flow streak: {$timer.flowStreak}
            </div>
        {/if}
    </div>
    {#if !isRunning}
        <input
            type="range"
            class="time-slider"
            min="0"
            max="100"
            value={progressPercentage}
            on:input={handleSliderChange}
        />
    {/if}
    <span class="progress-label">{frontmostApp}</span>
</div>

<style>
    .progress-container {
        position: relative;
        width: 100%;
        height: 20px;
        border-radius: calc(var(--border-radius) / 2);
        overflow: hidden;
        background-color: var(--background-color-light);
    }

    .progress-container:not(.running) {
        cursor: pointer;
    }

    .progress-background {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background-color: rgba(255, 255, 255, 0.1);
    }
    .progress-bar {
        position: absolute;
        top: 0;
        left: 0;
        height: 100%;
        background-color: var(--progress-color);
        will-change: width;
        transform: translateZ(0);
        transition: width 0.1s linear;
        backface-visibility: hidden;
    }

    .time-slider {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        margin: 0;
        opacity: 0;
        cursor: pointer;
        z-index: 3;
    }

    .progress-label {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        font-size: 12px;
        color: var(--text-color);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 90%;
        z-index: 2;
        pointer-events: none;
        user-select: none;
    }

    .flow .progress-bar {
        background-color: var(--flow-color, #4CAF50) !important;
    }

    /* Hover effects */
    .progress-container:hover .progress-bar {
        filter: brightness(1.1);
    }

    .progress-container:active .progress-bar {
        filter: brightness(0.9);
    }

    .progress-container:hover .progress-bar {
        opacity: 0.9;
    }

    .flow .progress-bar {
        background-color: var(--flow-color, #4CAF50) !important;
    }
    .progress-container:hover .progress-bar {
        opacity: 0.9;
    }

    .progress-bar {
        width: 100%;
        height: 100%;
        background-color: var(--progress-color);
        transform-origin: left;
        border-radius: 2px;
    }

    .progress-label {
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        font-size: 12px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 90%;
        z-index: 1;
    }
</style>
