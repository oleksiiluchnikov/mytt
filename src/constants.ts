export const LABELS = {
    START: 'start',
    PAUSE: 'pause',
    RESUME: 'resume',
    STOP: 'stop',
    LOG: 'log',
    SKIP_BREAK: 'skip',
    FLOW: 'In Flow',
    FOCUSED: 'Focused',
    OK: 'OK',
    DISTRACTED: 'Distracted'
} as const;

export const SESSION_TYPES = {
    WORK: 'work',
    SHORT_BREAK: 'shortBreak',
    LONG_BREAK: 'longBreak'
} as const;

export const TIMER_STATUS = {
    RUNNING: 'running',
    PAUSED: 'paused',
    STOPPED: 'stopped'
} as const;

export const FLOW_STATE = {
    DISTRACTED: 'distracted',
    OK: 'ok',
    FOCUSED: 'focused',
    FLOW: 'flow'
} as const;


export const DURATIONS = {
    MINIMUM: 2 * 60, // 2 minutes in seconds
    MAXIMUM: 50 * 60, // 50 minutes in seconds
    DEFAULT_START: 2 * 60, // 2 minutes in seconds (starting small)
    LOW_MOTIVATION_START: 2 * 60, // 2 minutes for low motivation
    FLOW_EXTENSION: 25 * 60, // 25 minutes extension when in flow
    INCREMENT: {
        DISTRACTED: 2 * 60, // 2 minutes increment when previously distracted
        OK: 3 * 60, // 3 minutes increment when doing ok
        FOCUSED: 5 * 60, // 5 minutes increment when focused
        FLOW: 25 * 60 // 25 minutes increment when in flow
    },
    BREAKS: {
        DISTRACTED: 5 * 60, // 5 minutes break when distracted
        OK: 3 * 60, // 3 minutes break when doing ok
        FOCUSED: 2 * 60, // 2 minutes optional break when focused
        FLOW: 0 // no break when in flow
    }
} as const;

export const SESSION_GROWTH = {
    DECREASE: 'decrease',
    MAINTAIN: 'maintain',
    INCREASE: 'increase'
} as const;

export const BREAK_TYPE = {
    OPTIONAL: 'optional', // When in flow state
    SUGGESTED: 'suggested', // When focused but not in flow
    REQUIRED: 'required' // When distracted or after long sessions
} as const;

export const FLOW_THRESHOLDS = {
    MIN_FLOW_RATING: 3, // Minimum rating to be considered in flow
    FLOW_STREAK_THRESHOLD: 3, // Number of flow sessions to suggest longer durations
    MAX_NO_BREAK_SESSIONS: 4, // Maximum sessions before requiring a break
    DAILY_FLOW_GOAL: 5 // Default daily goal for flow sessions
} as const;

export const DURATION_SUGGESTIONS = {
    FLOW: {
        MIN_EXTENSION: 5 * 60,
        MAX_EXTENSION: 15 * 60
    },
    FOCUSED: {
        INCREMENT: 5 * 60
    },
    OK: {
        MAINTAIN: true
    },
    DISTRACTED: {
        REDUCTION: 5 * 60
    }
} as const;

export const FOCUS_STATES = {
    DISTRACTED: { value: 1, label: 'Distracted', description: 'Had trouble focusing' },
    SOMEWHAT_FOCUSED: { value: 2, label: 'Somewhat Focused', description: 'Maintained some focus' },
    HIGHLY_FOCUSED: { value: 3, label: 'Highly Focused', description: 'Deep concentration' },
    FLOW: { value: 4, label: 'Flow', description: 'In the zone' }
} as const;

export const ANNOYING_LEVELS = {
    LOW: 'low',
    MEDIUM: 'medium',
    HIGH: 'high'
} as const;

export const THEMES = {
    LIGHT: 'light',
    DARK: 'dark'
} as const;
