/**
 * P2P-related type definitions
 */

export interface P2PNodeInfo {
  nodeId: string;
  relayUrl?: string;
  directAddresses: string[];
  isOnline: boolean;
}

export interface P2PPeer {
  id: string;
  peerId?: string;
  name?: string;
  deviceType?: 'desktop' | 'mobile' | 'tablet';
  connectionType: P2PConnectionType;
  latency?: number;
  latencyMs?: number;
  connectedAt: string;
}

export type P2PConnectionType = 'direct' | 'relayed' | 'hole_punched';

export interface P2PConnectionQuality {
  latencyMs: number;
  packetLoss: number;
  bandwidth: number;
  stability: 'excellent' | 'good' | 'fair' | 'poor';
}
