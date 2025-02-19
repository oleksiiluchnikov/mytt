export interface TimerConfig {
    workDuration: number;
    shortBreakDuration: number;
    longBreakDuration: number;
    sessionsBeforeLongBreak: number;
}

export interface BehaviorConfig {
    annoyingLevel: string;
    theme: string;
}

export interface AppConfig {
    timer: TimerConfig;
    behavior: BehaviorConfig;
}
