import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useConnectionStore } from '@/stores/connections';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core');

describe('Connection Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  describe('loadProfiles', () => {
    it('should load profiles from backend', async () => {
      const mockProfiles = [
        {
          id: '1',
          name: 'Test Server',
          host: 'example.com',
          port: 22,
          username: 'user',
          authType: 'password',
          tags: [],
          useCount: 0,
        },
      ];

      vi.mocked(invoke).mockResolvedValueOnce(mockProfiles);

      const store = useConnectionStore();
      await store.loadProfiles();

      expect(invoke).toHaveBeenCalledWith('profile_list');
      expect(store.profiles).toEqual(mockProfiles);
      expect(store.isLoading).toBe(false);
    });

    it('should handle load error', async () => {
      vi.mocked(invoke).mockRejectedValueOnce(new Error('Failed to load'));

      const store = useConnectionStore();
      await store.loadProfiles();

      expect(store.error).toBe('Error: Failed to load');
      expect(store.profiles).toEqual([]);
    });
  });

  describe('createProfile', () => {
    it('should create a new profile', async () => {
      vi.mocked(invoke)
        .mockResolvedValueOnce('new-id')
        .mockResolvedValueOnce([]);

      const store = useConnectionStore();
      const profile = {
        name: 'New Server',
        host: 'new.example.com',
        port: 22,
        username: 'admin',
        authType: 'password' as const,
        tags: [],
      };

      const id = await store.createProfile(profile);

      expect(invoke).toHaveBeenCalledWith('profile_create', { profile });
      expect(id).toBe('new-id');
    });
  });

  describe('connect', () => {
    it('should connect to a server', async () => {
      const mockProfile = {
        id: '1',
        name: 'Test Server',
        host: 'example.com',
        port: 22,
        username: 'user',
        authType: 'password' as const,
        tags: [],
        useCount: 0,
      };

      vi.mocked(invoke)
        .mockResolvedValueOnce([mockProfile]) // loadProfiles
        .mockResolvedValueOnce({ sessionId: 'session-123' }) // ssh_connect
        .mockResolvedValueOnce(undefined) // profile_update
        .mockResolvedValueOnce([{ ...mockProfile, useCount: 1 }]); // loadProfiles

      const store = useConnectionStore();
      await store.loadProfiles();

      const sessionId = await store.connect('1', 'password123');

      expect(sessionId).toBe('session-123');
      expect(store.activeConnections.get('1')?.status).toBe('connected');
    });

    it('should handle connection error', async () => {
      const mockProfile = {
        id: '1',
        name: 'Test Server',
        host: 'example.com',
        port: 22,
        username: 'user',
        authType: 'password' as const,
        tags: [],
        useCount: 0,
      };

      vi.mocked(invoke)
        .mockResolvedValueOnce([mockProfile])
        .mockRejectedValueOnce(new Error('Connection refused'));

      const store = useConnectionStore();
      await store.loadProfiles();

      await expect(store.connect('1', 'password')).rejects.toThrow('Connection refused');
      expect(store.activeConnections.get('1')?.status).toBe('error');
    });
  });

  describe('computed properties', () => {
    it('should sort profiles by folder and name', async () => {
      const mockProfiles = [
        { id: '1', name: 'Zebra', folder: 'B', host: 'a.com', port: 22, username: 'u', authType: 'password', tags: [], useCount: 0 },
        { id: '2', name: 'Alpha', folder: 'A', host: 'b.com', port: 22, username: 'u', authType: 'password', tags: [], useCount: 0 },
        { id: '3', name: 'Beta', folder: 'A', host: 'c.com', port: 22, username: 'u', authType: 'password', tags: [], useCount: 0 },
      ];

      vi.mocked(invoke).mockResolvedValueOnce(mockProfiles);

      const store = useConnectionStore();
      await store.loadProfiles();

      const sorted = store.sortedProfiles;
      expect(sorted[0].name).toBe('Alpha');
      expect(sorted[1].name).toBe('Beta');
      expect(sorted[2].name).toBe('Zebra');
    });

    it('should return unique folders', async () => {
      const mockProfiles = [
        { id: '1', name: 'A', folder: 'Production', host: 'a.com', port: 22, username: 'u', authType: 'password', tags: [], useCount: 0 },
        { id: '2', name: 'B', folder: 'Development', host: 'b.com', port: 22, username: 'u', authType: 'password', tags: [], useCount: 0 },
        { id: '3', name: 'C', folder: 'Production', host: 'c.com', port: 22, username: 'u', authType: 'password', tags: [], useCount: 0 },
      ];

      vi.mocked(invoke).mockResolvedValueOnce(mockProfiles);

      const store = useConnectionStore();
      await store.loadProfiles();

      expect(store.folders).toEqual(['Development', 'Production']);
    });
  });
});
