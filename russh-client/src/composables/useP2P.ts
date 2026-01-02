/**
 * P2P composable - handles P2P connection and status
 */

import { ref, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useNotificationStore } from '@/stores/notifications';
import type { P2PNodeInfo, P2PPeer } from '@/types/p2p';
import { parseBackendError } from '@/types/errors';

export function useP2P() {
  const notificationStore = useNotificationStore();
  
  const nodeInfo = ref<P2PNodeInfo | null>(null);
  const peers = ref<P2PPeer[]>([]);
  const isOnline = ref(false);
  const qrCodeData = ref<string | null>(null);
  const isLoading = ref(false);
  
  let unlistenStatus: UnlistenFn | null = null;

  async function initialize() {
    isLoading.value = true;
    try {
      nodeInfo.value = await invoke<P2PNodeInfo>('p2p_get_node_info');
      isOnline.value = nodeInfo.value.isOnline;
      
      unlistenStatus = await listen<{ isOnline: boolean }>('p2p-status', (event) => {
        isOnline.value = event.payload.isOnline;
      });
    } catch (e) {
      console.error('Failed to initialize P2P:', e);
    } finally {
      isLoading.value = false;
    }
  }

  async function connectToPeer(peerId: string): Promise<P2PPeer> {
    try {
      const peerInfo = await invoke<P2PPeer>('p2p_connect', { peerId });
      peers.value.push(peerInfo);
      notificationStore.success('P2P Connected', `Connected to peer ${peerId.slice(0, 8)}...`);
      return peerInfo;
    } catch (e) {
      const appError = parseBackendError(e);
      notificationStore.error('P2P Connection Failed', appError.message);
      throw e;
    }
  }

  async function disconnectPeer(peerId: string) {
    try {
      await invoke('p2p_disconnect', { peerId });
      peers.value = peers.value.filter(p => p.peerId !== peerId);
      notificationStore.info('P2P Disconnected', `Disconnected from peer ${peerId.slice(0, 8)}...`);
    } catch (e) {
      const appError = parseBackendError(e);
      notificationStore.error('P2P Disconnect Failed', appError.message);
      throw e;
    }
  }

  async function refreshPeers() {
    try {
      peers.value = await invoke<P2PPeer[]>('p2p_list_peers');
    } catch (e) {
      console.error('Failed to refresh peers:', e);
    }
  }

  async function generateQRCode(): Promise<string> {
    try {
      qrCodeData.value = await invoke<string>('p2p_generate_qr');
      return qrCodeData.value;
    } catch (e) {
      const appError = parseBackendError(e);
      notificationStore.error('QR Generation Failed', appError.message);
      throw e;
    }
  }

  async function copyNodeId() {
    if (nodeInfo.value) {
      await navigator.clipboard.writeText(nodeInfo.value.nodeId);
      notificationStore.success('Copied', 'Node ID copied to clipboard');
    }
  }

  function getConnectionQuality(peer: P2PPeer): 'excellent' | 'good' | 'fair' | 'poor' {
    const latency = peer.latencyMs ?? Infinity;
    if (latency < 50) return 'excellent';
    if (latency < 100) return 'good';
    if (latency < 200) return 'fair';
    return 'poor';
  }

  function dispose() {
    unlistenStatus?.();
  }

  onUnmounted(dispose);

  return {
    // State
    nodeInfo,
    peers,
    isOnline,
    qrCodeData,
    isLoading,
    // Actions
    initialize,
    connectToPeer,
    disconnectPeer,
    refreshPeers,
    generateQRCode,
    copyNodeId,
    getConnectionQuality,
  };
}
