import { DURATIONS, FLOW_STATUS } from '../constants';
import type { FlowStatus } from '../types';

interface DurationAdjustment {
    newDuration: number;
    suggestedBreakDuration: number;
    reason: string;
}

export class DurationManager {
    private readonly minDuration: number;
    private readonly maxDuration: number;
    private readonly baseIncrements: Record<FlowStatus, number>;

    constructor(
        minDuration: number = DURATIONS.MINIMUM,
        maxDuration: number = DURATIONS.MAXIMUM
    ) {
        this.minDuration = minDuration;
        this.maxDuration = maxDuration;
        this.baseIncrements = {
            [FLOW_STATUS.FLOW]: DURATIONS.INCREMENT.FLOW,
            [FLOW_STATUS.FOCUSED]: DURATIONS.INCREMENT.FOCUSED,
            [FLOW_STATUS.OK]: DURATIONS.INCREMENT.OK,
            [FLOW_STATUS.DISTRACTED]: -DURATIONS.INCREMENT.DISTRACTED
        };
    }

    adjustDuration(
        currentDuration: number,
        flowStatus: FlowStatus,
        flowStreak: number
    ): DurationAdjustment {
        const baseIncrement = this.baseIncrements[flowStatus];
        let adjustedIncrement = baseIncrement;
        let reason = '';

        // Progressive scaling based on current duration and flow status
        if (currentDuration < 10 * 60) { // Under 10 minutes
            adjustedIncrement = Math.floor(baseIncrement * 0.5); // Smaller increments
        } else if (currentDuration >= 25 * 60) { // 25+ minutes
            adjustedIncrement = Math.floor(baseIncrement * 0.25); // Very conservative increases
        }

        // Apply streak multiplier for FLOW state
        if (flowStatus === FLOW_STATUS.FLOW && flowStreak > 0) {
            const multiplier = Math.min(flowStreak, 3) * 0.3; // More conservative multiplier
            adjustedIncrement = Math.floor(adjustedIncrement * (1 + multiplier));
            reason = `Flow streak (${flowStreak}x) bonus applied`;
        } else {
            reason = this.getAdjustmentReason(flowStatus);
        }

        // Calculate new duration
        let newDuration = currentDuration + adjustedIncrement;
        newDuration = Math.max(this.minDuration, Math.min(newDuration, this.maxDuration));

        // Calculate suggested break duration based on flow status
        const suggestedBreakDuration = this.calculateBreakDuration(flowStatus, flowStreak);

        return {
            newDuration,
            suggestedBreakDuration,
            reason
        };
    }

    private getAdjustmentReason(flowStatus: FlowStatus): string {
        switch (flowStatus) {
            case FLOW_STATUS.FLOW:
                return 'Extended due to flow state';
            case FLOW_STATUS.FOCUSED:
                return 'Increased due to good focus';
            case FLOW_STATUS.OK:
                return 'Maintained current duration';
            case FLOW_STATUS.DISTRACTED:
                return 'Reduced due to distraction';
            default:
                return 'Standard adjustment';
        }
    }

    private calculateBreakDuration(flowStatus: FlowStatus, flowStreak: number): number {
        switch (flowStatus) {
            case FLOW_STATUS.FLOW:
                return flowStreak > 3 ? DURATIONS.BREAKS.FOCUSED : DURATIONS.BREAKS.OK;
            case FLOW_STATUS.FOCUSED:
                return DURATIONS.BREAKS.FOCUSED;
            case FLOW_STATUS.OK:
                return DURATIONS.BREAKS.OK;
                return Math.min(10 * 60, this.maxDuration); // 10 minutes
            case FLOW_STATUS.DISTRACTED:
                return Math.max(5 * 60, this.minDuration);  // 5 minutes
            default:
                return DURATIONS.DEFAULT_START;
        }
    }

    getNextStartDuration(currentLevel: FlowStatus): number {
        switch (currentLevel) {
            case FLOW_STATUS.FLOW:
                return Math.min(15 * 60, this.maxDuration); // 15 minutes
            case FLOW_STATUS.FOCUSED:
                return Math.min(10 * 60, this.maxDuration); // 10 minutes
            case FLOW_STATUS.OK:
                return Math.min(5 * 60, this.maxDuration); // 5 minutes
            case FLOW_STATUS.DISTRACTED:
                return Math.max(2 * 60, this.minDuration);  // 2 minutes
            default:
                return DURATIONS.DEFAULT_START;
        }
    }
}
