import { writable, get } from 'svelte/store';
import { TIMER_STATUS, SESSION_TYPES, ANNOYING_LEVELS, FLOW_STATE, DURATIONS, BREAK_TYPE } from '../constants';
import type { TimerStoreState, SessionType, AnnoyingLevel, BreakType } from '../types/index';

export type FlowState = typeof FLOW_STATE[keyof typeof FLOW_STATE];

export interface TimerStore {
    subscribe: (callback: (value: TimerStoreState) => void) => () => void;
    update: (updater: (state: TimerStoreState) => TimerStoreState) => void;
    start: () => void;
    pause: () => void;
    stop: () => void;
    reset: () => void;
    setSessionType: (type: SessionType) => void;
    setRemainingTime: (time: number) => void;
    setAnnoyingLevel: (level: AnnoyingLevel) => void;
    setFrontmostApp: (app: string) => void;
    updateConfig: (config: Partial<TimerStoreState>) => void;
    destroy: () => void;
    skipBreak: () => void;
    openLog: () => void;
    setFlowState: (state: FlowState) => void;
    completeSession: () => void;
    setNextWorkDuration: (duration: number) => void;
    continueInFlow: () => void;
}

const DEFAULT_STATE: TimerStoreState = {
    status: TIMER_STATUS.STOPPED,
    sessionType: SESSION_TYPES.WORK,
    remainingTime: DURATIONS.DEFAULT_START,
    showFlowPrompt: false,
    focusRating: null,
    focusRatingHistory: [],
    focusDurations: [],
    currentFocusDuration: 0,
    workDuration: DURATIONS.DEFAULT_START,
    shortBreakDuration: 300,
    longBreakDuration: 900,
    sessionsBeforeLongBreak: 4,
    annoyingLevel: ANNOYING_LEVELS.LOW,
    completedSessions: 0,
    frontmostApp: '',
    isFlowMode: false,
    minimumDuration: DURATIONS.MINIMUM,
    maximumDuration: DURATIONS.MAXIMUM,
    dailyFlowSessions: 0,
    dailyFlowGoal: 0,
    sessionGrowth: undefined,
    breakType: undefined,
    flowStreak: 0,
    averageFocusLevel: 0,
    lastFocusRatings: []
};

function createTimerStore(): TimerStore {
    const savedState = localStorage.getItem('timerStore');
    const initialState = savedState ? JSON.parse(savedState) : DEFAULT_STATE;
    const { subscribe, set, update } = writable<TimerStoreState>(initialState);

    let animationFrameId: number | undefined;
    let lastUpdateTime: number | undefined;

    subscribe((state) => {
        localStorage.setItem('timerStore', JSON.stringify(state));
    });

    function startTimer(): void {
        stopTimer();
        lastUpdateTime = performance.now();

        function tick(currentTime: number) {
            if (!lastUpdateTime) lastUpdateTime = currentTime;
            const deltaTime = currentTime - lastUpdateTime;

            if (deltaTime >= 1000) { // Update every second
                const state = get({ subscribe });

                if (state.status !== TIMER_STATUS.RUNNING) {
                    stopTimer();
                    return;
                }

                if (state.remainingTime <= 0) {
                    completeSession();
                    return;
                }

                const secondsToDecrease = Math.floor(deltaTime / 1000);
                const newRemainingTime = Math.max(0, state.remainingTime - secondsToDecrease);
                setRemainingTime(newRemainingTime);

                lastUpdateTime = currentTime - (deltaTime % 1000); // Account for leftover time
            }

            animationFrameId = requestAnimationFrame(tick);
        }

        animationFrameId = requestAnimationFrame(tick);
    }

    function stopTimer(): void {
        if (animationFrameId) {
            cancelAnimationFrame(animationFrameId);
            animationFrameId = undefined;
            lastUpdateTime = undefined;
        }
    }

    function start(): void {
        const state = get({ subscribe });
        if (state.remainingTime <= 0) {
            update((s) => ({
                ...s,
                remainingTime: s.workDuration,
                status: TIMER_STATUS.RUNNING
            }));
        } else {
            update((s) => ({ ...s, status: TIMER_STATUS.RUNNING }));
        }
        startTimer();
    }

    function pause(): void {
        update((s) => ({ ...s, status: TIMER_STATUS.PAUSED }));
        stopTimer();
    }

    function stop(): void {
        update((s) => ({
            ...s,
            status: TIMER_STATUS.STOPPED,
            remainingTime: s.workDuration,
            showFlowPrompt: s.sessionType === SESSION_TYPES.WORK
        }));
        stopTimer();
    }

    function reset(): void {
        stopTimer();
        set(DEFAULT_STATE);
    }

    function setSessionType(type: SessionType): void {
        update((s) => {
            const duration = type === SESSION_TYPES.WORK
                ? s.workDuration
                : type === SESSION_TYPES.SHORT_BREAK
                    ? s.shortBreakDuration
                    : s.longBreakDuration;

            return {
                ...s,
                sessionType: type,
                remainingTime: duration,
                status: TIMER_STATUS.STOPPED
            };
        });
        stopTimer();
    }

    function suggestNextDuration(currentDuration: number, flowState: FlowState, flowStreak: number): number {
        let increment: number;

        switch (flowState) {
            case FLOW_STATE.FLOW:
                increment = flowStreak >= 3 ? DURATIONS.INCREMENT.FLOW : DURATIONS.INCREMENT.FOCUSED;
                return Math.min(currentDuration + increment, DURATIONS.MAXIMUM);

            case FLOW_STATE.FOCUSED:
                increment = DURATIONS.INCREMENT.FOCUSED;
                return Math.min(currentDuration + increment, DURATIONS.MAXIMUM);

            case FLOW_STATE.OK:
                increment = DURATIONS.INCREMENT.OK;
                return currentDuration; // Maintain current duration when OK

            case FLOW_STATE.DISTRACTED:
                // Reduce duration when distracted, but not below minimum
                return Math.max(DURATIONS.DEFAULT_START, currentDuration - DURATIONS.INCREMENT.DISTRACTED);

            default:
                return currentDuration;
        }
    }
    // function suggestNextDuration(currentDuration: number, flowState: FlowState, flowStreak: number): number {
    //     const increment = flowStreak >= 3
    //         ? DURATIONS.INCREMENT.LARGE
    //         : flowStreak >= 1
    //             ? DURATIONS.INCREMENT.MEDIUM
    //             : DURATIONS.INCREMENT.SMALL;
    //
    //     let rating: number;
    //     switch (flowState) {
    //         case FLOW_STATE.FLOW:
    //             rating = 4;
    //             break;
    //         case FLOW_STATE.FOCUSED:
    //             rating = 3;
    //             break;
    //         case FLOW_STATE.OK:
    //             rating = 2;
    //             break;
    //         case FLOW_STATE.DISTRACTED:
    //             rating = 1;
    //             break;
    //         default:
    //             rating = 2;
    //     }
    //
    //     switch (true) {
    //         case rating === 4: // Flow
    //             return Math.min(currentDuration + increment, DURATIONS.MAXIMUM);
    //         case rating === 3: // Highly Focused
    //             return Math.min(currentDuration + (increment / 2), DURATIONS.MAXIMUM);
    //         case rating === 2: // Somewhat Focused
    //             return currentDuration;
    //         case rating === 1: // Distracted
    //             return Math.max(currentDuration - increment, DURATIONS.MINIMUM);
    //         default:
    //             return currentDuration;
    //     }
    // }

    // function shouldTakeBreak(flowState: FlowState, focusStreak: number): BreakType {
    //     let rating: number;
    //     switch (flowState) {
    //         case FLOW_STATE.FLOW:
    //             rating = 4;
    //             break;
    //         case FLOW_STATE.FOCUSED:
    //             rating = 3;
    //             break;
    //         case FLOW_STATE.OK:
    //             rating = 2;
    //             break;
    //         case FLOW_STATE.DISTRACTED:
    //             rating = 1;
    //             break;
    //         default:
    //             rating = 2;
    //     }
    //
    //     if (rating === 4 && focusStreak < 3) { // In flow but not for too long
    //         return BREAK_TYPE.OPTIONAL;
    //     } else if (rating === 4 && focusStreak >= 3) { // In flow for a while
    //         return BREAK_TYPE.SUGGESTED;
    //     } else if (rating <= 2) { // Low focus
    //         return BREAK_TYPE.REQUIRED;
    //     } else {
    //         return BREAK_TYPE.SUGGESTED;
    //     }
    // }

function shouldTakeBreak(flowState: FlowState, flowStreak: number): BreakType {
    switch (flowState) {
        case FLOW_STATE.FLOW:
            return BREAK_TYPE.OPTIONAL; // Always optional during flow

        case FLOW_STATE.FOCUSED:
            return BREAK_TYPE.SUGGESTED;

        case FLOW_STATE.OK:
            return BREAK_TYPE.SUGGESTED;

        case FLOW_STATE.DISTRACTED:
            return BREAK_TYPE.REQUIRED;

        default:
            return BREAK_TYPE.SUGGESTED;
    }
}
// function shouldTakeBreak(flowState: FlowState, flowStreak: number): BreakType {
//     switch (flowState) {
//         case FLOW_STATE.FLOW:
//             return flowStreak >= 4 ? BREAK_TYPE.SUGGESTED : BREAK_TYPE.OPTIONAL;
//
//         case FLOW_STATE.FOCUSED:
//             return flowStreak >= 3 ? BREAK_TYPE.SUGGESTED : BREAK_TYPE.OPTIONAL;
//
//         case FLOW_STATE.OK:
//             return BREAK_TYPE.SUGGESTED;
//
//         case FLOW_STATE.DISTRACTED:
//             return BREAK_TYPE.REQUIRED;
//
//         default:
//             return BREAK_TYPE.SUGGESTED;
//     }
// }
    function completeSession(): void {
        update((s) => {
            if (s.sessionType === SESSION_TYPES.WORK) {
                return {
                    ...s,
                    status: TIMER_STATUS.STOPPED,
                    showFlowPrompt: true,
                    remainingTime: 0
                };
            } else {
                const completedSessions = s.completedSessions + 1;
                const needLongBreak = completedSessions % s.sessionsBeforeLongBreak === 0;

                return {
                    ...s,
                    sessionType: SESSION_TYPES.WORK,
                    remainingTime: s.workDuration,
                    status: TIMER_STATUS.STOPPED,
                    completedSessions,
                    showFlowPrompt: false,
                    focusRating: null,
                    nextSessionType: needLongBreak ? SESSION_TYPES.LONG_BREAK : SESSION_TYPES.SHORT_BREAK
                };
            }
        });
        stopTimer();
    }

    function setRemainingTime(time: number): void {
        update((s) => ({ ...s, remainingTime: Math.max(0, Math.min(time, s.workDuration)) }));
    }

    function setAnnoyingLevel(level: AnnoyingLevel): void {
        update((s) => ({ ...s, annoyingLevel: level }));
    }

    function setFrontmostApp(app: string): void {
        update((s) => ({ ...s, frontmostApp: app }));
    }

    function updateConfig(config: Partial<TimerStoreState>): void {
        update((s) => ({ ...s, ...config }));
    }

    function skipBreak(): void {
        update((state) => ({
            ...state,
            sessionType: SESSION_TYPES.WORK,
            remainingTime: state.workDuration
        }));
    }

    function openLog(): void {
        console.log('Opening log...');
    }

    function setFlowState(state: FlowState): void {
        update((s) => {
            const isFlow = state === FLOW_STATE.FLOW;
            const newFlowStreak = isFlow ? s.flowStreak + 1 : 0;
            const nextDuration = suggestNextDuration(s.workDuration, state, newFlowStreak);
            const breakType = shouldTakeBreak(state, newFlowStreak);

            // Don't force a break if in flow state
            const shouldTakeBreakNow = isFlow ? false : breakType !== BREAK_TYPE.OPTIONAL;

            const lastFocusRatings = [...s.lastFocusRatings, state].slice(-5);

            // If in flow, continue with next duration, otherwise consider a break
            const newRemainingTime = shouldTakeBreakNow ? s.shortBreakDuration : nextDuration;
            const nextSessionType = shouldTakeBreakNow ? SESSION_TYPES.SHORT_BREAK : SESSION_TYPES.WORK;

            return {
                ...s,
                showFlowPrompt: false,
                currentFlowState: state,
                focusRating: state,
                focusRatingHistory: [...s.focusRatingHistory, state],
                suggestedNextDuration: nextDuration,
                isFlowMode: isFlow,
                status: TIMER_STATUS.STOPPED,
                sessionType: nextSessionType,
                workDuration: nextDuration,
                remainingTime: newRemainingTime,
                flowStreak: newFlowStreak,
                lastFocusRatings,
                dailyFlowSessions: isFlow ? s.dailyFlowSessions + 1 : s.dailyFlowSessions,
                breakType // Store the break type for UI decisions
            };
        });
    }
    // function setFlowState(state: FlowState): void {
    //     update((s) => {
    //         const isFlow = state === FLOW_STATE.FLOW;
    //         const newFlowStreak = isFlow ? s.flowStreak + 1 : 0;
    //         const nextDuration = suggestNextDuration(s.workDuration, state, newFlowStreak);
    //         const needsBreak = shouldTakeBreak(state, newFlowStreak);
    //
    //         const lastFocusRatings = [...s.lastFocusRatings, state].slice(-5);
    //
    //         // Set the remaining time based on either the suggested duration or break duration
    //         const newRemainingTime = needsBreak ? s.shortBreakDuration : nextDuration;
    //
    //         return {
    //             ...s,
    //             showFlowPrompt: false,
    //             currentFlowState: state,
    //             focusRating: state,
    //             focusRatingHistory: [...s.focusRatingHistory, state],
    //             suggestedNextDuration: nextDuration,
    //             isFlowMode: isFlow,
    //             status: TIMER_STATUS.STOPPED,
    //             sessionType: needsBreak ? SESSION_TYPES.SHORT_BREAK : s.sessionType,
    //             workDuration: nextDuration,
    //             remainingTime: newRemainingTime, // Apply the new remaining time
    //             flowStreak: newFlowStreak,
    //             lastFocusRatings,
    //             dailyFlowSessions: isFlow ? s.dailyFlowSessions + 1 : s.dailyFlowSessions
    //         };
    //     });
    // }
    // function setFlowState(state: FlowState): void {
    //     update((s) => {
    //         const isFlow = state === FLOW_STATE.FLOW;
    //         const newFlowStreak = isFlow ? s.flowStreak + 1 : 0;
    //         const nextDuration = suggestNextDuration(s.workDuration, state, newFlowStreak);
    //         const needsBreak = shouldTakeBreak(state, newFlowStreak);
    //
    //         const lastFocusRatings = [...s.lastFocusRatings, state].slice(-5);
    //
    //         return {
    //             ...s,
    //             showFlowPrompt: false,
    //             currentFlowState: state,
    //             focusRating: state,
    //             focusRatingHistory: [...s.focusRatingHistory, state],
    //             suggestedNextDuration: nextDuration,
    //             isFlowMode: isFlow,
    //             status: TIMER_STATUS.STOPPED,
    //             sessionType: needsBreak ? SESSION_TYPES.SHORT_BREAK : s.sessionType,
    //             workDuration: nextDuration,
    //             remainingTime: needsBreak ? s.shortBreakDuration : nextDuration,
    //             flowStreak: newFlowStreak,
    //             lastFocusRatings,
    //             dailyFlowSessions: isFlow ? s.dailyFlowSessions + 1 : s.dailyFlowSessions
    //         };
    //     });
    // }

    function setNextWorkDuration(duration: number): void {
        update((s) => {
            const newDuration = Math.max(
                Math.min(duration, DURATIONS.MAXIMUM),
                DURATIONS.MINIMUM
            );
            return {
                ...s,
                workDuration: newDuration,
                remainingTime: newDuration,
                suggestedNextDuration: undefined
            };
        });
    }

    function continueInFlow(): void {
        update((s) => ({
            ...s,
            isFlowMode: true,
            remainingTime: s.workDuration
        }));
        start();
    }

    return {
        subscribe,
        update,
        start,
        pause,
        stop,
        reset,
        setSessionType,
        setRemainingTime,
        setAnnoyingLevel,
        setFrontmostApp,
        updateConfig,
        destroy: stopTimer,
        skipBreak,
        openLog,
        setFlowState,
        completeSession,
        setNextWorkDuration,
        continueInFlow
    };
}

const timerStore = createTimerStore();

export { timerStore as timer };
