
import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';

export const getContentSize = async () => {
    const mainElement = document.querySelector('main');
    if (!mainElement) return { width: 240, height: 200 };

    // Get the actual content height including all children
    const contentHeight = mainElement.scrollHeight;

    const { height: outerHeight } = await getCurrentWindow().outerSize();
    const { height: innerHeight } = await getCurrentWindow().innerSize();
    const titleBarHeight = Math.max(outerHeight - innerHeight, 30);

    // Add some padding to prevent scrollbars
    const padding = 10;

    return {
        width: 240,
        height: Math.ceil(contentHeight + titleBarHeight + padding)
    };
};

export const debounce = (func: { (): Promise<void>; apply?: any; }, wait: number) => {
    let timeout: number;
    return (...args: any) => {
        clearTimeout(timeout);
        timeout = setTimeout(() => func.apply(this, args), wait);
    };
};

export const setRandomPosition = async () => {
    const appWindow = getCurrentWindow();
    const { width: screenWidth, height: screenHeight } = await appWindow.primaryMonitor();
    const { width: windowWidth, height: windowHeight } = await appWindow.innerSize();

    const maxX = screenWidth - windowWidth;
    const maxY = screenHeight - windowHeight;

    const x = Math.floor(Math.random() * maxX);
    const y = Math.floor(Math.random() * maxY);

    await appWindow.setPosition(new LogicalPosition(x, y));
};

export const setRandomSize = async () => {
    const baseHeight = 200;
    const maxExtra = 100;
    const fixedWidth = 240;

    const randomHeight = baseHeight + Math.floor(Math.random() * maxExtra);

    await getCurrentWindow().setSize(new LogicalSize(fixedWidth, randomHeight));
};

export const annoyUser = async (level: 'low' | 'medium' | 'high') => {
    if (level === 'low') return;

    if (level === 'medium') {
        await setRandomPosition();
    }

    if (level === 'high') {
        await setRandomPosition();
        await setRandomSize();
    }
};

const resizeDebounced = debounce(async () => {
    const size = await getContentSize();
    await getCurrentWindow().setSize(new LogicalSize(size.width, size.height));
}, 100);

export const resizeWindow = () => {
    resizeDebounced();
};
