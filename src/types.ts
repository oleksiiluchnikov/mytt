import type { LABELS, SESSION_TYPES, TIMER_STATUS, FLOW_STATUS as FLOW_STATUS } from './constants';

export type Label = typeof LABELS[keyof typeof LABELS];
export type SessionType = typeof SESSION_TYPES[keyof typeof SESSION_TYPES];
export type TimerStatus = typeof TIMER_STATUS[keyof typeof TIMER_STATUS];
export type FlowStatus = typeof FLOW_STATUS[keyof typeof FLOW_STATUS];
