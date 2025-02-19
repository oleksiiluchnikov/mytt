import {
    isPermissionGranted,
    requestPermission,
    sendNotification,
} from '@tauri-apps/plugin-notification';
import type { Options as NotificationOptions } from '@tauri-apps/plugin-notification';

export async function notify(options: NotificationOptions): Promise<void> {
    let permissionGranted = await isPermissionGranted();

    if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
    }

    if (!options.title) {
        throw new Error('A title is required to send a notification');
    }

    if (permissionGranted) {
        sendNotification(options);
    }
}
