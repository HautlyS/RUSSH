/**
 * File transfer composable - handles file listing, upload, download
 */

import { ref, computed, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useNotificationStore } from '@/stores/notifications';
import type { TransferItem, FileEntry, TransferProgress } from '@/types/files';
import { parseBackendError } from '@/types/errors';

export function useFileTransfer() {
  const notificationStore = useNotificationStore();
  
  const transfers = ref<TransferItem[]>([]);
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  
  let unlistenProgress: UnlistenFn | null = null;

  const activeTransfers = computed(() => 
    transfers.value.filter(t => t.status === 'transferring' || t.status === 'pending')
  );

  const completedTransfers = computed(() =>
    transfers.value.filter(t => t.status === 'completed')
  );

  async function initialize() {
    unlistenProgress = await listen<TransferProgress>('transfer-progress', (event) => {
      const progress = event.payload;
      const index = transfers.value.findIndex(t => t.id === progress.transferId);
      
      if (index !== -1) {
        const total = progress.totalBytes || 1;
        transfers.value[index] = {
          ...transfers.value[index],
          transferred: progress.bytesTransferred,
          size: progress.totalBytes,
          progress: Math.round((progress.bytesTransferred / total) * 100),
          speed: progress.speedBps,
          status: progress.bytesTransferred >= progress.totalBytes ? 'completed' : 'transferring',
        };

        if (progress.bytesTransferred >= progress.totalBytes) {
          notificationStore.success('Transfer Complete', `${progress.filename} transferred successfully`);
        }
      }
    });
  }

  async function listFiles(sessionId: string, path: string): Promise<FileEntry[]> {
    isLoading.value = true;
    error.value = null;
    
    try {
      const files = await invoke<FileEntry[]>('file_list', { sessionId, path });
      return files.map(f => ({ ...f, isDirectory: f.isDir ?? f.isDirectory }));
    } catch (e) {
      const appError = parseBackendError(e);
      error.value = appError.message;
      throw e;
    } finally {
      isLoading.value = false;
    }
  }

  async function uploadFile(sessionId: string, remotePath: string, file: File): Promise<string> {
    const transferId = crypto.randomUUID();
    
    transfers.value.push({
      id: transferId,
      direction: 'upload',
      name: file.name,
      remotePath: `${remotePath}/${file.name}`,
      status: 'pending',
      transferred: 0,
      size: file.size,
      progress: 0,
    });
    
    try {
      // Read file as array buffer and send to backend
      const buffer = await file.arrayBuffer();
      await invoke('file_upload', {
        sessionId,
        remotePath: `${remotePath}/${file.name}`,
        data: Array.from(new Uint8Array(buffer)),
      });
      
      const idx = transfers.value.findIndex(t => t.id === transferId);
      if (idx !== -1) {
        transfers.value[idx].status = 'completed';
        transfers.value[idx].progress = 100;
        transfers.value[idx].transferred = file.size;
      }
      
      return transferId;
    } catch (e) {
      const idx = transfers.value.findIndex(t => t.id === transferId);
      if (idx !== -1) {
        transfers.value[idx].status = 'failed';
        transfers.value[idx].error = String(e);
      }
      throw e;
    }
  }

  async function downloadFile(sessionId: string, remotePath: string, filename: string): Promise<void> {
    const transferId = crypto.randomUUID();
    
    transfers.value.push({
      id: transferId,
      direction: 'download',
      name: filename,
      remotePath,
      status: 'pending',
      transferred: 0,
      size: 0,
      progress: 0,
    });
    
    try {
      await invoke('file_download', { sessionId, remotePath, filename });
      
      const idx = transfers.value.findIndex(t => t.id === transferId);
      if (idx !== -1) {
        transfers.value[idx].status = 'completed';
        transfers.value[idx].progress = 100;
      }
    } catch (e) {
      const idx = transfers.value.findIndex(t => t.id === transferId);
      if (idx !== -1) {
        transfers.value[idx].status = 'failed';
        transfers.value[idx].error = String(e);
      }
      throw e;
    }
  }

  async function deleteFile(sessionId: string, path: string): Promise<void> {
    try {
      await invoke('file_delete', { sessionId, path });
      notificationStore.success('Deleted', `${path} deleted successfully`);
    } catch (e) {
      const appError = parseBackendError(e);
      notificationStore.error('Delete Failed', appError.message);
      throw e;
    }
  }

  async function createDirectory(sessionId: string, path: string): Promise<void> {
    try {
      await invoke('file_mkdir', { sessionId, path });
      notificationStore.success('Created', `Directory created successfully`);
    } catch (e) {
      const appError = parseBackendError(e);
      notificationStore.error('Create Failed', appError.message);
      throw e;
    }
  }

  function cancelTransfer(id: string) {
    const idx = transfers.value.findIndex(t => t.id === id);
    if (idx !== -1) {
      transfers.value[idx].status = 'cancelled';
    }
  }

  function clearCompleted() {
    transfers.value = transfers.value.filter(t => 
      t.status !== 'completed' && t.status !== 'cancelled'
    );
  }

  function dispose() {
    unlistenProgress?.();
  }

  onUnmounted(dispose);

  return {
    transfers,
    isLoading,
    error,
    activeTransfers,
    completedTransfers,
    initialize,
    listFiles,
    uploadFile,
    downloadFile,
    deleteFile,
    createDirectory,
    cancelTransfer,
    clearCompleted,
    dispose,
  };
}
