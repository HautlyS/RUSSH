/**
 * File-related type definitions
 */

export interface FileEntry {
  name: string;
  path: string;
  isDirectory: boolean;
  isDir?: boolean; // Alias for compatibility
  size?: number;
  permissions?: string;
  modified?: number; // Unix timestamp
  owner?: string;
}

export interface TransferItem {
  id: string;
  direction: 'upload' | 'download';
  name: string;
  localPath?: string;
  remotePath?: string;
  status: TransferStatus;
  transferred: number;
  size: number;
  progress: number;
  speed?: number;
  error?: string;
}

export type TransferStatus = 
  | 'pending' 
  | 'transferring'
  | 'paused'
  | 'completed' 
  | 'failed' 
  | 'cancelled';

export interface TransferProgress {
  transferId: string;
  filename: string;
  bytesTransferred: number;
  totalBytes: number;
  speedBps: number;
  etaSeconds: number;
}

export interface FileOperation {
  type: 'rename' | 'delete' | 'mkdir' | 'chmod';
  path: string;
  newPath?: string;
  permissions?: string;
}
