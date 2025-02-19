import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from '../types/config';
import { timer } from './timer';
import { uiStore } from './ui';

const DEFAULT_CONFIG: AppConfig = {
    timer: {
        workDuration: 25 * 60,
        shortBreakDuration: 5 * 60,
        longBreakDuration: 15 * 60,
        sessionsBeforeLongBreak: 4
    },
    behavior: {
        annoyingLevel: 'low',
        theme: 'dark'
    }
};

function createConfigStore() {
    const { subscribe, set, update } = writable<AppConfig>(DEFAULT_CONFIG);

    return {
        subscribe,
        async load() {
            try {
                const config: any = await invoke('fetch_config').then(JSON.parse as any);
                const newConfig: AppConfig = {
                    timer: {
                        workDuration: config.work_duration * 60,
                        shortBreakDuration: config.short_break_duration * 60,
                        longBreakDuration: config.long_break_duration * 60,
                        sessionsBeforeLongBreak: config.sessions_before_long_break
                    },
                    behavior: {
                        annoyingLevel: config.annoying_level,
                        theme: config.theme as unknown as Theme
                    }
                };

                set(newConfig);

                // Update related stores
                timer.updateConfig({
                    workDuration: newConfig.timer.workDuration,
                    shortBreakDuration: newConfig.timer.shortBreakDuration,
                    longBreakDuration: newConfig.timer.longBreakDuration,
                    sessionsBeforeLongBreak: newConfig.timer.sessionsBeforeLongBreak,
                    annoyingLevel: newConfig.behavior.annoyingLevel as any,
                    remainingTime: newConfig.timer.workDuration
                });

                uiStore.update(s => ({ ...s, theme: newConfig.behavior.theme }));

                return newConfig;
            } catch (error) {
                console.error('Error loading config:', error);
                throw error;
            }
        },

        async save(partialConfig: Partial<AppConfig>) {
            try {
                const currentConfig = get(this);
                const newConfig = { ...currentConfig, ...partialConfig };
                await invoke('save_config', { config: JSON.stringify(newConfig) });
                set(newConfig);
                return newConfig;
            } catch (error) {
                console.error('Error saving config:', error);
                throw error;
            }
        },

        async reset() {
            try {
                await invoke('save_config', { config: JSON.stringify(DEFAULT_CONFIG) });
                set(DEFAULT_CONFIG);
                return DEFAULT_CONFIG;
            } catch (error) {
                console.error('Error resetting config:', error);
                throw error;
            }
        }
    };
}

export const configStore = createConfigStore();

export { configStore as config };
