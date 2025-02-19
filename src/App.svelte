<script lang="ts">
import { onMount, onDestroy } from 'svelte';
import { listen } from '@tauri-apps/api/event';
import { configStore } from './stores/config';
import ButtonGroup from './components/ButtonGroup.svelte';
import ProgressBar from './components/ProgressBar.svelte';
import FocusStats from './components/FocusStats.svelte';
import { timer } from './stores/timer';
import { progress, progressColor } from './stores/timerDerived';
import { resizeWindow } from './utils/window';


const onBlur = () => {
    timer.setAnnoyingLevel('high');
};

const onFocus = () => {
    timer.setAnnoyingLevel('low');
};
onMount(async () => {
    await configStore.load();
    const unlistenEvents = await Promise.all([
        listen('on_blur', onBlur),
        listen('on_focus', onFocus)
    ]);

    const annoyInterval = setInterval(() => {
        timer.setAnnoyingLevel('high');
        setTimeout(() => timer.setAnnoyingLevel('low'), 500);
    }, 5000);

    // Create ResizeObserver to watch for content changes
    const resizeObserver = new ResizeObserver(async () => {
        await resizeWindow();
    });

    // Observe the main content container
    const mainElement = document.querySelector('main');
    if (mainElement) {
        resizeObserver.observe(mainElement);
    }

    onDestroy(() => {
        unlistenEvents.forEach(unsubscribe => unsubscribe());
        clearInterval(annoyInterval);
        resizeObserver.disconnect();
        timer.update(s => ({ ...s, annoyingLevel: 'low' }));
    });
});
</script>

<main class="container"
    class:blinking={$timer.status !== 'running' && $timer.annoyingLevel === 'high'}>
    <div class="content">
        <div class="top-section">
            <FocusStats />
        </div>
        <div class="middle-section">
            <ProgressBar
                status={$timer.status}
                progressColor={$progressColor}
                frontmostApp={$timer.frontmostApp}
            />
        </div>
        <div class="bottom-section">
            <ButtonGroup/>
        </div>
    </div>
</main>

<style>
:global(:root) {
    --font-family: "San Francisco Display", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen-Sans, Ubuntu, Cantarell, "Helvetica Neue", sans-serif;
    --background-color: rgb(38, 38, 38);
    --background-color-light: rgb(56, 56, 56);
    --text-color: rgba(255, 255, 255, 0.8);
    --progress-color: rgb(255, 89, 0);
    --blink-color: rgb(255, 0, 0);
    --border-radius: 8px;
    --button-padding: 5px 10px;
    --spacing-small: 5px;
    --spacing-medium: 16px;
    --window-width: 240px;
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
    background-color: var(--background-color);
    height: var(--window-height);
    width: var(--window-width);
}

main {
    display: flex;
    height: 100%;
    width: 100%;
    flex-direction: column;
    background-color: var(--background-color);
    transition: background-color 0.3s ease;
    overflow: hidden;
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

.container:hover {
    background-color: var(--background-color-light);
}

@keyframes blink {
0%, 100% { background-color: var(--background-color); }
50% { background-color: var(--blink-color); }
}

.blinking {
    animation: blink 1s infinite;
}
</style>
