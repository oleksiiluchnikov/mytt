// Basic Types
export type Theme = 'light' | 'dark';
export type AnnoyingLevel = 'low' | 'medium' | 'high';

// Time and Progress Types
export interface TimeState {
    total: number;
    remaining: number;
    display: string;
}

export interface ProgressState {
    total: number;
    percentage: number;
    color: string;
}

// Timer Related Types
export type TimerStatus = 'running' | 'paused' | 'stopped';

export interface TimerStoreState {
    time: TimeState;
    progress: ProgressState;
    status: TimerStatus;
    preferences: TimerPreferences;
    system: SystemState;
    ui: UIState;
}

export interface TimerPreferences {
    workDuration: number;
    shortBreakDuration: number;
    longBreakDuration: number;
    sessionsBeforeLongBreak: number;
    minimumDuration: number;
    maximumDuration: number;
    annoyingLevel: AnnoyingLevel;
}

export interface FlowPromptState {
    isActive: boolean;
}

// Flow and Focus Related Types
export type FlowStatus = 'distracted' | 'ok' | 'focused' | 'flow';

export interface FlowState {
    rating: FlowStatus | null;
    ratingHistory: FlowStatus[];
    status?: FlowStatus;
    streak: number;
    averageFocusLevel: number;
    lastFocusRatings: FlowStatus[];
    dailyFlowSessions: number;
    dailyFlowGoal: number;
    isActive: boolean;
    prompt: FlowPromptState;
}

export interface FocusState {
    level: FlowStatus;
    durations: number[];
    current: number;
}

// Session and Break Related Types
export type SessionType = 'work' | 'shortBreak' | 'longBreak';
export type SessionGrowth = 'decrease' | 'maintain' | 'increase';
export type BreakType = 'optional' | 'suggested' | 'required';

export interface SessionState {
    type: SessionType;
    completed: number;
    lastSessionDuration?: number;
    suggestedNextDuration?: number;
    sessionGrowth?: SessionGrowth;
}

export interface BreakState {
    type: BreakType;
    skipped?: boolean;
    duration?: number;
}

// System and UI Related Types
export interface SystemState {
    frontmostApp: string;
    showFlowPrompt: boolean;
}

export interface UIState {
    theme: Theme;
}
