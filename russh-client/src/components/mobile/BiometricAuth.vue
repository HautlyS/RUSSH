<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { usePlatform } from '@/composables/usePlatform';
import { Fingerprint, Eye, Lock, AlertCircle } from 'lucide-vue-next';

const props = defineProps<{
  title?: string;
  subtitle?: string;
}>();

const emit = defineEmits<{
  success: [];
  cancel: [];
  fallback: [];
}>();

const { isMobile, hapticFeedback } = usePlatform();

const isAuthenticating = ref(false);
const error = ref<string | null>(null);
const biometricType = ref<'face' | 'fingerprint' | 'none'>('none');

// Check biometric availability
async function checkBiometricAvailability() {
  try {
    const result = await invoke<{ available: boolean; type: string }>('biometric_check');
    if (result.available) {
      biometricType.value = result.type === 'face' ? 'face' : 'fingerprint';
    }
  } catch {
    biometricType.value = 'none';
  }
}

// Authenticate with biometrics
async function authenticate() {
  if (isAuthenticating.value) return;
  
  isAuthenticating.value = true;
  error.value = null;
  hapticFeedback('light');
  
  try {
    const result = await invoke<{ success: boolean; error?: string }>('biometric_authenticate', {
      reason: props.title || 'Authenticate to continue'
    });
    
    if (result.success) {
      hapticFeedback('success');
      emit('success');
    } else {
      hapticFeedback('error');
      error.value = result.error || 'Authentication failed';
    }
  } catch (e) {
    hapticFeedback('error');
    error.value = String(e);
  } finally {
    isAuthenticating.value = false;
  }
}

function useFallback() {
  hapticFeedback('light');
  emit('fallback');
}

function cancel() {
  hapticFeedback('light');
  emit('cancel');
}

const biometricIcon = computed(() => {
  return biometricType.value === 'face' ? Eye : Fingerprint;
});

const biometricLabel = computed(() => {
  return biometricType.value === 'face' ? 'Face ID' : 'Touch ID';
});

onMounted(() => {
  checkBiometricAvailability();
});
</script>

<template>
  <div class="biometric-auth flex flex-col items-center justify-center min-h-[300px] p-6">
    <!-- Icon -->
    <div 
      class="w-20 h-20 rounded-full flex items-center justify-center mb-6 transition-colors"
      :class="[
        isAuthenticating ? 'bg-blue-500/20 animate-pulse' : 'bg-gray-100 dark:bg-gray-800',
        error ? 'bg-red-500/20' : ''
      ]"
    >
      <component 
        :is="error ? AlertCircle : biometricIcon" 
        class="w-10 h-10"
        :class="error ? 'text-red-500' : 'text-blue-500'"
      />
    </div>
    
    <!-- Title -->
    <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
      {{ title || 'Authentication Required' }}
    </h2>
    
    <!-- Subtitle -->
    <p class="text-sm text-gray-500 dark:text-gray-400 text-center mb-6">
      {{ subtitle || `Use ${biometricLabel} to authenticate` }}
    </p>
    
    <!-- Error Message -->
    <p v-if="error" class="text-sm text-red-500 mb-4 text-center">
      {{ error }}
    </p>
    
    <!-- Authenticate Button -->
    <button
      v-if="biometricType !== 'none'"
      @click="authenticate"
      :disabled="isAuthenticating"
      class="w-full max-w-xs flex items-center justify-center gap-2 px-6 py-3 bg-blue-600 text-white rounded-lg font-medium transition-colors disabled:opacity-50"
      :class="{ 'hover:bg-blue-500': !isAuthenticating }"
    >
      <component :is="biometricIcon" class="w-5 h-5" />
      {{ isAuthenticating ? 'Authenticating...' : `Use ${biometricLabel}` }}
    </button>
    
    <!-- Fallback Option -->
    <button
      @click="useFallback"
      class="mt-4 text-sm text-blue-600 dark:text-blue-400 hover:underline"
    >
      <Lock class="w-4 h-4 inline mr-1" />
      Use password instead
    </button>
    
    <!-- Cancel -->
    <button
      @click="cancel"
      class="mt-4 text-sm text-gray-500 dark:text-gray-400 hover:underline"
    >
      Cancel
    </button>
  </div>
</template>
