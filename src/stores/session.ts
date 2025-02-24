import { writable, get, Writable } from 'svelte/store';
import { SESSION_TYPES } from '../constants';
import type { SessionState } from '../types/index';
import { flowStore } from './flow';
import { breakStore } from './break';

const DEFAULT_SESSION_STATE: SessionState = {
    type: 'work',
    completed: 0,
    lastSessionDuration: undefined,
    suggestedNextDuration: undefined,
    sessionGrowth: undefined
};

export class SessionStore {
    private store: Writable<SessionState>;

    constructor() {
        const savedState = localStorage.getItem('sessionStore');
        let initialState = savedState ? JSON.parse(savedState) : DEFAULT_SESSION_STATE;
        // Ensure we always start with a work session when initializing
        initialState = { ...initialState, type: SESSION_TYPES.WORK };
        this.store = writable<SessionState>(initialState);

        // Setup persistent storage
        this.store.subscribe((state: SessionState) => {
            localStorage.setItem('sessionStore', JSON.stringify(state));
        });
    }

    get state(): SessionState {
        return get(this.store);
    }

    subscribe = (run: (value: SessionState) => void): (() => void) => {
        return this.store.subscribe(run);
    }

    update = (updater: (state: SessionState) => SessionState): void => {
        this.store.update(updater);
    }

    set(partial: Partial<SessionState>): void {
        this.update(s => ({ ...s, ...partial }));
    }

    start(): void {
        this.update(s => ({
            ...s,
            type: 'work',
            completed: s.completed + 1
        }));
    }

    complete(duration: number): void {
        const currentState = this.state;
        const isWorkSession = currentState.type === SESSION_TYPES.WORK;

        if (isWorkSession) {
            this.update(s => ({
                ...s,
                lastSessionDuration: duration
            }));
        } else {
            const completedSessions = currentState.completed + 1;

            this.update(s => ({
                ...s,
                type: SESSION_TYPES.WORK,
                completed: completedSessions,
                lastSessionDuration: duration
            }));

            breakStore.set({
                type: needLongBreak ? 'long' : 'short'
            });
        }
    }

    reset(): void {
        this.store.set(DEFAULT_SESSION_STATE);
    }
}

export const sessionStore = new SessionStore();
