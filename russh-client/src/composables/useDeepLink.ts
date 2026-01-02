/**
 * Deep linking composable - handles russh:// URL scheme
 */

import { ref, onMounted, onUnmounted } from 'vue';
import { useRouter } from 'vue-router';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { usePlatform } from './usePlatform';
import { useConnectionStore } from '@/stores/connections';

export interface DeepLinkParams {
  action: 'connect' | 'p2p' | 'profile';
  host?: string;
  port?: number;
  username?: string;
  profileId?: string;
  nodeId?: string;
}

export function useDeepLink() {
  const router = useRouter();
  const { isTauri } = usePlatform();
  const connectionStore = useConnectionStore();
  
  const pendingLink = ref<DeepLinkParams | null>(null);
  let unlisten: UnlistenFn | null = null;

  // Parse russh:// URL
  function parseDeepLink(url: string): DeepLinkParams | null {
    try {
      // Format: russh://action/params
      // Examples:
      //   russh://connect?host=example.com&port=22&username=user
      //   russh://p2p?nodeId=abc123
      //   russh://profile/profile-id
      
      const parsed = new URL(url);
      const action = parsed.hostname as DeepLinkParams['action'];
      const params = new URLSearchParams(parsed.search);
      
      switch (action) {
        case 'connect':
          return {
            action: 'connect',
            host: params.get('host') || undefined,
            port: params.get('port') ? parseInt(params.get('port')!) : undefined,
            username: params.get('username') || undefined,
          };
        
        case 'p2p':
          return {
            action: 'p2p',
            nodeId: params.get('nodeId') || undefined,
          };
        
        case 'profile':
          return {
            action: 'profile',
            profileId: parsed.pathname.slice(1) || undefined,
          };
        
        default:
          return null;
      }
    } catch {
      return null;
    }
  }

  // Handle deep link
  async function handleDeepLink(params: DeepLinkParams) {
    switch (params.action) {
      case 'connect':
        if (params.host) {
          // Navigate to quick connect with pre-filled data
          router.push({
            path: '/connections/new',
            query: {
              host: params.host,
              port: params.port?.toString(),
              username: params.username,
            },
          });
        }
        break;
      
      case 'p2p':
        if (params.nodeId) {
          // Navigate to P2P view and initiate connection
          router.push({
            path: '/p2p',
            query: { connect: params.nodeId },
          });
        }
        break;
      
      case 'profile':
        if (params.profileId) {
          // Find profile and connect
          const profile = connectionStore.profiles.find(p => p.id === params.profileId);
          if (profile) {
            await connectionStore.connect(params.profileId);
          }
        }
        break;
    }
  }

  // Generate deep link URL
  function generateConnectLink(host: string, port: number = 22, username?: string): string {
    const params = new URLSearchParams({ host, port: port.toString() });
    if (username) params.set('username', username);
    return `russh://connect?${params.toString()}`;
  }

  function generateP2PLink(nodeId: string): string {
    return `russh://p2p?nodeId=${encodeURIComponent(nodeId)}`;
  }

  function generateProfileLink(profileId: string): string {
    return `russh://profile/${encodeURIComponent(profileId)}`;
  }

  // Share link using system share sheet
  async function shareLink(url: string, title: string = 'RUSSH Connection') {
    if (navigator.share) {
      try {
        await navigator.share({ title, url });
      } catch {
        // User cancelled or share failed
      }
    } else {
      // Fallback to clipboard
      await navigator.clipboard.writeText(url);
    }
  }

  onMounted(async () => {
    if (!isTauri.value) return;

    // Listen for deep link events from Tauri
    unlisten = await listen<string>('deep-link', (event) => {
      const params = parseDeepLink(event.payload);
      if (params) {
        handleDeepLink(params);
      }
    });

    // Check for initial deep link (app opened via URL)
    try {
      const { getCurrent } = await import('@tauri-apps/api/window');
      const window = getCurrent();
      // Check if there's a pending URL from app launch
      // This would be set by the native code
    } catch {
      // Not available
    }
  });

  onUnmounted(() => {
    unlisten?.();
  });

  return {
    pendingLink,
    parseDeepLink,
    handleDeepLink,
    generateConnectLink,
    generateP2PLink,
    generateProfileLink,
    shareLink,
  };
}
