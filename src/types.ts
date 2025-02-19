import type { LABELS, SESSION_TYPES, TIMER_STATUS, FLOW_STATE } from './constants';

export type Label = typeof LABELS[keyof typeof LABELS];
export type SessionType = typeof SESSION_TYPES[keyof typeof SESSION_TYPES];
export type TimerStatus = typeof TIMER_STATUS[keyof typeof TIMER_STATUS];
export type FlowState = typeof FLOW_STATE[keyof typeof FLOW_STATE];
