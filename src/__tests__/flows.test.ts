// src/__tests__/flows.test.ts
import { test, expect } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import App from '../App.svelte';

function mockTimerEnd() {
    return new Promise(resolve => {
        setTimeout(() => {
            resolve();
        }, 1000);
    });
}

test('Progressive flow: Start → Rate → Continue', async () => {
    const { getByRole, getByText } = render(App);

    // Start short session
    await fireEvent.click(getByRole('button', { name: '5m' }));
    expect(getByText(/running/i)).toBeInTheDocument();

    // Fast-forward timer (mock Tauri events)
    await mockTimerEnd();

    // Rate as Flow
    await fireEvent.click(getByRole('button', { name: 'Flow' }));

    // Should show increased duration
    expect(getByText(/Next: \d+m/)).toHaveTextContent('10m');

    // Continue without break
    await fireEvent.click(getByRole('button', { name: 'Continue Work' }));

    // Verify running again
    expect(getByText(/running/i)).toBeInTheDocument();
});
