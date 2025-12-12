import { writable, derived } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { AppStateDto, IntentDto } from '../types/index';

const DEFAULT_STATE: AppStateDto = {
    status: 'idle',
    phase: 'work',
    plannedDurationS: 1500,
    remainingMs: 1500000,
    workSessionsCompleted: 0,
    flowStreak: 0
};

function createAppStore() {
    const { subscribe, set, update } = writable<AppStateDto>(DEFAULT_STATE);

    return {
        subscribe,

        // Connect to Tauri backend
        init: async () => {
            // 1. Get initial state synchronously-ish
            try {
                const state = await invoke<AppStateDto>('get_state');
                set(state);
            } catch (e) {
                console.error('Failed to get state:', e);
            }

            // 2. Listen for updates
            const unlisten = await listen<AppStateDto>('state', (event) => {
                set(event.payload);
            });

            return unlisten;
        },

        // Send intents to backend
        intent: (intent: IntentDto) => {
            invoke('intent', { payload: intent }).catch(console.error);
        }
    };
}

export const appStore = createAppStore();

// Derived store for formatted time
export const timeDisplay = derived(appStore, $s => {
    const totalSeconds = Math.ceil($s.remainingMs / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
});

// Derived store for progress percentage
export const progressPct = derived(appStore, $s => {
    if ($s.status === 'idle') return 0;
    const totalMs = $s.plannedDurationS * 1000;
    if (totalMs === 0) return 0;
    // Calculate percentage remaining (100 -> 0) or elapsed (0 -> 100)?
    // Usually progress bars fill up.
    // Elapsed = Total - Remaining
    const elapsed = totalMs - $s.remainingMs;
    return Math.min(100, (elapsed / totalMs) * 100);
});
