/**
 * Biometric authentication composable
 */

import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { usePlatform } from './usePlatform';

export type BiometricType = 'face' | 'fingerprint' | 'none';

export function useBiometric() {
  const { isMobile, isTauri, hapticFeedback } = usePlatform();
  
  const isAvailable = ref(false);
  const biometricType = ref<BiometricType>('none');
  const isAuthenticating = ref(false);
  const error = ref<string | null>(null);

  // Check if biometric authentication is available
  async function checkAvailability(): Promise<boolean> {
    if (!isMobile.value || !isTauri.value) {
      isAvailable.value = false;
      return false;
    }

    try {
      const result = await invoke<{ available: boolean; type: string }>('biometric_check');
      isAvailable.value = result.available;
      biometricType.value = result.type === 'face' ? 'face' : result.type === 'fingerprint' ? 'fingerprint' : 'none';
      return result.available;
    } catch {
      isAvailable.value = false;
      biometricType.value = 'none';
      return false;
    }
  }

  // Authenticate with biometrics
  async function authenticate(reason: string = 'Authenticate to continue'): Promise<boolean> {
    if (!isAvailable.value || isAuthenticating.value) {
      return false;
    }

    isAuthenticating.value = true;
    error.value = null;
    hapticFeedback('light');

    try {
      const result = await invoke<{ success: boolean; error?: string }>('biometric_authenticate', {
        reason
      });

      if (result.success) {
        hapticFeedback('success');
        return true;
      } else {
        hapticFeedback('error');
        error.value = result.error || 'Authentication failed';
        return false;
      }
    } catch (e) {
      hapticFeedback('error');
      error.value = String(e);
      return false;
    } finally {
      isAuthenticating.value = false;
    }
  }

  // Store credentials securely with biometric protection
  async function storeCredential(key: string, value: string): Promise<boolean> {
    if (!isTauri.value) return false;

    try {
      await invoke('secure_store', { key, value });
      return true;
    } catch {
      return false;
    }
  }

  // Retrieve credentials with biometric authentication
  async function getCredential(key: string, reason: string = 'Access credentials'): Promise<string | null> {
    if (!isTauri.value) return null;

    // Require biometric auth if available
    if (isAvailable.value) {
      const authenticated = await authenticate(reason);
      if (!authenticated) return null;
    }

    try {
      return await invoke<string>('secure_retrieve', { key });
    } catch {
      return null;
    }
  }

  // Delete stored credential
  async function deleteCredential(key: string): Promise<boolean> {
    if (!isTauri.value) return false;

    try {
      await invoke('secure_delete', { key });
      return true;
    } catch {
      return false;
    }
  }

  return {
    isAvailable,
    biometricType,
    isAuthenticating,
    error,
    checkAvailability,
    authenticate,
    storeCredential,
    getCredential,
    deleteCredential,
  };
}
