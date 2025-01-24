<script>
import { invoke } from '@tauri-apps/api/core';
import { writable, derived } from 'svelte/store';
import { onMount, onDestroy } from 'svelte';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { afterUpdate } from 'svelte';
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

const START_LABEL = 'start';
const PAUSE_LABEL = 'pause';
const RESUME_LABEL = 'resume';
const STOP_LABEL = 'stop';
const LOG_LABEL = 'log';
const SKIP_BREAK_LABEL = 'skip break';
// when using `"withGlobalTauri": true`, you may use
// const { isPermissionGranted, requestPermission, sendNotification, } = window.__TAURI__.notification;

/**
 * Sends a notification.
 * @async
 * @param {import('@tauri-apps/plugin-notification').Options} options - The notification options.
 */
async function notify(options) {
    // Do you have permission to send a notification?
    let permissionGranted = await isPermissionGranted();

    // If not we need to request it
    if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
    }

    // Once permission has been granted we can send the notification
    if (permissionGranted) {
        sendNotification(options);
    }
}

/** Timer status. It can be 'running', 'paused', or 'stopped'. */
const timerStatus = writable('stopped');

/** Work session duration in seconds */
const workDuration = writable(1500); // 25 minutes by default

/** Short break duration in seconds */
const shortBreakDuration = writable(300); // 5 minutes

/** Long break duration in seconds */
const longBreakDuration = writable(900); // 15 minutes

/** Number of work sessions before a long break */
const sessionsBeforeLongBreak = writable(4);

/** Current session type: 'work', 'shortBreak', or 'longBreak' */
const sessionType = writable('work');

/** Number of completed work sessions */
const completedSessions = writable(0);

/** Total time for the current session in seconds */
const totalTime = derived([sessionType, workDuration, shortBreakDuration, longBreakDuration],
    ([$sessionType, $workDuration, $shortBreakDuration, $longBreakDuration]) => {
        switch($sessionType) {
            case 'work': return $workDuration;
            case 'shortBreak': return $shortBreakDuration;
            case 'longBreak': return $longBreakDuration;
        }
    }
);

/** Remaining time in seconds */
const remainingTime = writable(1500); // Start with work duration

/** Progress value for the progress bar (100 to 0) */
const progress = derived(remainingTime, $remainingTime => $remainingTime);

/** Color for the progress bar, changing from green to red */
const progressColor = derived([remainingTime, totalTime], ([$remainingTime, $totalTime]) => {
    const ratio = $remainingTime / $totalTime;
    const endColor = [255, 89, 0];
    const startColor = [255, 0, 0];
    const red = Math.round(startColor[0] + ratio * (endColor[0] - startColor[0]));
    const green = Math.round(startColor[1] + ratio * (endColor[1] - startColor[1]));
    const blue = Math.round(startColor[2] + ratio * (endColor[2] - startColor[2]));
    return `rgb(${red}, ${green}, ${blue})`;
});

/** Displayed time format in 'HH:MM:SS' */
const displayTime = derived([remainingTime, timerStatus], ([$remainingTime, $timerStatus]) => {
    if ($timerStatus === 'stopped') {
        return new Date($remainingTime * 1000).toISOString().substr(11, 8);
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
         * Updates the frontmost application name.
         * @async
         */
async function updateFrontmostApp() {
    frontmostApp.set(await invoke('get_frontmost_application').then(app => app));
}

/** @type {number} ID of the setInterval function. */
let intervalId;


function transitionToNextSession() {
    sessionType.update(currentType => {
        if (currentType === 'work') {
            completedSessions.update(s => s + 1);
            if ($completedSessions % $sessionsBeforeLongBreak === 0) {
                return 'longBreak';
            } else {
                return 'shortBreak';
            }
        } else {
            return 'work';
        }
    });
    remainingTime.set($totalTime);
    timerStatus.set('stopped');
}

/**
 * Runs the on_log script placed in the ~/.config/mytt/scripts directory.
 * @async
 */
async function onLog() {
    await invoke('on_log').catch(error => {
        console.error('Error logting time:', error);
        // Handle the error appropriately, maybe show a user-friendly message
    });
}

async function onStart() {
    await invoke('on_start').catch(error => {
        console.error('Error starting:', error);
        // Handle the error appropriately, maybe show a user-friendly message
    });
    startTimer();
}

/**
    * Starts the timer.
    */
async function startTimer() {
    intervalId = setInterval(() => {
        remainingTime.update(t => {
            if (t > 0) {
                return t-1;
            } else {
                onStop();
                return $totalTime;
            }
        });
    }, 1000);
    timerStatus.set('running');
}

/**
 * Toggles the timer between 'paused' and 'running'.
 * @async
 */
async function onPause() {
    await invoke('on_pause').catch(error => {
        console.error('Error pausing:', error);
    });
    pauseTimer();
}

/**
* Pauses the timer.
*/
function pauseTimer() {
    if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
    }
    $timerStatus = 'paused';
}

async function onStop() {
    await invoke('on_stop').catch(error => {
        console.error('Error stopping:', error);
        // Handle the error appropriately, maybe show a user-friendly message
    });
    stopTimer();
    transitionToNextSession();
}

function stopTimer() {
    if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
    }
    timerStatus.set('stopped');
    remainingTime.set($totalTime);
}

async function onSkipBreak() {
    await invoke('on_skip_break').catch(error => {
        console.error('Error resetting:', error);
        // Handle the error appropriately, maybe show a user-friendly message
    });
    skipBreak();
}

function skipBreak() {
    if ($sessionType === 'work') return;
    sessionType.set('work');
    remainingTime.set($workDuration);
    timerStatus.set('stopped');
}


async function onResume() {
    await invoke('on_resume').catch(error => {
        console.error('Error resuming:', error);
        // Handle the error appropriately, maybe show a user-friendly message
    });
    timerStatus.set('running');
    startTimer();
}

/**
    * On pointerup event, call the corresponding function based on the button clicked.
    * @param {PointerEvent} event - The pointerup event.
    */
function onPointerDown(event) {
    const button = event.target
    if (!button) return;
    switch (button.textContent) {
        case START_LABEL:
            onStart();
            break;
        case PAUSE_LABEL:
            onPause();
            break;
        case RESUME_LABEL:
            onResume();
            break;
        case STOP_LABEL:
            onStop();
            break;

        case LOG_LABEL:
            onLog();
            break;
        case SKIP_BREAK_LABEL:
            onSkipBreak();
            break;
    }
}

async function onBlur() {
    document.body.style.backgroundColor = 'rgb(20, 20, 20)'
    let buttons = document.querySelectorAll('button');
    buttons.forEach(button => {
        if (!button) return;
        button.style.backgroundColor = 'rgb(38, 38, 38)';
    });
}
async function onFocus() {
    document.body.style.backgroundColor = 'rgb(38, 38, 38)'
    let buttons = document.querySelectorAll('button');
    if (!buttons) return;
    buttons.forEach(button => {
        if (!button) return;
        button.style.backgroundColor = 'rgb(56, 56, 56)';
    });
}


async function getContentSize() {
    const mainElement = document.querySelector('main');
    if (!mainElement) return { width: 300, height: 200 }; // fallback size

    const rect = mainElement.getBoundingClientRect();
    const outerSize = await getCurrentWindow().outerSize();
    const innerSize = await getCurrentWindow().innerSize();
    let titleBarHeight = outerSize.height - innerSize.height;
    if (titleBarHeight === 0) {
        titleBarHeight = 30;
    }

    return {
        width: Math.ceil(rect.width),
        height: Math.ceil(rect.height) + titleBarHeight
    };
}

let resizeTimeout;

function debounceResize(func, wait) {
    return (...args) => {
        clearTimeout(resizeTimeout);
        resizeTimeout = setTimeout(() => func.apply(this, args), wait);
    };
}

const resizeWindow = debounceResize(async () => {
    const { width, height } = await getContentSize();
    await getCurrentWindow().setSize(new LogicalSize(width, height));
}, 50);

afterUpdate(resizeWindow);

onMount(async () => {
    // Initial resize
    resizeWindow();
});

onDestroy(async () => {
});

listen('on_blur', onBlur);
listen('on_focus', onFocus);

</script>

<main class="container">
    <!-- <div class="frontmost-application-name-container"> -->
    <!--     <div class="frontmost-application-name"> -->
    <!--         {$frontmostApp} -->
    <!--     </div> -->
    <!-- </div> -->
    <!-- <h1>{$displayTime}</h1> -->
    <!-- <div class="timer-container"> -->
    <!-- <span class="session-type">{$sessionType}</span> -->
    <!-- </div> -->
    <div class="timer-container">
        <h1>{$displayTime}</h1>
    </div>
    <div class="progress-container">
        <div class="progress-bar" class:blink={$timerStatus === 'stopped' || $timerStatus === 'paused'} style="transform: scaleX({$progress / $totalTime}); background-color: {$progressColor};"></div>
        <span class="progress-label">{$frontmostApp}</span>
    </div>


    <div class="buttons-container">
        <button on:pointerdown={onPointerDown}>
            { $timerStatus === 'stopped' ? START_LABEL : $timerStatus === 'paused' ? RESUME_LABEL : PAUSE_LABEL }
        </button>
        <button on:pointerdown={onPointerDown}>
            { $sessionType === 'work' ? STOP_LABEL : SKIP_BREAK_LABEL }
        </button>
        <button on:pointerdown={onPointerDown}>
            { LOG_LABEL }
        </button>
    </div>
</main>

<style>
:global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}
:global(body) {
    overflow: hidden;
}
main {
    padding: 16px;
    padding-top: 0;
}

.session-type {
    font-size: 12px;
    color: #fffffbaa;
    font-family: "San Francisco Display", sans-serif;
    align-self: flex-end;
}

@keyframes blink {
    0%, 100% { opacity: 0; }
    50% { opacity: 1; }
}

.blink {
    animation: blink 2s step-end infinite;
}

.container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;

    width: fit-content;
    height: fit-content;
}
.timer-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
}
h1 {
    display: flex;
    font-size: 2.6em;
    font-family: "San Francisco Display", sans-serif;
    margin-block-start: 0;
    margin-block-end: 0;
    margin-inline-start: 0;
    margin-inline-end: 0;
    margin-top: 5px;
    font-weight: bold;
}
.progress-container {
    position: relative;
    width: 100%;
    height: 1em;
    border-radius: 4px;
    overflow: hidden;
    padding: 2px;
    border: 1px solid var(--progress-color, rgb(255, 89, 0));
}

.progress-bar {
    width: 100%;
    height: 100%;
    background-color: var(--progress-color, rgb(255, 89, 0));
    transform-origin: left;
    transition: transform 0.1s linear;
    border-radius: 2px;
}

.progress-label {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 12px;
    color: rgba(255, 255, 255, 0.8);
    font-family: "San Francisco Display", sans-serif;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 90%;
    z-index: 1;
}

.blink .progress-bar {
    animation: blink 2s step-end infinite;
}
.buttons-container {
    display: flex;
    margin-top: 5px;
    flex-direction: row;
    justify-content: space-between;
    width: 100%;
    gap: 5px;
}
button {
    flex: 1;
    padding: 5px 10px;
    border-radius: 5px;
    border: none;
    background-color: rgb(56, 56, 56);
    font-size: 12px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
}
button:hover {
    background-color: hsla(0, 0%, 100%, 0.2);
}
</style>
