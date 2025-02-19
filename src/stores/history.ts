import { writable } from 'svelte/store';
import type { FlowState } from '../types';

export interface SessionRecord {
    startTime: Date;
    duration: number;
    focusRating: FlowState;
    breakTaken: boolean;
    breakDuration?: number;
}

function createHistoryStore() {
    const { subscribe, update } = writable<SessionRecord[]>([]);

    return {
        subscribe,
        addSession: (record: SessionRecord) => update(history => [...history, record]),
        clearHistory: () => update(() => []),
    };
}

export const historyStore = createHistoryStore();
