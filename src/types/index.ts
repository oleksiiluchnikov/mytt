export type TimerStatus = 'running' | 'paused' | 'stopped';
export type SessionType = 'work' | 'shortBreak' | 'longBreak';
export type AnnoyingLevel = 'low' | 'medium' | 'high';
export type Theme = 'light' | 'dark';
export type FlowState = 'distracted' | 'ok' | 'focused' | 'flow';
export type SessionGrowth = 'decrease' | 'maintain' | 'increase';
export type BreakType = 'optional' | 'suggested' | 'required';

export interface TimerStoreState {
    status: TimerStatus;
    sessionType: SessionType;
    remainingTime: number;
    showFlowPrompt: boolean;
    focusRating: FlowState | null;
    focusRatingHistory: FlowState[];
    focusDurations: number[];
    currentFocusDuration: number;
    workDuration: number;
    shortBreakDuration: number;
    longBreakDuration: number;
    sessionsBeforeLongBreak: number;
    annoyingLevel: AnnoyingLevel;
    completedSessions: number;
    frontmostApp: string;
    currentFlowState?: FlowState;
    lastSessionDuration?: number;
    isFlowMode: boolean;
    suggestedNextDuration?: number;
    minimumDuration: number;
    maximumDuration: number;
    // New fields for Progressive Pomodoro
    sessionGrowth?: SessionGrowth;
    breakType?: BreakType;
    flowStreak: number;
    averageFocusLevel: number;
    lastFocusRatings: FlowState[];
    dailyFlowSessions: number;
    dailyFlowGoal: number;
}

export interface UIStore {
    theme: Theme;
}
