<script>
/**
 * @file Manages the main functionality of a timer application.
 * @author Oleksii
 */

import { invoke } from '@tauri-apps/api/core';
import { writable, derived } from 'svelte/store';

/** Timer status. It can be 'running', 'paused', or 'stopped'. */
const timerStatus = writable('stopped');

/** Total time for the timer in seconds */
const totalTime = writable(1500); // 25 minutes by default

/** Remaining time in seconds */
const remainingTime = writable(1500); // 25 minutes by default

/** Progress value for the progress bar (100 to 0) */
const progress = derived(remainingTime, $remainingTime => $remainingTime);

/** Color for the progress bar, changing from green to red */
const progressColor = derived([remainingTime, totalTime], ([$remainingTime, $totalTime]) => {
    const ratio = $remainingTime / $totalTime;
    const red = Math.round(255 * Math.max(0, 1 - ratio));
    const green = Math.round(255 * Math.min(1, ratio));
    const blue = 0;
    return `rgb(${red}, ${green}, ${blue})`;
});

/** Displayed time format in 'HH:MM:SS' */
const displayTime = derived([remainingTime, timerStatus], ([$remainingTime, $timerStatus]) => {
    if ($timerStatus === 'stopped') {
        return '00:00:00';
    } else {
        const hours = Math.floor($remainingTime / 3600);
        const minutes = Math.floor(($remainingTime % 3600) / 60);
        const seconds = $remainingTime % 60;
        return [hours, minutes, seconds].map(unit => unit.toString().padStart(2, '0')).join(':');
    }
});

/** Frontmost application name */
const frontmostApp = writable('');

/**
 * Toggles the timer between 'paused' and 'running'.
 * @async
 */
async function toggleTimer() {
    try {
        if ($timerStatus !== 'running') {
            await invoke('start_timer');
            timerStatus.set('running');
            startTimer();
        } else {
            await invoke('pause_timer');
            timerStatus.set('paused');
            stopTimer();
        }
    } catch (error) {
        console.error('Error toggling timer:', error);
        // Handle the error appropriately, maybe show a user-friendly message
    }
}

        /**
         * Stops and resets the timer.
         * @async
         */
        async function stopAndReset() {
            try {
                await invoke('stop_timer');
                await invoke('submit');
                await invoke('reset_timer');
                timerStatus.set('stopped');
                stopTimer();
                remainingTime.set($totalTime); // Reset to the total time
            } catch (error) {
                console.error('Error stopping and resetting timer:', error);
                // Handle the error appropriately, maybe show a user-friendly message
            }
        }

        /**
         * Updates the frontmost application name.
         * @async
         */
        async function updateFrontmostApp() {
            frontmostApp.set(await invoke('get_frontmost_application').then(app => app));
        }

        /** @type {number} ID of the setInterval function. */
        let intervalId;

        function startTimer() {
            stopTimer();
            intervalId = setInterval(() => {
                remainingTime.update(t => {
                    if (t > 0) {
                        updateFrontmostApp();
                        return t - 1;
                    } else {
                        stopTimer();
                        timerStatus.set('stopped');
                        return 0;
                    }
                });
            }, 1000);
        }

        function stopTimer() {
            if (intervalId) {
                clearInterval(intervalId);
                intervalId = null;
            }
        }

        // Start or stop the timer based on its status
        timerStatus.subscribe(status => {
            if (status === 'running') {
                startTimer();
            } else {
                stopTimer();
    }
});

/**
 * Submits the current time to Obsidian.
 * @async
 */
async function submit() {
    await invoke('submit').catch(error => {
        console.error('Error submitting time:', error);
        // Handle the error appropriately, maybe show a user-friendly message
    });
}

/** set background color based on the mouseleave event */
function onmouseleave() {
    document.body.style.backgroundColor = 'rgb(20, 20, 20)'
}

function onmouseenter() {
    document.body.style.backgroundColor = '#2c2c2c'
}

</script>

<main class="container"
    on:mouseleave={onmouseleave}
    on:mouseenter={onmouseenter}
    >
    <div class="clock-wrapper">
        <!-- <div class="frontmost-application-name-container"> -->
        <!--     <div class="frontmost-application-name"> -->
        <!--         {$frontmostApp} -->
        <!--     </div> -->
        <!-- </div> -->
        <h1>{$displayTime}</h1>
        <progress
            value={$progress}
            max={$totalTime}
            style="--progress-color: {$progressColor}"
            class:blink={$timerStatus === 'stopped' || $timerStatus === 'paused'}
        ></progress>
        <div class="buttons-container">
            <button on:click={toggleTimer}>
                { $timerStatus === 'paused' ? 'Resume' : ($timerStatus === 'running' ? 'Pause' : 'Start') }
            </button>
            <button on:click={stopAndReset}>
                { $timerStatus === 'stopped' ? 'Reset' : 'Stop' }
            </button>
            <button on:click={submit}>
                Submit
            </button>
        </div>
    </div>
</main>

<style>
@keyframes blink {
    0% { opacity: 1; }
    50% { opacity: 0; }
    100% { opacity: 1; }
}

.blink {
    animation: blink 1s ease-in-out infinite;
}
.frontmost-application-name {
    font-size: 12px;
    color: #fffffbaa;
    font-family: "San Francisco Display", sans-serif;
    font-weight: 700;
    padding: 0;
    margin: 0;
}
.container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 70px;
    padding: 12px;
}
.clock-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    /* gap: 5px; */
    width: 100%;
}
h1 {
    margin: 0;
    font-size: 2.6rem;
    font-family: "San Francisco Display", sans-serif;
}
progress {
    width: 100%;
    height: 5px;
    -webkit-appearance: none;
    appearance: none;
}
progress::-webkit-progress-bar {
    background-color: #2c2c2c;
    border-radius: 2px;
}
progress::-webkit-progress-value {
    background-color: var(--progress-color, rgb(255, 89, 0));
    border-radius: 2px;
}
.buttons-container {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 5px;
}
button {
    padding: 5px 10px;
    border-radius: 5px;
    border: none;
    background-color: #1a1a1a;
    font-size: 12px;
    cursor: pointer;
}
button:hover {
    background-color: hsla(0, 0%, 100%, 0.2);
}
</style>
