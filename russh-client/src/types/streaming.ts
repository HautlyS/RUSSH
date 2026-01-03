/**
 * Streaming types for synchronized video playback
 */

export interface StreamRoom {
  roomId: string;
  name: string;
  hostId: string;
  source: StreamSource;
  playback: PlaybackState;
  peers: string[];
  shareLink: string;
}

export type StreamSource =
  | { type: 'url'; url: string }
  | { type: 'localFile'; path: string; size: number }
  | { type: 'p2pFile'; hostId: string; fileId: string; size: number };

export interface PlaybackState {
  playing: boolean;
  position: number;
  speed: number;
  syncTime: number;
}

export interface SyncEvent {
  type: 'play' | 'pause' | 'seek' | 'speed' | 'peerJoined' | 'peerLeft' | 'sourceChanged' | 'requestSync' | 'stateSync';
  position?: number;
  speed?: number;
  peerId?: string;
  source?: StreamSource;
  state?: PlaybackState;
}

export interface CreateStreamRequest {
  name: string;
  sourceType: 'url' | 'file';
  url?: string;
  filePath?: string;
}

export interface SyncEventRequest {
  roomId: string;
  eventType: 'play' | 'pause' | 'seek' | 'speed';
  position?: number;
  speed?: number;
}
