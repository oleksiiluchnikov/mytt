/**
 * Timer Store Module
 *
 * A comprehensive timer implementation for managing work sessions, breaks, and flow states.
 * Features include:
 * - Work/break session management
 * - Flow state tracking
 * - Progress tracking and statistics
 * - Configurable durations and settings
 * - Local storage persistence
 *
 * @module TimerStore
 */

import { writable, get, Writable } from 'svelte/store';
import { TIMER_STATUS, SESSION_TYPES, ANNOYING_LEVELS, FLOW_STATUS as FLOW_STATUS, DURATIONS, BREAK_TYPE } from '../constants';
import { flowStore } from './flow';
import { sessionStore } from './session';
import { notify } from '../utils/notification';
import { invoke } from '@tauri-apps/api/core';
import * as utils from '../utils/time';

import type {
    BreakType,
    FlowStatus,
    TimerStoreState,
} from '../types/index';

/**
 * Default state configuration for the timer store.
 * All durations are in seconds.
 */
const DEFAULT_STATE: TimerStoreState = {
    time: {
        total: DURATIONS.LOW_MOTIVATION_START,
        remaining: DURATIONS.LOW_MOTIVATION_START,
        display: utils.formatTime(DURATIONS.LOW_MOTIVATION_START)
    },
    progress: {
        total: DURATIONS.LOW_MOTIVATION_START,
        percentage: 0,
        color: 'rgb(255, 0, 0)'
    },
    status: TIMER_STATUS.STOPPED,
    preferences: {
        workDuration: DURATIONS.LOW_MOTIVATION_START,
        shortBreakDuration: 300,
        longBreakDuration: 900,
        sessionsBeforeLongBreak: 4,
        minimumDuration: DURATIONS.MINIMUM,
        maximumDuration: DURATIONS.MAXIMUM,
        annoyingLevel: ANNOYING_LEVELS.LOW
    },

    system: {
        frontmostApp: '',
        showFlowPrompt: false
    },
    ui: {
        theme: 'light'
    }
};

/**
 * Defines the public interface for the TimerStore.
 * All methods that can be called from outside the store should be defined here.
 */
interface ITimerStore {
    subscribe: (callback: (value: TimerStoreState) => void) => () => void;
    update: (updater: (state: TimerStoreState) => TimerStoreState) => void;
    set: (partial: Partial<TimerStoreState>) => void;
    start: () => void;
    pause: () => void;
    resume: () => void;
    stop: () => void;
    reset: () => void;
    destroy: () => void;
}

/**
 * Implementation of the timer store that manages work sessions, breaks, and flow states.
 * @implements {ITimerStore}
 */
class TimerStore implements ITimerStore {
    store: Writable<TimerStoreState>;
    private animationFrameId: number | undefined;
    private lastUpdateTime: number | undefined;
    private intervalId: number | undefined;

    // Getters for commonly accessed state
    get state(): TimerStoreState {
        return get(this.store);
    }

    // Setter for state updates
    set state(value: TimerStoreState) {
        this.store.set(value);
    }

    constructor() {
        const savedState = localStorage.getItem('timerStore');
        const initialState = savedState ? JSON.parse(savedState) : DEFAULT_STATE;

        // Ensure time display matches the actual duration
        if (initialState.time) {
            initialState.time.display = utils.formatTime(initialState.time.remaining);
        }

        this.store = writable<TimerStoreState>(DEFAULT_STATE);
        this.animationFrameId = undefined;
        this.lastUpdateTime = undefined;

        // Setup persistent storage
        const unsubscribe = this.store.subscribe((state: TimerStoreState) => {
            localStorage.setItem('timerStore', JSON.stringify(state));
        });

        // Ensure cleanup when store is destroyed
        if (typeof window !== 'undefined') {
            window.addEventListener('beforeunload', () => {
                unsubscribe();
            });
        }
    }

    subscribe = (run: (value: TimerStoreState) => void): (() => void) => {
        return this.store.subscribe(run);
    }

    update = (updater: (state: TimerStoreState) => TimerStoreState): void => {
        this.store.update(updater);
    }

    set(partial: Partial<TimerStoreState>): void {
        this.update(s => ({...s, ...partial}));
    }

    destroy(): void {
        this.stop();
        this.store.set(DEFAULT_STATE);
    }

    start(): void {
        this.lastUpdateTime = Date.now();

        // Use setInterval for consistent timer updates
        this.intervalId = window.setInterval(() => {
            const currentTime = Date.now();
            const deltaTime = currentTime - (this.lastUpdateTime || currentTime);
            const secondsToDecrease = Math.floor(deltaTime / 1000);

            if (this.state.status === TIMER_STATUS.STOPPED) {
                this.stop();
                return;
            }

            if (this.state.status === TIMER_STATUS.PAUSED) {
                return;
            }

            // Update progress bar
            const remaining = Math.max(0, this.state.time.remaining);
            const total = this.state.time.total;
            const percentage = Math.max(0, Math.min(100, (remaining / total) * 100));

            this.update(s => ({
                ...s,
                progress: {
                    ...s.progress,
                    percentage
                }
            }));

            if (secondsToDecrease <= 0) {
                return;
            }

            const newRemainingTime = Math.max(0, this.state.time.remaining - secondsToDecrease);
            this.update(s => ({
                ...s,
                time: {
                    ...s.time,
                    remaining: newRemainingTime,
                    display: utils.formatTime(newRemainingTime)
                }
            }));

            this.lastUpdateTime = currentTime - (deltaTime % 1000);

            if (newRemainingTime !== 0) {
                return;
            }

            // if (sessionStore.state.type === SESSION_TYPES.WORK) {
            switch (true) {
                case sessionStore.state.type === SESSION_TYPES.WORK:
                    notify({
                        title: "Work Session Complete!",
                        body: "Time for a break! Keep up the good work!",
                    });
                    this.set({
                        status: TIMER_STATUS.STOPPED,
                        time: {
                            ...this.state.time,
                            remaining: this.state.preferences.workDuration,
                            display: utils.formatTime(this.state.preferences.workDuration)
                        },
                    });
                    break;
                default:
                    notify({
                        title: "Break Time Over",
                        body: "Ready to get back to work?",
                    });
                    sessionStore.set({
                        type: SESSION_TYPES.WORK,
                        completed: sessionStore.state.completed + 1,
                        lastSessionDuration: undefined,
                        suggestedNextDuration: undefined,
                        sessionGrowth: undefined
                    });
                    this.set({
                        time: {
                            total: this.state.preferences.workDuration,
                            remaining: this.state.preferences.workDuration,
                            display: utils.formatTime(this.state.preferences.workDuration)
                        },
                        status: TIMER_STATUS.STOPPED,
                    });
            }
            this.stop();

        }, 100);

        // Use requestAnimationFrame for smooth UI updates
        const animate = () => {
            if (this.state.status === TIMER_STATUS.RUNNING) {
                this.animationFrameId = requestAnimationFrame(animate);
            }
        };
        this.animationFrameId = requestAnimationFrame(animate);

        if (this.state.time.remaining <= 0) {
            this.update(s => ({
                ...s,
                time: {
                    ...s.time,
                    remaining: this.state.preferences.workDuration,
                    display: utils.formatTime(s.preferences.workDuration)
                },
            }));
        }

        flowStore.set({
            ...flowStore.state,
            prompt: {
                ...flowStore.state.prompt,
                isActive: false
            }
        });

        this.update(s => ({
            ...s,
            status: TIMER_STATUS.RUNNING,
        }));

        invoke('on_start');
    }

    pause(): void {
        // Only pause if we're running
        if (this.state.status !== TIMER_STATUS.RUNNING) {
            return;
        }

        this.lastUpdateTime = Date.now(); // Save the exact pause time
        this.update((s: TimerStoreState) => ({
            ...s,
            status: TIMER_STATUS.PAUSED
        }));

        invoke('on_pause');
    }

    resume(): void {
        if (this.state.status !== TIMER_STATUS.PAUSED) {
            return;
        }
        this.lastUpdateTime = Date.now();
        this.update((s: TimerStoreState) => ({
            ...s,
            status: TIMER_STATUS.RUNNING
        }));

        invoke('on_resume');
    }

    stop(): void {
        this.update((s: TimerStoreState) => ({
            ...s,
            status: TIMER_STATUS.STOPPED,
            time: {
                ...s.time,
                remaining: sessionStore.state.type === 'work' ? this.state.preferences.workDuration : this.state.preferences.workDuration,
            },
        }));
        if (this.animationFrameId) {
            cancelAnimationFrame(this.animationFrameId);
            this.animationFrameId = undefined;
        }
        if (this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = undefined;
        }
        this.lastUpdateTime = undefined;

        flowStore.set({
            ...flowStore.state,
            prompt: {
                ...flowStore.state.prompt,
                isActive: true
            }
        });

        invoke('on_stop');
    }

    reset(): void {
        this.stop();
        this.state = DEFAULT_STATE;
    }

    nextDuration(currentDuration: number, flowStatus: FlowStatus, flowStreak: number, customDuration?: number): number {
        // If a custom duration was set through the progress bar, use that
        if (customDuration !== undefined) {
            const boundedDuration = Math.min(
                Math.max(customDuration, DURATIONS.MINIMUM),
                DURATIONS.MAXIMUM
            );

            this.update(s => ({
                ...s,
                preferences: {
                    ...s.preferences,
                    workDuration: boundedDuration
                },
                time: {
                    total: boundedDuration,
                    remaining: boundedDuration,
                    display: utils.formatTime(boundedDuration)
                }
            }));

            return boundedDuration;
        }

        // Otherwise use the flow-based duration adjustment logic
        let newDuration = currentDuration;
        let reason = '';

        switch (flowStatus) {
            case FLOW_STATUS.FLOW:
                reason = 'Extending duration due to flow state';
                newDuration = Math.min(
                    currentDuration + DURATIONS.INCREMENT.FLOW,
                    DURATIONS.MAXIMUM
                );
                break;
            case FLOW_STATUS.FOCUSED:
                reason = 'Increasing duration due to high focus';
                newDuration = Math.min(
                    currentDuration + DURATIONS.INCREMENT.FOCUSED,
                    DURATIONS.MAXIMUM
                );
                break;
            case FLOW_STATUS.OK:
                reason = 'Maintaining current duration';
                newDuration = Math.min(
                    currentDuration + DURATIONS.INCREMENT.OK,
                    DURATIONS.MAXIMUM
                );
                break;
            case FLOW_STATUS.DISTRACTED:
                reason = 'Reducing duration due to distraction';
                newDuration = Math.max(
                    DURATIONS.MINIMUM,
                    currentDuration + DURATIONS.INCREMENT.DISTRACTED
                );
                break;
        }

        this.update(s => ({
            ...s,
            preferences: {
                ...s.preferences,
                workDuration: newDuration
            },
            time: {
                total: newDuration,
                remaining: newDuration,
                display: utils.formatTime(newDuration)
            }
        }));

        return newDuration;
    }

    shouldTakeBreak(flowStatus: FlowStatus, flowStreak: number): BreakType {
        switch (flowStatus) {
            case FLOW_STATUS.DISTRACTED:
                return BREAK_TYPE.REQUIRED;
            case FLOW_STATUS.FLOW:
                return flowStreak > 4 ? BREAK_TYPE.SUGGESTED : BREAK_TYPE.OPTIONAL;
            case FLOW_STATUS.FOCUSED:
                return flowStreak > 2 ? BREAK_TYPE.SUGGESTED : BREAK_TYPE.OPTIONAL;
            case FLOW_STATUS.OK:
                return BREAK_TYPE.SUGGESTED;
            default:
                return BREAK_TYPE.SUGGESTED;
        }
    }
}

// Export singleton instances
export const timerStore = new TimerStore();
