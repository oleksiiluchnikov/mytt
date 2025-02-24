import { writable } from 'svelte/store';
import type { UIState } from '../types/index';

export const uiStore = writable<UIState>({
    theme: 'dark'
});

export const updateStyles = (bodyColor: string, buttonColor: string) => {
    document.documentElement.style.setProperty('--body-color', bodyColor);
    document.documentElement.style.setProperty('--button-color', buttonColor);
};
