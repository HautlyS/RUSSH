/**
 * Mock Tauri backend for web-only development/testing
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
  {
    id: 'peer-2',
    peerId: 'def456',
    name: 'iPhone',
    deviceType: 'mobile',
    connectionType: 'relayed',
    latencyMs: 45,
    connectedAt: new Date(Date.now() - 3600000).toISOString(),
  },
];

const mockFiles: FileEntry[] = [
  { name: 'Documents', path: '/home/user/Documents', isDirectory: true, isDir: true, size: 4096, modified: Date.now() },
  { name: 'Downloads', path: '/home/user/Downloads', isDirectory: true, isDir: true, size: 4096, modified: Date.now() },
  { name: '.bashrc', path: '/home/user/.bashrc', isDirectory: false, isDir: false, size: 3771, modified: Date.now() - 86400000 },
  { name: 'notes.txt', path: '/home/user/notes.txt', isDirectory: false, isDir: false, size: 1024, modified: Date.now() - 3600000 },
];

let settings: AppSettings = { ...defaultSettings };
let sessionCounter = 0;

// Command handlers
const handlers: Record<string, (args?: unknown) => unknown> = {
  // Profiles
  profile_list: () => mockProfiles,
  profile_create: () => `mock-${Date.now()}`,
  profile_update: () => null,
  profile_delete: () => null,

  // SSH
  ssh_connect: () => {
    sessionCounter++;
    return { sessionId: `session-${sessionCounter}` };
  },
  ssh_disconnect: () => null,
  ssh_execute: () => ({ stdout: 'mock output\n', stderr: '', exitCode: 0 }),
  ssh_keep_alive: () => null,
  ssh_check_connection: () => true,

  // Terminal
  terminal_start: () => null,
  terminal_input: () => null,
  terminal_resize: () => null,

  // Files
  file_list: () => mockFiles,
  file_upload: () => null,
  file_download: () => null,
  file_delete: () => null,
  file_mkdir: () => null,

  // Settings
  settings_load: () => settings,
  settings_save: (args: unknown) => {
    settings = (args as { settings: AppSettings }).settings;
    return null;
  },

  // P2P
  p2p_get_node_info: (): P2PNodeInfo => ({
    nodeId: 'mock-node-id-12345',
    relayUrl: 'wss://relay.example.com',
    directAddresses: ['192.168.1.10:9000'],
    isOnline: true,
  }),
  p2p_connect: () => mockPeers[0],
  p2p_disconnect: () => null,
  p2p_list_peers: () => mockPeers,
  p2p_generate_qr: () => 'russh://connect?node=mock-node-id-12345',
  p2p_send_message: () => null,
  p2p_send_typing: () => null,

  // Biometric
  biometric_check: () => ({ available: false, type: 'none' }),
  biometric_authenticate: () => ({ success: true }),
  secure_store: () => null,
  secure_retrieve: () => null,
  secure_delete: () => null,

  // Platform
  haptic_feedback: () => null,
};

// Mock invoke function
export async function invoke<T>(cmd: string, args?: unknown): Promise<T> {
  console.log(`[Mock] invoke: ${cmd}`, args);
  
  const handler = handlers[cmd];
  if (!handler) {
    console.warn(`[Mock] Unknown command: ${cmd}`);
    return null as T;
  }

  // Simulate network delay
  await new Promise(r => setTimeout(r, 50 + Math.random() * 100));
  
  return handler(args) as T;
}

// Mock listen function for events
export async function listen<T>(
  _event: string,
  _handler: (event: { payload: T }) => void
): Promise<() => void> {
  // Return unlisten function
  return () => {};
}

// Install mocks globally
export function installMocks() {
  if (typeof window !== 'undefined') {
    (window as unknown as { 
      __TAURI_INTERNALS__: { invoke: typeof invoke };
      __TAURI_PLUGIN_EVENT__: { listen: typeof listen };
    }).__TAURI_INTERNALS__ = { invoke };
    
    // Mock the event module
    (window as unknown as { __TAURI_PLUGIN_EVENT__: { listen: typeof listen } }).__TAURI_PLUGIN_EVENT__ = { listen };
    
    console.log('[Mock] Tauri mocks installed');
  }
}
