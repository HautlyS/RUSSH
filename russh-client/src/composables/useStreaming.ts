/**
 * Streaming composable - manages synchronized video streaming
 */

import { ref, computed, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { StreamRoom, CreateStreamRequest, SyncEvent, SyncEventRequest } from '@/types/streaming';

export function useStreaming() {
  const room = ref<StreamRoom | null>(null);
  const isHost = ref(false);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const syncOffset = ref(0); // Offset to correct for latency
  
  let unlistenEvents: UnlistenFn | null = null;
  let syncInterval: ReturnType<typeof setInterval> | null = null;

  const shareLink = computed(() => room.value?.shareLink || '');
  const isPlaying = computed(() => room.value?.playback.playing || false);
  const currentPosition = computed(() => room.value?.playback.position || 0);
  const playbackSpeed = computed(() => room.value?.playback.speed || 1);
  const peerCount = computed(() => room.value?.peers.length || 0);

  async function createRoom(request: CreateStreamRequest): Promise<StreamRoom> {
    isLoading.value = true;
    error.value = null;
    
    try {
      const result = await invoke<StreamRoom>('stream_create_room', { request });
      room.value = result;
      isHost.value = true;
      await setupEventListener(result.roomId);
      return result;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function joinRoom(roomId: string, hostId: string): Promise<StreamRoom> {
    isLoading.value = true;
    error.value = null;
    
    try {
      const result = await invoke<StreamRoom>('stream_join_room', { roomId, hostId });
      room.value = result;
      isHost.value = false;
      await setupEventListener(result.roomId);
      
      // Request sync from host
      await requestSync();
      
      return result;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function leaveRoom(): Promise<void> {
    if (!room.value) return;
    
    try {
      await invoke('stream_leave_room', { roomId: room.value.roomId });
    } finally {
      cleanup();
      room.value = null;
      isHost.value = false;
    }
  }

  async function play(): Promise<void> {
    if (!room.value) return;
    
    await invoke('stream_sync', {
      request: {
        roomId: room.value.roomId,
        eventType: 'play',
        position: room.value.playback.position,
      } as SyncEventRequest,
    });
  }

  async function pause(): Promise<void> {
    if (!room.value) return;
    
    await invoke('stream_sync', {
      request: {
        roomId: room.value.roomId,
        eventType: 'pause',
        position: room.value.playback.position,
      } as SyncEventRequest,
    });
  }

  async function seek(position: number): Promise<void> {
    if (!room.value) return;
    
    await invoke('stream_sync', {
      request: {
        roomId: room.value.roomId,
        eventType: 'seek',
        position,
      } as SyncEventRequest,
    });
  }

  async function setSpeed(speed: number): Promise<void> {
    if (!room.value) return;
    
    await invoke('stream_sync', {
      request: {
        roomId: room.value.roomId,
        eventType: 'speed',
        speed,
      } as SyncEventRequest,
    });
  }

  async function updatePosition(position: number): Promise<void> {
    if (!room.value) return;
    
    await invoke('stream_update_position', {
      roomId: room.value.roomId,
      position,
    });
  }

  async function getExpectedPosition(): Promise<number> {
    if (!room.value) return 0;
    
    return await invoke<number>('stream_get_expected_position', {
      roomId: room.value.roomId,
    });
  }

  async function requestSync(): Promise<void> {
    // This would send a sync request to the host
    // For now, just refresh room state
    if (!room.value) return;
    
    const result = await invoke<StreamRoom>('stream_get_room', {
      roomId: room.value.roomId,
    });
    room.value = result;
  }

  async function setupEventListener(roomId: string): Promise<void> {
    unlistenEvents = await listen<SyncEvent>(`stream-event-${roomId}`, (event) => {
      handleSyncEvent(event.payload);
    });
    
    // Start periodic sync check
    syncInterval = setInterval(async () => {
      if (room.value?.playback.playing) {
        const expected = await getExpectedPosition();
        syncOffset.value = expected - (room.value?.playback.position || 0);
      }
    }, 5000);
  }

  function handleSyncEvent(event: SyncEvent): void {
    if (!room.value) return;
    
    switch (event.type) {
      case 'play':
        room.value.playback.playing = true;
        if (event.position !== undefined) {
          room.value.playback.position = event.position;
        }
        break;
      case 'pause':
        room.value.playback.playing = false;
        if (event.position !== undefined) {
          room.value.playback.position = event.position;
        }
        break;
      case 'seek':
        if (event.position !== undefined) {
          room.value.playback.position = event.position;
        }
        break;
      case 'speed':
        if (event.speed !== undefined) {
          room.value.playback.speed = event.speed;
        }
        break;
      case 'peerJoined':
        if (event.peerId && !room.value.peers.includes(event.peerId)) {
          room.value.peers.push(event.peerId);
        }
        break;
      case 'peerLeft':
        if (event.peerId) {
          room.value.peers = room.value.peers.filter(p => p !== event.peerId);
        }
        break;
      case 'stateSync':
        if (event.state) {
          room.value.playback = event.state;
        }
        break;
    }
  }

  function cleanup(): void {
    unlistenEvents?.();
    unlistenEvents = null;
    
    if (syncInterval) {
      clearInterval(syncInterval);
      syncInterval = null;
    }
  }

  onUnmounted(() => {
    cleanup();
  });

  return {
    room,
    isHost,
    isLoading,
    error,
    shareLink,
    isPlaying,
    currentPosition,
    playbackSpeed,
    peerCount,
    syncOffset,
    createRoom,
    joinRoom,
    leaveRoom,
    play,
    pause,
    seek,
    setSpeed,
    updatePosition,
    getExpectedPosition,
    requestSync,
  };
}
