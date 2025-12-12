import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

// Mirror of src-tauri/src/core/config/file_v1.rs
export interface FileConfigV1 {
    schemaVersion: number;
    timer: {
        workDefaultS?: number;
        shortBreakS?: number;
        longBreakS?: number;
    };
    behavior: {
        theme?: string;
        annoyingLevel?: string;
    };
}

const DEFAULT_CONFIG: FileConfigV1 = {
    schemaVersion: 1,
    timer: {
        workDefaultS: 1500,
        shortBreakS: 300,
        longBreakS: 900
    },
    behavior: {
        theme: 'dark',
        annoyingLevel: 'high'
    }
};

function createConfigStore() {
    const { subscribe, set, update } = writable<FileConfigV1>(DEFAULT_CONFIG);

    return {
        subscribe,

        init: async () => {
            try {
                // Fetch valid config from Rust (source of truth)
                const config = await invoke<FileConfigV1>('get_config');
                set(config);
                return config;
            } catch (e) {
                console.error('Failed to load config:', e);
                // Fallback to default if load fails
                set(DEFAULT_CONFIG);
                return DEFAULT_CONFIG;
            }
        },

        // Update specific fields and persist to disk immediately
        patch: async (partial: DeepPartial<FileConfigV1>) => {
            update(current => {
                const newConfig = {
                    ...current,
                    ...partial,
                    timer: { ...current.timer, ...partial.timer },
                    behavior: { ...current.behavior, ...partial.behavior }
                };

                // Persist to backend
                invoke('set_config', { config: newConfig }).catch(err => {
                    console.error('Failed to save config:', err);
                    // Optionally revert state here if strict consistency is needed
                });

                return newConfig;
            });
        }
    };
}

// Helper type for deep partial updates
type DeepPartial<T> = {
    [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

export const configStore = createConfigStore();
