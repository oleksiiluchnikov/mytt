import { derived } from 'svelte/store';
import { timer } from './timer';
import { TIMER_STATUS, SESSION_TYPES } from '../constants';
import type { TimerStoreState, SessionType, AnnoyingLevel } from '../types/index';

// Derived store for formatted time (MM:SS)
export const formattedTime = derived(timer, ($timer) => {
    const minutes = Math.floor($timer.remainingTime / 60);
    const seconds = $timer.remainingTime % 60;
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
});

// Helper function to get session duration
const getSessionDuration = (timer: TimerStoreState): number => {
    switch(timer.sessionType) {
        case SESSION_TYPES.WORK: return timer.workDuration;
        case SESSION_TYPES.SHORT_BREAK: return timer.shortBreakDuration;
        case SESSION_TYPES.LONG_BREAK: return timer.longBreakDuration;
        default: return timer.workDuration;
    }
};

// Derived store for total time
export const totalTime = derived<typeof timer, number>(
    timer,
    $timer => getSessionDuration($timer)
);

// Derived store for progress information
export const progress = derived<typeof timer, {
    total: number;
    remaining: number;
    percentage: number;
}>(
    timer,
    $timer => {
        const total = getSessionDuration($timer);
        const remaining = $timer.remainingTime;
        const percentage = (1 - (remaining / total)) * 100;

        return {
            total,
            remaining,
            percentage
        };
    }
);

// Color constants
const BREAK_COLOR = 'rgb(76, 175, 80)' as const;
const WORK_COLORS = {
    start: [255, 0, 0],
    end: [255, 89, 0]
} as const;

// Derived store for progress percentage
export const progressColor = derived<[typeof timer, typeof totalTime], string>(
    [timer, totalTime],
    ([$timer, $totalTime]) => {
        if ($timer.sessionType !== SESSION_TYPES.WORK) {
            return BREAK_COLOR;
        }

        const ratio = Math.min(Math.max($timer.remainingTime / $totalTime, 0), 1);
        const rgb = WORK_COLORS.start.map((start, i) =>
            Math.round(start + ratio * (WORK_COLORS.end[i] - start))
        );

        return `rgb(${rgb.join(', ')})`;
    }
);

// Derived store for checking if timer is active
export const isActive = derived(timer, ($timer) =>
    $timer.status === TIMER_STATUS.RUNNING
);

// Derived store for checking if timer is paused
export const isPaused = derived(timer, ($timer) =>
    $timer.status === TIMER_STATUS.PAUSED
);

// Derived store for checking if it's a work session
export const isWorkSession = derived(timer, ($timer) =>
    $timer.sessionType === SESSION_TYPES.WORK
);

// Derived store for checking if it's a break session
export const isBreakSession = derived(timer, ($timer) =>
    $timer.sessionType === SESSION_TYPES.SHORT_BREAK ||
    $timer.sessionType === SESSION_TYPES.LONG_BREAK
);

// Derived store for session title
export const sessionTitle = derived(timer, ($timer) => {
    switch ($timer.sessionType) {
        case SESSION_TYPES.WORK:
            return 'Work Session';
        case SESSION_TYPES.SHORT_BREAK:
            return 'Short Break';
        case SESSION_TYPES.LONG_BREAK:
            return 'Long Break';
        default:
            return 'Session';
    }
});

export const averageFocusRating = derived(timer, ($timer) =>
    $timer.focusRatingHistory.length > 0
        ? Math.round(($timer.focusRatingHistory.reduce((a: number, b: number) => a + b, 0) / $timer.focusRatingHistory.length) * 10) / 10
        : 0
);

// Derived store for stats
export const stats = derived(
    [timer, averageFocusRating],
    ([$timer, $averageFocusRating]) => ({
        completedSessions: $timer.completedSessions,
        averageFocusRating: $averageFocusRating,
        totalWorkTime: $timer.focusDurations.reduce((a: number, b: number) => a + b, 0),
        focusRatingHistory: $timer.focusRatingHistory,
    })
);

// Derived store for window title
export const windowTitle = derived([formattedTime, sessionTitle], ([$formattedTime, $sessionTitle]) => {
    return `${$formattedTime} - ${$sessionTitle}`;
});
