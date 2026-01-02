/**
 * Mock @tauri-apps/api for web-only mode
 */

import type { ConnectionProfile } from '@/types/ssh';
import type { AppSettings } from '@/types/settings';
import type { P2PNodeInfo, P2PPeer } from '@/types/p2p';
import type { FileEntry } from '@/types/files';
import { defaultSettings } from '@/types/settings';

// Mock data
const mockProfiles: ConnectionProfile[] = [
  {
    id: 'mock-1',
    name: 'Production Server',
    host: '192.168.1.100',
    port: 22,
    username: 'admin',
    authType: 'key',
    keyPath: '~/.ssh/id_rsa',
    tags: ['favorite', 'production'],
    folder: 'Work',
    color: '#ef4444',
    autoReconnect: true,
    lastConnected: new Date().toISOString(),
    useCount: 42,
  },
  {
    id: 'mock-2',
    name: 'Dev Server',
    host: 'dev.example.com',
    port: 22,
    username: 'developer',
    authType: 'password',
    tags: ['development'],
    folder: 'Work',
    color: '#3b82f6',
    autoReconnect: false,
    lastConnected: new Date(Date.now() - 86400000).toISOString(),
    useCount: 15,
  },
  {
    id: 'mock-3',
    name: 'Raspberry Pi',
    host: '192.168.1.50',
    port: 22,
    username: 'pi',
    authType: 'password',
    tags: [],
    folder: 'Home',
    color: '#22c55e',
    autoReconnect: true,
    useCount: 8,
  },
];

const mockPeers: P2PPeer[] = [
  {
    id: 'peer-1',
    peerId: 'abc123',
    name: 'MacBook Pro',
    deviceType: 'desktop',
    connectionType: 'direct',
    latencyMs: 12,
    connectedAt: new Date().toISOString(),
  },
];

const mockFiles: FileEntry[] = [
  { name: 'Documents', path: '/home/user/Documents', isDirectory: true, isDir: true, size: 4096, modified: Date.now() },
  { name: 'Downloads', path: '/home/user/Downloads', isDirectory: true, isDir: true, size: 4096, modified: Date.now() },
  { name: '.bashrc', path: '/home/user/.bashrc', isDirectory: false, isDir: false, size: 3771, modified: Date.now() },
];

let settings: AppSettings = { ...defaultSettings };
let sessionCounter = 0;

const handlers: Record<string, (args?: unknown) => unknown> = {
  profile_list: () => mockProfiles,
  profile_create: () => `mock-${Date.now()}`,
  profile_update: () => null,
  profile_delete: () => null,
  ssh_connect: () => ({ sessionId: `session-${++sessionCounter}` }),
  ssh_disconnect: () => null,
  ssh_execute: () => ({ stdout: 'mock output\n', stderr: '', exitCode: 0 }),
  ssh_keep_alive: () => null,
  ssh_check_connection: () => true,
  terminal_start: () => null,
  terminal_input: () => null,
  terminal_resize: () => null,
  file_list: () => mockFiles,
  file_upload: () => null,
  file_download: () => null,
  file_delete: () => null,
  file_mkdir: () => null,
  settings_load: () => settings,
  settings_save: (args: unknown) => { settings = (args as { settings: AppSettings }).settings; },
  p2p_get_node_info: (): P2PNodeInfo => ({
    nodeId: 'mock-node-12345',
    relayUrl: 'wss://relay.example.com',
    directAddresses: ['192.168.1.10:9000'],
    isOnline: true,
  }),
  p2p_connect: () => mockPeers[0],
  p2p_disconnect: () => null,
  p2p_list_peers: () => mockPeers,
  p2p_generate_qr: () => 'russh://connect?node=mock-node-12345',
  p2p_send_message: () => null,
  p2p_send_typing: () => null,
  biometric_check: () => ({ available: false, type: 'none' }),
  biometric_authenticate: () => ({ success: true }),
  secure_store: () => null,
  secure_retrieve: () => null,
  secure_delete: () => null,
  haptic_feedback: () => null,
};

export async function invoke<T>(cmd: string, args?: unknown): Promise<T> {
  console.log(`[Mock] ${cmd}`, args);
  await new Promise(r => setTimeout(r, 50));
  const handler = handlers[cmd];
  return (handler ? handler(args) : null) as T;
}

// Event system mock
type EventCallback = (event: { payload: unknown }) => void;
const listeners = new Map<string, Set<EventCallback>>();

export async function listen(event: string, callback: EventCallback) {
  if (!listeners.has(event)) listeners.set(event, new Set());
  listeners.get(event)!.add(callback);
  return () => listeners.get(event)?.delete(callback);
}

export async function emit(event: string, payload?: unknown) {
  listeners.get(event)?.forEach(cb => cb({ payload }));
}

export const event = { listen, emit };
export const core = { invoke };

// Window mock
export async function type() { return 'webview'; }
export const window = { type };
