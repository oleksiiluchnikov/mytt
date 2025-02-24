export interface TimerConfig {
    workDuration: number;
    shortBreakDuration: number;
    longBreakDuration: number;
    sessionsBeforeLongBreak: number;
    minimumDuration?: number;
    maximumDuration?: number;
}

export interface BehaviorConfig {
    annoyingLevel: 'low' | 'medium' | 'high';
    theme: 'light' | 'dark';
}

export interface SystemConfig {
    frontmostApp?: string;
}

export interface AppConfig {
    timer: TimerConfig;
    behavior: BehaviorConfig;
    system: SystemConfig;
}

export interface ConfigState extends AppConfig {
    isLoaded: boolean;
    isError: boolean;
    errorMessage?: string;
}
