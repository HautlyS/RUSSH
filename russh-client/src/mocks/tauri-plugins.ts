/**
 * Mock Tauri plugins for web-only mode
 */

// @tauri-apps/plugin-os
export function platform() { return 'linux'; }
export function arch() { return 'x86_64'; }
export function type() { return 'Linux'; }
export function version() { return '6.0.0'; }

// @tauri-apps/plugin-biometric
export async function checkStatus() {
  return { isAvailable: false };
}
export async function authenticate() {
  return { success: true };
}

// @tauri-apps/plugin-dialog
export async function open() { return null; }
export async function save() { return null; }
export async function message() { return undefined; }
export async function ask() { return true; }
export async function confirm() { return true; }

// @tauri-apps/plugin-haptics
export async function vibrate() {}
export async function impactFeedback() {}
export async function notificationFeedback() {}
export async function selectionFeedback() {}

// @tauri-apps/plugin-notification
export async function isPermissionGranted() { return true; }
export async function requestPermission() { return 'granted'; }
export async function sendNotification() {}
