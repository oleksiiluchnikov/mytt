import { writable, get, Writable } from 'svelte/store';
import { BREAK_TYPE } from '../constants';
import type { BreakState, FlowStatus, BreakType } from '../types/index';
import { DURATIONS, FLOW_THRESHOLDS, FLOW_STATUS as FLOW_STATUS } from '../constants';
import { invoke } from '@tauri-apps/api/core';

const DEFAULT_BREAK_STATE: BreakState = {
    type: 'optional'
};

export class BreakStore {
    private store: Writable<BreakState>;

    constructor() {
        const savedState = localStorage.getItem('breakStore');
        const initialState = savedState ? JSON.parse(savedState) : DEFAULT_BREAK_STATE;
        this.store = writable<BreakState>(initialState);

        // Setup persistent storage
        this.store.subscribe((state: BreakState) => {
            localStorage.setItem('breakStore', JSON.stringify(state));
        });
    }

    get state(): BreakState {
        return get(this.store);
    }

    subscribe = (run: (value: BreakState) => void): (() => void) => {
        return this.store.subscribe(run);
    }

    update = (updater: (state: BreakState) => BreakState): void => {
        this.store.update(updater);
    }

    set(partial: Partial<BreakState>): void {
        this.update(s => ({ ...s, ...partial }));
    }

    skip(): void {
        // Update break state
        this.update(s => ({
            ...s,
            type: BREAK_TYPE.OPTIONAL,
            skipped: true
        }));

        invoke('on_skip_break');
    }

    start(flowStatus: FlowStatus, flowStreak: number): void {
        let breakType: BreakType;
        let breakDuration: number;

        switch (true) {
            case flowStatus === FLOW_STATUS.FLOW && flowStreak < FLOW_THRESHOLDS.MAX_NO_BREAK_SESSIONS:
                breakType = BREAK_TYPE.OPTIONAL;
                breakDuration = DURATIONS.BREAKS.FLOW;
                break;
            case flowStatus === FLOW_STATUS.DISTRACTED:
                breakType = BREAK_TYPE.REQUIRED;
                breakDuration = DURATIONS.BREAKS.DISTRACTED;
                break;
            case flowStreak >= FLOW_THRESHOLDS.MAX_NO_BREAK_SESSIONS:
                breakType = BREAK_TYPE.REQUIRED;
                breakDuration = DURATIONS.BREAKS.FOCUSED;
                break;
            default:
                breakType = BREAK_TYPE.SUGGESTED;
                breakDuration = flowStatus === FLOW_STATUS.FOCUSED ?
                    DURATIONS.BREAKS.FOCUSED : DURATIONS.BREAKS.OK;
        }

        this.update(s => ({
            ...s,
            type: breakType,
            duration: breakDuration
        }));
    }

    reset(): void {
        this.store.set(DEFAULT_BREAK_STATE);
    }
}

export const breakStore = new BreakStore();
