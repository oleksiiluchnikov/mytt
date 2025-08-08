import { writable, get, Writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, ConfigState } from '../types/config';
import { timerStore } from './timer';
import { uiStore } from './ui';
import { DURATIONS, ANNOYING_LEVELS } from '../constants';
import * as utils from '../utils/time';

const DEFAULT_CONFIG: ConfigState = {
    isLoaded: false,
    isError: false,
    timer: {
        workDuration: DURATIONS.DEFAULT_START,
        shortBreakDuration: DURATIONS.BREAKS.OK,
        longBreakDuration: DURATIONS.BREAKS.FLOW,
        sessionsBeforeLongBreak: 4,
        minimumDuration: DURATIONS.MINIMUM,
        maximumDuration: DURATIONS.MAXIMUM
    },
    behavior: {
        annoyingLevel: 'high',
        theme: 'dark'
    },
    system: {
        frontmostApp: ''
    }
};

export class ConfigStore {
    private store: Writable<ConfigState>;

    constructor() {
        this.store = writable<ConfigState>(DEFAULT_CONFIG);

        // Setup persistent storage
        this.store.subscribe((state: ConfigState) => {
            localStorage.setItem('configStore', JSON.stringify(state));
        });

        // Load config immediately
        this.initializeConfig();
    }

    private async initializeConfig() {
        try {
            const config: any = await invoke('fetch_config').then(JSON.parse as any) || {};
            const newConfig: AppConfig = {
                timer: {
                    workDuration: config.work_duration * 60 || DURATIONS.DEFAULT_START,
                    shortBreakDuration: config.short_break_duration * 60 || DURATIONS.BREAKS.OK,
                    longBreakDuration: config.long_break_duration * 60 || DURATIONS.BREAKS.FLOW,
                    sessionsBeforeLongBreak: config.sessions_before_long_break,
                    minimumDuration: config.minimum_duration * 60 || DURATIONS.MINIMUM,
                    maximumDuration: config.maximum_duration * 60 || DURATIONS.MAXIMUM
                },
                behavior: {
                    annoyingLevel: config.annoying_level as typeof ANNOYING_LEVELS[keyof typeof ANNOYING_LEVELS] || ANNOYING_LEVELS.HIGH,
                    theme: config.theme as 'light' | 'dark' || 'dark'
                },
                system: {
                    frontmostApp: config.frontmost_app || ''
                }
            };

            this.set({
                ...newConfig,
                isLoaded: true,
                isError: false
            });

            // Update related stores
            this.updateRelatedStores(newConfig);

            console.log(newConfig);

            return newConfig;
        } catch (error) {
            console.error('Error loading config:', error);
            this.set({
                ...DEFAULT_CONFIG,
                isError: true,
                errorMessage: error instanceof Error ? error.message : 'Unknown error'
            });
        }
    }

    get state(): ConfigState {
        return get(this.store);
    }

    subscribe = (run: (value: ConfigState) => void): (() => void) => {
        return this.store.subscribe(run);
    }

    update = (updater: (state: ConfigState) => ConfigState): void => {
        this.store.update(updater);
    }

    set(partial: Partial<ConfigState>): void {
        this.update(s => ({ ...s, ...partial }));
    }

    // Method to manually reload config if needed
    async load() {
        return this.initializeConfig();
    }

    private updateRelatedStores(config: AppConfig) {
        // Update timer store
        timerStore.set({
            time: {
                total: config.timer.workDuration,
                remaining: config.timer.workDuration,
                display: utils.formatTime(config.timer.workDuration)
            },
            preferences: {
                workDuration: config.timer.workDuration,
                shortBreakDuration: config.timer.shortBreakDuration,
                longBreakDuration: config.timer.longBreakDuration,
                sessionsBeforeLongBreak: config.timer.sessionsBeforeLongBreak,
                minimumDuration: config.timer.minimumDuration as number,
                maximumDuration: config.timer.maximumDuration as number,
                annoyingLevel: config.behavior.annoyingLevel
            }
        });

        // Update UI store
        uiStore.update(s => ({ ...s, theme: config.behavior.theme }));
    }

    async save(partialConfig: Partial<AppConfig>) {
        try {
            const currentConfig = this.state;
            const newConfig = {
                ...currentConfig,
                ...partialConfig,
                isLoaded: true,
                isError: false
            };

            await invoke('save_config', {
                config: JSON.stringify({
                    work_duration: newConfig.timer.workDuration / 60,
                    short_break_duration: newConfig.timer.shortBreakDuration / 60,
                    long_break_duration: newConfig.timer.longBreakDuration / 60,
                    sessions_before_long_break: newConfig.timer.sessionsBeforeLongBreak,
                    annoying_level: newConfig.behavior.annoyingLevel,
                    theme: newConfig.behavior.theme,
                    frontmost_app: newConfig.system.frontmostApp
                })
            });

            this.set(newConfig);
            this.updateRelatedStores(newConfig);

            return newConfig;
        } catch (error) {
            console.error('Error saving config:', error);
            this.set({
                isError: true,
                errorMessage: error instanceof Error ? error.message : 'Unknown error'
            });
            throw error;
        }
    }

    async reset() {
        try {
            await this.save(DEFAULT_CONFIG);
            return DEFAULT_CONFIG;
        } catch (error) {
            console.error('Error resetting config:', error);
            throw error;
        }
    }
}

export const configStore = new ConfigStore();
