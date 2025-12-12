// Matches src-tauri/src/ipc/dto.rs
export interface AppStateDto {
    status: 'idle' | 'running' | 'paused' | 'ratingPrompt' | 'decision';
    phase: 'work' | 'shortBreak' | 'longBreak';
    plannedDurationS: number;
    remainingMs: number;
    workSessionsCompleted: number;
    flowStreak: number;
    lastRating?: 'distracted' | 'ok' | 'focused' | 'flow';
    nextWorkS?: number;
    breakSuggestion?: BreakSuggestion;
    dailySessionsCompleted: number;
    dailyGoal: number;
}

type BreakSuggestion =
  | { type: 'none' }
  | { type: 'optional', duration: number }
  | { type: 'suggested', duration: number }
  | { type: 'required', duration: number };



// Helper types for UI components
export type FlowRating = 'distracted' | 'ok' | 'focused' | 'flow';


export type IntentType =
    | 'start'
    | 'pause'
    | 'resume'
    | 'stop'
    | 'rate'
    | 'chooseBreak'
    | 'skipBreak'
    | 'log';

export interface IntentDto {
    type: IntentType; // Stricter!
    [key: string]: any;
}
