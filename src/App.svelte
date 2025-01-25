<script>
import { invoke } from '@tauri-apps/api/core';
import { writable, derived, get } from 'svelte/store';
import { onMount, onDestroy, afterUpdate } from 'svelte';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

const LABELS = {
    START: 'start',
    PAUSE: 'pause',
    RESUME: 'resume',
    STOP: 'stop',
    LOG: 'log',
    SKIP_BREAK: 'skip'
};

const SESSION_TYPES = {
    WORK: 'work',
    SHORT_BREAK: 'shortBreak',
    LONG_BREAK: 'longBreak'
};

const TIMER_STATUS = {
    RUNNING: 'running',
    PAUSED: 'paused',
    STOPPED: 'stopped'
};
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

/**
 * Creates a writable store for managing the state of a timer.
 */
const createTimerStore = () => {
    const { subscribe, set, update } = writable({
        theme: 'dark',
        status: TIMER_STATUS.STOPPED,
        sessionType: SESSION_TYPES.WORK,
        workDuration: 1500,
        shortBreakDuration: 300,
        longBreakDuration: 900,
        sessionsBeforeLongBreak: 4,
        annoyingLevel: null,
        completedSessions: 0,
        remainingTime: 1500,
        frontmostApp: ''
    });

    return {
        subscribe,
        update,
        setStatus: (/** @type {any} */ status) => update(s => ({ ...s, status })),
        setSessionType: (/** @type {any} */ type) => update(s => ({ ...s, sessionType: type })),
        setRemainingTime: (/** @type {any} */ time) => update(s => ({ ...s, remainingTime: time })),
        incrementCompletedSessions: () => update(s => ({ ...s, completedSessions: s.completedSessions + 1 })),
        setFrontmostApp: (/** @type {any} */ app) => update(s => ({ ...s, frontmostApp: app })),
        updateConfig: (/** @type {any} */ config) => update(s => ({ ...s, ...config }))
    };
};

const timerStore = createTimerStore();
const isBlinking = derived(timerStore, $store => {
    const { status, annoyingLevel } = $store;
    return (status === TIMER_STATUS.STOPPED || status === TIMER_STATUS.PAUSED) && annoyingLevel === 'high';
});

$: if ($isBlinking) {
    triggerAnnoyingBehavior();
}

$: backgroundColor = $timerStore.theme === 'dark' ? 'rgb(20, 20, 20)' : 'rgb(38, 38, 38)';
$: backgroundColorLight = $timerStore.theme === 'dark' ? 'rgb(38, 38, 38)' : 'rgb(56, 56, 56)';

const totalTime = derived(timerStore, $store => {
    switch($store.sessionType) {
        case SESSION_TYPES.WORK: return $store.workDuration;
        case SESSION_TYPES.SHORT_BREAK: return $store.shortBreakDuration;
        case SESSION_TYPES.LONG_BREAK: return $store.longBreakDuration;
    }
});

const progress = derived(timerStore, $store => $store.remainingTime);

const progressColor = derived([timerStore, totalTime], ([$store, $totalTime]) => {
    const ratio = $store.remainingTime / $totalTime;
    const endColor = [255, 89, 0];
    const startColor = [255, 0, 0];
    const red = Math.round(startColor[0] + ratio * (endColor[0] - startColor[0]));
    const green = Math.round(startColor[1] + ratio * (endColor[1] - startColor[1]));
    const blue = Math.round(startColor[2] + ratio * (endColor[2] - startColor[2]));
    return `rgb(${red}, ${green}, ${blue})`;
});

const displayTime = derived(timerStore, $store => {
    const formatTime = (/** @type {number} */ time) => {
        const hours = Math.floor(time / 3600);
        const minutes = Math.floor((time % 3600) / 60);
        const seconds = time % 60;
        return [hours, minutes, seconds].map(unit => unit.toString().padStart(2, '0')).join(':');
    };

    return $store.status === TIMER_STATUS.STOPPED
        ? new Date($store.remainingTime * 1000).toISOString().substr(11, 8)
        : formatTime($store.remainingTime);
});

/** @type {number} ID of the setInterval function. */
let intervalId;
let randomPositionIntervalId;

const transitionToNextSession = () => {
    const store = get(timerStore);
    if (store.sessionType === SESSION_TYPES.WORK) {
        timerStore.incrementCompletedSessions();
        const nextSessionType = (store.completedSessions + 1) % store.sessionsBeforeLongBreak === 0
            ? SESSION_TYPES.LONG_BREAK
            : SESSION_TYPES.SHORT_BREAK;
        timerStore.setSessionType(nextSessionType);
        timerStore.setRemainingTime(store[`${nextSessionType}Duration`]);
    } else {
        timerStore.setSessionType(SESSION_TYPES.WORK);
        timerStore.setRemainingTime(store.workDuration);
    }
    timerStore.setStatus(TIMER_STATUS.STOPPED);
};

const invokeWithErrorHandling = async (/** @type {string} */ action, /** @type {string} */ errorMessage) => {
    try {
        await invoke(action);
    } catch (error) {
        console.error(`${errorMessage}:`, error);
        // Handle the error appropriately, maybe show a user-friendly message
    }
};

const onLog = () => invokeWithErrorHandling('on_log', 'Error logging time');

const onStart = async () => {
    await invokeWithErrorHandling('on_start', 'Error starting');
    startTimer();
};

const startTimer = () => {
    stopAnnoyingBehavior();
    intervalId = setInterval(() => {
        const store = get(timerStore);
        if (store.remainingTime > 0) {
            timerStore.setRemainingTime(store.remainingTime - 1);
        } else {
            clearInterval(intervalId);
            intervalId = null;
            onStop();
        }
    }, 1000);
    timerStore.setStatus(TIMER_STATUS.RUNNING);
};

const stopAnnoyingBehavior = () => {
    if (randomPositionIntervalId) {
        clearInterval(randomPositionIntervalId);
        randomPositionIntervalId = null;
    }
    clearInterval(randomPositionIntervalId);
};


const triggerAnnoyingBehavior = async () => {
    const { annoyingLevel } = $timerStore;
    if (annoyingLevel !== 'high') return;

    stopAnnoyingBehavior();

    const notificationOptions = {
        title: 'mytt',
        body: 'Time to start working!',
        icon: 'assets/icon.png',
        sound: 'default'
    };

    await notify(notificationOptions);

    randomPositionIntervalId = setInterval(setRandomPosition, 5000);
};

const onPause = async () => {
    await invokeWithErrorHandling('on_pause', 'Error pausing');
    pauseTimer();
};

const pauseTimer = () => {
    if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
    }
    timerStore.setStatus(TIMER_STATUS.PAUSED);
triggerAnnoyingBehavior();
};

const onStop = async () => {
    await invokeWithErrorHandling('on_stop', 'Error stopping');
    stopTimer();
    transitionToNextSession();
};

const stopTimer = () => {
    if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
    }
    timerStore.setStatus(TIMER_STATUS.STOPPED);
    timerStore.setRemainingTime($totalTime);
};

const onSkipBreak = async () => {
    await invokeWithErrorHandling('on_skip_break', 'Error skipping break');
    skipBreak();
};

const skipBreak = () => {
    if ($timerStore.sessionType === SESSION_TYPES.WORK) return;
    timerStore.setSessionType(SESSION_TYPES.WORK);
    timerStore.setRemainingTime($timerStore.workDuration);
    timerStore.setStatus(TIMER_STATUS.STOPPED);
};

const onResume = async () => {
    await invokeWithErrorHandling('on_resume', 'Error resuming');
    timerStore.setStatus(TIMER_STATUS.RUNNING);
    startTimer();
};

const onPointerDown = (/** @type {{ target: any; }} */ event) => {
    const button = event.target;
    if (!button) return;
    const actionMap = {
        [LABELS.START]: onStart,
        [LABELS.PAUSE]: onPause,
        [LABELS.RESUME]: onResume,
        [LABELS.STOP]: onStop,
        [LABELS.LOG]: onLog,
        [LABELS.SKIP_BREAK]: onSkipBreak
    };
    const action = actionMap[button.textContent];
    if (action) action();
};

const updateStyles = (/** @type {string} */ bodyColor, /** @type {string} */ buttonColor) => {
    // document.body.style.backgroundColor = bodyColor;
    document.querySelector('main').style.backgroundColor = bodyColor;
    document.querySelectorAll('button').forEach(button => {
        if (button) button.style.backgroundColor = buttonColor;
    });
};

const onBlur = () => {
    updateStyles('rgb(20, 20, 20)', 'rgb(38, 38, 38)');
    timerStore.update(s => ({ ...s, annoyingLevel: 'high' }));
};

const onFocus = () => {
    updateStyles('rgb(38, 38, 38)', 'rgb(56, 56, 56)');
    timerStore.update(s => ({ ...s, annoyingLevel: 'low' }));
    stopAnnoyingBehavior();
};

const getContentSize = async () => {
    const mainElement = document.querySelector('main');
    if (!mainElement) return { width: 300, height: 200 }; // fallback size

    const rect = mainElement.getBoundingClientRect();
    const { height: outerHeight } = await getCurrentWindow().outerSize();
    const { height: innerHeight } = await getCurrentWindow().innerSize();
    const titleBarHeight = Math.max(outerHeight - innerHeight, 30);

    return {
        width: Math.ceil(rect.width),
        height: Math.ceil(rect.height) + titleBarHeight
    };
};

const debounce = (/** @type {{ (): Promise<void>; apply?: any; }} */ func, /** @type {number} */ wait) => {
    let timeout;
    return (/** @type {any} */ ...args) => {
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(this, args), wait);
    };
};

const resizeWindow = debounce(async () => {
    // Get previous window size
    const { width: prevWidth, height: prevHeight } = await getCurrentWindow().innerSize();
    // Get new content size
    const size = await getContentSize();
    const buffer = 10;
    // If the new size is the same as the previous size+-buffer, don't resize
    if (size.width >= prevWidth - buffer && size.width <= prevWidth + buffer &&
        size.height >= prevHeight - buffer && size.height <= prevHeight + buffer) {
        return;
    }

    await getCurrentWindow().setSize(new LogicalSize(size.width, size.height));
}, 50);

const setRandomPosition = async () => {
    // get the screen size of the display the window is on
    const screenWidth = 2160;
    const screenHeight = 1080;
    const x = Math.floor(Math.random() * screenWidth);
    const y = Math.floor(Math.random() * screenHeight);
    await getCurrentWindow().setPosition(new LogicalPosition(x, y));
};

const setRandomSize = async () => {
    const baseWidth = 300;
    const baseHeight = 200;
    const randomWidth = baseWidth + Math.floor(Math.random() * 100);
    const randomHeight = baseHeight + Math.floor(Math.random() * 100);
    await getCurrentWindow().setSize(new LogicalSize(randomWidth, randomHeight));
};

afterUpdate(resizeWindow);

const loadConfig = async () => {
    try {
        const config = await invoke('fetch_config').then(JSON.parse);
        timerStore.updateConfig({
            workDuration: config.work_duration * 60,
            shortBreakDuration: config.short_break_duration * 60,
            longBreakDuration: config.long_break_duration * 60,
            annoyingLevel: config.annoying_level,
            sessionsBeforeLongBreak: config.sessions_before_long_break,
            theme: config.theme
        });
    } catch (error) {
        console.error('Error loading config:', error);
    }
};

onMount(async () => {
    await loadConfig();

    // set to blinking until the status changes
    timerStore.update(s => ({ ...s, annoyingLevel: 'high' }));

    resizeWindow();
});

onDestroy(() => {
    if (intervalId) clearInterval(intervalId);
    if (randomPositionIntervalId) clearInterval(randomPositionIntervalId);
});

listen('on_blur', onBlur);
listen('on_focus', onFocus);

</script>

<main class="container"
    class:blinking={$isBlinking}
    style="
    --background-color: {backgroundColor};
    --background-color-light: {backgroundColorLight};
    --blink-color: rgb(255, 0, 0);
    "
>
    <div class="timer-container">
        <h1>{$displayTime}</h1>
    </div>
    <div class="progress-container">
        <div class="progress-bar"
             style="transform: scaleX({$progress / $totalTime}); background-color: {$progressColor};">
        </div>
        <span class="progress-label">{$timerStore.frontmostApp}</span>
    </div>
    <div class="buttons-container">
        <button on:pointerdown={onPointerDown}>
            {$timerStore.status === TIMER_STATUS.STOPPED ? LABELS.START :
              $timerStore.status === TIMER_STATUS.PAUSED ? LABELS.RESUME : LABELS.PAUSE}
        </button>
        <button on:pointerdown={onPointerDown}>
            {$timerStore.sessionType === SESSION_TYPES.WORK ? LABELS.STOP : LABELS.SKIP_BREAK}
        </button>
        <button on:pointerdown={onPointerDown}>
            {LABELS.LOG}
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

.container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: fit-content;
    height: fit-content;
    border-radius: 8px;
}

.container:hover {
    background-color: rgb(38, 38, 38);
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
    background-color: rgb(38, 38, 38);
    color: white;
}
@keyframes blink {
    0%, 100% { background-color: var(--background-color); }
    50% { background-color: var(--blink-color); }
}

.blinking {
    animation: blink 1s infinite;
}

.container {
    background-color: var(--background-color);
}
</style>
