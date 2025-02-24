<script lang="ts">
import { onMount, onDestroy } from 'svelte';
import { listen } from '@tauri-apps/api/event';
import { configStore } from './stores/config';
import ButtonGroup from './components/ButtonGroup.svelte';
import ProgressBar from './components/ProgressBar.svelte';
import FocusStats from './components/FocusStats.svelte';
import { timerStore } from './stores/timer';
import { sessionStore } from './stores/session';
import { flowStore } from './stores/flow';
import { resizeWindow } from './utils/window';

const onBlur = () => {
    document.body.style.backgroundColor = 'var(--background-color-blur)';
};

const onFocus = () => {
    document.body.style.backgroundColor = 'var(--background-color-focus)';
};

let unsubscribers: Array<() => void> = [];

onMount(async () => {
    await configStore.load();
    onBlur();

    listen('on_blur', onBlur);
    listen('on_focus', onFocus);

    // Subscribe to store changes that should trigger resize
    unsubscribers = [
        flowStore.subscribe(() => resizeWindow()),
        sessionStore.subscribe(() => resizeWindow())
    ];

    // Initial resize
    resizeWindow();
});

onDestroy(() => {
    // Cleanup subscriptions
    unsubscribers.forEach(unsubscribe => unsubscribe());
});

</script>

<main class="container"
    class:blinking={($timerStore.status === 'paused' || $timerStore.status === 'stopped') && $configStore.behavior.annoyingLevel === 'high'}
>
    <div class="content">
        <div class="top-section">
            <FocusStats />
        </div>
        <div class="middle-section">
            <ProgressBar/>
        </div>
        <div class="bottom-section">
            <ButtonGroup/>
        </div>
    </div>
</main>

<style>
:global(:root) {
    --font-family: "San Francisco Display", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen-Sans, Ubuntu, Cantarell, "Helvetica Neue", sans-serif;
    --font-size-small: 12px;
    --background-color: rgb(38, 38, 38);
    --background-color-light: rgb(56, 56, 56);
    --background-color-blur: rgba(20, 20, 20, 1.00);
    --background-color-focus: rgba(38, 38, 38, 1.00);
    --text-color: rgba(255, 255, 255, 0.8);
    --progress-color: rgb(255, 89, 0);
    --progress-height: var(--font-size-small);
    --progress-background-color: rgb(56, 56, 56);
    --blink-color: rgb(255, 0, 0);
    --border-radius: 8px;
    --button-padding: 5px 10px;
    --spacing-small: 5px;
    --spacing-medium: 16px;
    --window-width: 160px;
    --session-info-height: var(--font-size-small);
    --break-text-color: rgb(0, 255, 0);
}

:global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

:global(body) {
    overflow: hidden;
    font-family: var(--font-family);
    color: var(--text-color);
    height: var(--window-height);
    width: var(--window-width);
    background-color: var(--background-color-focus);
}

.content {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: var(--spacing-medium);
    padding-top: var(--spacing-small);
}

.top-section {
    flex: 0 0 auto;
    margin-bottom: var(--spacing-small);
}

.middle-section {
    flex: 0 0 auto;
    margin: var(--spacing-small) 0;
}

.bottom-section {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    min-height: 0;
    overflow-y: auto;
}

@keyframes blink {
0% { background-color: var(--background-color-blur); }
50% { background-color: var(--blink-color); }
100% { background-color: var(--background-color-blur); }
}

.blinking {
    animation: blink 3s infinite;
}
</style>
