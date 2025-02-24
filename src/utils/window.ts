import { getCurrentWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window';
import { primaryMonitor } from '@tauri-apps/api/window';

export const getContentSize = async () => {
    try {
        const mainElement = window.document.querySelector('#app');
        if (!mainElement) return { width: 160, height: 200 };

        // Get the computed styles
        const styles = window.getComputedStyle(mainElement);

        // Calculate total content height including padding and margins
        const contentHeight = mainElement.scrollHeight;
        const verticalMargin = parseFloat(styles.marginTop || '0') + parseFloat(styles.marginBottom || '0');
        const totalContentHeight = contentHeight + verticalMargin;

        // Get window metrics
        const appWindow = getCurrentWindow();
        const outerSize = await appWindow.outerSize();
        const innerSize = await appWindow.innerSize();

        // Calculate titlebar height (minimum 30px for safety)
        const titleBarHeight = Math.max(outerSize.height - innerSize.height, 30);

        // Add safety padding to prevent scrollbars
        const padding = 16;

        return {
            width: 160, // Fixed width as per design
            height: Math.ceil(totalContentHeight + titleBarHeight + padding)
        };
    } catch (error) {
        console.error('Error calculating window size:', error);
        return { width: 160, height: 200 }; // Fallback size
    }
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
    const primaryMontitor = await primaryMonitor();
    if (!primaryMontitor) return;
    const { width: screenWidth, height: screenHeight } = primaryMontitor.size;

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
    const fixedWidth = 100;

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
    try {
        const size = await getContentSize();
        const appWindow = getCurrentWindow();
        await appWindow.setSize(new LogicalSize(size.width, size.height - 14)); // 14 pixels for the title bar
    } catch (error) {
        console.error('Failed to resize window:', error);
    }
}, 50); // Reduced debounce time for smoother resizing

export const resizeWindow = () => {
    resizeDebounced();
};
