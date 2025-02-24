import { writable, Writable, get } from 'svelte/store';
import { FLOW_STATUS, BREAK_TYPE } from '../constants';
import type { FlowState, FlowStatus, SessionType, BreakType } from '../types/index';
import { timerStore } from './timer';
import { sessionStore } from './session';
import { breakStore } from './break';


const DEFAULT_FLOW_STATE: FlowState = {
    rating: null,
    ratingHistory: [],
    status: 'distracted',
    streak: 0,
    averageFocusLevel: 0,
    lastFocusRatings: [],
    dailyFlowSessions: 0,
    dailyFlowGoal: 3,
    isActive: false,
    prompt: {
        isActive: false
    }
};

export class FlowStore {
    private store: Writable<FlowState>;

    constructor() {
        const savedState = localStorage.getItem('flowStore');
        const initialState = savedState ? JSON.parse(savedState) : DEFAULT_FLOW_STATE;
        this.store = writable<FlowState>(DEFAULT_FLOW_STATE);

        // Setup persistent storage
        this.store.subscribe((state: FlowState) => {
            localStorage.setItem('flowStore', JSON.stringify(state));
        });
    }

    get state(): FlowState {
        return get(this.store);
    }

    subscribe = (run: (value: FlowState) => void): (() => void) => {
        return this.store.subscribe(run);
    }

    update = (updater: (state: FlowState) => FlowState): void => {
        this.store.update(updater);
    }

    set(partial: Partial<FlowState>): void {
        this.update(s => ({ ...s, ...partial }));
    }

    set status(status: FlowStatus) {
        const isActive = status === FLOW_STATUS.FLOW;
        const currentState = this.state;
        const streak = isActive ? currentState.streak + 1 : 0;
        const lastFocusRatings = [...currentState.lastFocusRatings, status].slice(-5);

        const shouldTakeBreak = !isActive && streak === 0;
        const nextSessionType = shouldTakeBreak ? 'break' : 'work';

        // Update session store
        sessionStore.set({
            ...sessionStore.state,
            type: nextSessionType as SessionType,
        });

        // Update flow store state
        this.set({
            ...this.state,
            status,
            streak,
            lastFocusRatings,
            isActive,
            ratingHistory: [...currentState.ratingHistory, status],
            dailyFlowSessions: isActive ? currentState.dailyFlowSessions + 1 : currentState.dailyFlowSessions,
            prompt: {
                ...currentState.prompt,
                isActive: false
            }
        });
    }

    reset(): void {
        this.store.set(DEFAULT_FLOW_STATE);
    }

    getAverageFlowRating(): number {
        const history = this.state.ratingHistory;
        if (!history.length) return 0;

        const flowCount = history.filter(status => status === FLOW_STATUS.FLOW).length;
        return Math.round((flowCount / history.length) * 100) / 100;
    }

    getDailyProgress(): { current: number; goal: number; percentage: number } {
        const { dailyFlowSessions, dailyFlowGoal } = this.state;
        return {
            current: dailyFlowSessions,
            goal: dailyFlowGoal,
            percentage: (dailyFlowSessions / dailyFlowGoal) * 100
        };
    }

    shouldTakeBreak(): BreakType {
        switch (this.state.status) {
            case FLOW_STATUS.DISTRACTED:
                return BREAK_TYPE.REQUIRED;
            case FLOW_STATUS.FLOW:
                return this.state.streak > 4 ? BREAK_TYPE.SUGGESTED : BREAK_TYPE.OPTIONAL;
            case FLOW_STATUS.FOCUSED:
                return this.state.streak > 2 ? BREAK_TYPE.SUGGESTED : BREAK_TYPE.OPTIONAL;
            case FLOW_STATUS.OK:
                return BREAK_TYPE.SUGGESTED;
            default:
                return BREAK_TYPE.SUGGESTED;
        }
    }
}

export const flowStore = new FlowStore();
