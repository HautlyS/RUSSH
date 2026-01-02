/**
 * Error-related type definitions
 */

export interface AppError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
  recoverable: boolean;
}

export const ErrorCodes = {
  // Connection errors
  CONNECTION_TIMEOUT: 'CONNECTION_TIMEOUT',
  CONNECTION_REFUSED: 'CONNECTION_REFUSED',
  AUTH_FAILED: 'AUTH_FAILED',
  HOST_KEY_MISMATCH: 'HOST_KEY_MISMATCH',
  
  // Session errors
  SESSION_NOT_FOUND: 'SESSION_NOT_FOUND',
  SESSION_DISCONNECTED: 'SESSION_DISCONNECTED',
  
  // File errors
  FILE_NOT_FOUND: 'FILE_NOT_FOUND',
  PERMISSION_DENIED: 'PERMISSION_DENIED',
  TRANSFER_FAILED: 'TRANSFER_FAILED',
  
  // P2P errors
  PEER_NOT_FOUND: 'PEER_NOT_FOUND',
  P2P_CONNECTION_FAILED: 'P2P_CONNECTION_FAILED',
  
  // General errors
  UNKNOWN_ERROR: 'UNKNOWN_ERROR',
  INVALID_INPUT: 'INVALID_INPUT',
} as const;

export type ErrorCode = typeof ErrorCodes[keyof typeof ErrorCodes];

export function parseBackendError(error: unknown): AppError {
  if (typeof error === 'string') {
    return {
      code: ErrorCodes.UNKNOWN_ERROR,
      message: error,
      recoverable: false,
    };
  }
  
  if (error && typeof error === 'object' && 'code' in error) {
    return error as AppError;
  }
  
  return {
    code: ErrorCodes.UNKNOWN_ERROR,
    message: String(error),
    recoverable: false,
  };
}

export function isRecoverableError(error: AppError): boolean {
  const recoverableCodes = [
    ErrorCodes.CONNECTION_TIMEOUT,
    ErrorCodes.SESSION_DISCONNECTED,
    ErrorCodes.TRANSFER_FAILED,
  ];
  return recoverableCodes.includes(error.code as ErrorCode);
}
