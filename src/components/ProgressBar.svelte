<script lang="ts">
    import { appStore, progressPct } from '../stores/app';
</script>

<div class="progress-container">
    <div class="progress-bar" style:width="{$progressPct}%"></div>
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
