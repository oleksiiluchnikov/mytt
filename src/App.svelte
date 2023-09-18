<script>
/**
 * @file Manages the main functionality of a timer application.
 * @author Oleksii
 */

import { invoke } from '@tauri-apps/api/tauri';
import { writable } from 'svelte/store';

/** Timer status. It can be 'running', 'paused', or 'stopped'. */
const timerStatus = writable('stopped');

/** Displayed time format in 'HH:MM:SS' */
let displayTime = '00:00:00';
let frontmostApp = '';

/**
 * Invokes a backend API action.
 * @async
 * @param {string} action - The action to invoke.
 * @returns {Promise<any>} The result of the invoked action.
 */
async function invokeAPI(action) {
  try {
    return await invoke(action);
  } catch (error) {
    // Handle errors through UI or logging
    console.error(`Failed to invoke ${action}: ${error}`);
  }
}

/**
 * Toggles the timer between 'paused' and 'running'.
 * @async
 */
async function toggleTimer() {
  if ($timerStatus !== 'running') {
    await invokeAPI('start_timer');
    timerStatus.set('running');
  } else {
    await invokeAPI('pause_timer');
    timerStatus.set('paused');
  }
}

/**
 * Stops and resets the timer.
 * @async
 */
async function stopAndReset() {
  await invokeAPI('stop_timer');
  await invokeAPI('reset_timer');
  displayTime = '00:00:00';
  timerStatus.set('stopped');
}

/**
 * Updates the display time.
 * @async
 */
async function updateDisplay() {
  if ($timerStatus === 'running') {
    displayTime = await invokeAPI('get_time');
  }
  await getFrontmostApp();
}

/** @type {number} ID of the setInterval function. */
let intervalId;

/**
 * Timer Status Subscriber
 * @param {string} status - The current timer status.
 */
timerStatus.subscribe(status => {
  clearInterval(intervalId);
  if (status === 'running') {
    intervalId = setInterval(updateDisplay, 1000);
  }
});

/**
 * Submits the current time to Obsidian.
 * @async
 */
async function submitToObsidian() {
  if ($timerStatus === 'running') {
    await invokeAPI('submit_to_obsidian');
  }
}

async function getFrontmostApp() {
  frontmostApp = await invokeAPI('get_frontmost_application');
}


</script>


<main class="container">
  <div class="clock-wrapper">
    <div class="frontmost-application-name">
    {frontmostApp}
    </div>
    <h1>{displayTime}</h1>
  <div class="buttons-container">
<button on:click={toggleTimer}>
  { $timerStatus === 'paused' ? 'Resume' : ($timerStatus === 'running' ? 'Pause' : 'Start') }
</button>
<button on:click={stopAndReset}>
  { $timerStatus === 'stopped' ? 'Reset' : 'Stop' }
</button>

<button on:click={submitToObsidian}>
  Submit
</button>
</div>
</div>
</main>

<style>

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
    height: 70px; /* Adjust the height to fit a 200px rectangle */
    padding: 12px; /* Optional padding for spacing */
  }
  .clock-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 5px;
  }

  .buttons-container {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    gap: 2px;
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

  h1 {
    margin: 0;
    font-size: 2.6rem; /* Adjust the font size as needed */
    font-family: "San Francisco Display", sans-serif;
  }
</style>

