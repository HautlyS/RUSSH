/**
 * Connection store - manages SSH connection profiles and active connections
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ConnectionProfile, ConnectionState, ConnectionStats } from '@/types/ssh';

export const useConnectionStore = defineStore('connections', () => {
  // State
  const profiles = ref<ConnectionProfile[]>([]);
  const activeConnections = ref<Map<string, ConnectionState>>(new Map());
  const selectedProfileId = ref<string | null>(null);
  const isLoading = ref(false);
  const error = ref<string | null>(null);

  // Getters
  const sortedProfiles = computed(() => {
    return [...profiles.value].sort((a, b) => {
      // Sort by folder, then by name
      if (a.folder !== b.folder) {
        return (a.folder || '').localeCompare(b.folder || '');
      }
      return a.name.localeCompare(b.name);
    });
  });

  const connectedProfiles = computed(() => {
    return profiles.value.filter(p => {
      const conn = activeConnections.value.get(p.id);
      return conn && conn.status === 'connected';
    });
  });

  const folders = computed(() => {
    const folderSet = new Set<string>();
    profiles.value.forEach(p => {
      if (p.folder) folderSet.add(p.folder);
    });
    return Array.from(folderSet).sort();
  });

  const favoriteProfiles = computed(() => {
    return profiles.value.filter(p => p.tags.includes('favorite'));
  });

  const recentProfiles = computed(() => {
    return [...profiles.value]
      .filter(p => p.lastConnected)
      .sort((a, b) => {
        const dateA = new Date(a.lastConnected || 0);
        const dateB = new Date(b.lastConnected || 0);
        return dateB.getTime() - dateA.getTime();
      })
      .slice(0, 5);
  });

  // Actions
  async function loadProfiles() {
    isLoading.value = true;
    error.value = null;
    try {
      profiles.value = await invoke<ConnectionProfile[]>('profile_list');
    } catch (e) {
      error.value = String(e);
      console.error('Failed to load profiles:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function createProfile(profile: Omit<ConnectionProfile, 'id' | 'useCount'>) {
    const id = await invoke<string>('profile_create', { profile });
    await loadProfiles();
    return id;
  }

  async function updateProfile(profile: ConnectionProfile) {
    await invoke('profile_update', { profile });
    await loadProfiles();
  }

  async function deleteProfile(profileId: string) {
    await invoke('profile_delete', { profileId });
    profiles.value = profiles.value.filter(p => p.id !== profileId);
  }

  async function connect(profileId: string, password?: string) {
    const profile = profiles.value.find(p => p.id === profileId);
    if (!profile) throw new Error('Profile not found');

    // Set connecting state
    activeConnections.value.set(profileId, {
      sessionId: '',
      profileId,
      status: 'connecting',
      stats: createEmptyStats(),
    });

    try {
      const response = await invoke<{ sessionId: string }>('ssh_connect', {
        request: {
          host: profile.host,
          port: profile.port,
          username: profile.username,
          authType: profile.authType,
          password,
          keyPath: profile.keyPath,
        }
      });

      // Update to connected state
      activeConnections.value.set(profileId, {
        sessionId: response.sessionId,
        profileId,
        status: 'connected',
        connectedAt: new Date(),
        stats: createEmptyStats(),
      });

      // Update profile's last connected time
      const updatedProfile = { ...profile, lastConnected: new Date().toISOString(), useCount: profile.useCount + 1 };
      await updateProfile(updatedProfile);

      return response.sessionId;
    } catch (e) {
      // Set error state
      activeConnections.value.set(profileId, {
        sessionId: '',
        profileId,
        status: 'error',
        error: String(e),
        stats: createEmptyStats(),
      });
      throw e;
    }
  }

  async function disconnect(profileId: string) {
    const connection = activeConnections.value.get(profileId);
    if (!connection) return;

    await invoke('ssh_disconnect', { sessionId: connection.sessionId });
    activeConnections.value.delete(profileId);
  }

  function getConnection(profileId: string): ConnectionState | undefined {
    return activeConnections.value.get(profileId);
  }

  function getConnectionBySessionId(sessionId: string): ConnectionState | undefined {
    for (const conn of activeConnections.value.values()) {
      if (conn.sessionId === sessionId) {
        return conn;
      }
    }
    return undefined;
  }

  function selectProfile(profileId: string | null) {
    selectedProfileId.value = profileId;
  }

  function createEmptyStats(): ConnectionStats {
    return {
      uptime: 0,
      bytesSent: 0,
      bytesReceived: 0,
      commandsExecuted: 0,
    };
  }

  // Alias for component compatibility
  const connectionStates = computed(() => activeConnections.value);

  async function moveToFolder(profileId: string, folder: string) {
    const profile = profiles.value.find(p => p.id === profileId);
    if (profile) {
      await updateProfile({ ...profile, folder });
    }
  }

  return {
    // State
    profiles,
    activeConnections,
    connectionStates,
    selectedProfileId,
    isLoading,
    error,
    // Getters
    sortedProfiles,
    connectedProfiles,
    folders,
    favoriteProfiles,
    recentProfiles,
    // Actions
    loadProfiles,
    createProfile,
    updateProfile,
    deleteProfile,
    connect,
    disconnect,
    getConnection,
    getConnectionBySessionId,
    selectProfile,
    moveToFolder,
  };
});
