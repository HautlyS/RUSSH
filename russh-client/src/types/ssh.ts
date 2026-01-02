/**
 * SSH-related type definitions
 */

export interface ConnectionProfile {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  authType: 'password' | 'key' | 'agent';
  keyPath?: string;
  tags: string[];
  folder?: string;
  color?: string;
  autoReconnect: boolean;
  lastConnected?: string;
  useCount: number;
}

export interface ConnectionState {
  sessionId: string;
  profileId: string;
  status: ConnectionStatus;
  connectedAt?: Date;
  error?: string;
  stats: ConnectionStats;
}

export type ConnectionStatus = 
  | 'connecting' 
  | 'connected' 
  | 'disconnected' 
  | 'reconnecting' 
  | 'error';

export interface ConnectionStats {
  uptime: number;
  bytesSent: number;
  bytesReceived: number;
  commandsExecuted: number;
}

export interface CommandResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}

export interface ConnectionRequest {
  host: string;
  port: number;
  username: string;
  authType?: string;
  authMethod?: string;
  password?: string;
  keyPath?: string;
  keyPassphrase?: string;
}

export interface ConnectionResponse {
  sessionId: string;
  connected: boolean;
  host: string;
  username: string;
}

export interface CommandRequest {
  sessionId: string;
  command: string;
  timeoutSecs?: number;
}
