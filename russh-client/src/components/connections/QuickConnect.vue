<script setup lang="ts">
import { ref, computed } from 'vue';
import { Zap, Eye, EyeOff } from 'lucide-vue-next';
import { useSSH } from '@/composables/useSSH';
import { useNotification } from '@/composables/useNotification';

const emit = defineEmits<{
  (e: 'connected', sessionId: string): void;
  (e: 'cancel'): void;
}>();

const { connect, isConnecting } = useSSH();
const { error: showError } = useNotification();

const connectionString = ref('');
const password = ref('');
const showPassword = ref(false);

// Parse connection string like user@host:port or user@host
const parsed = computed(() => {
  const str = connectionString.value.trim();
  const match = str.match(/^([^@]+)@([^:]+)(?::(\d+))?$/);
  if (match) {
    return {
      username: match[1],
      host: match[2],
      port: parseInt(match[3] || '22', 10),
      valid: true
    };
  }
  return { username: '', host: '', port: 22, valid: false };
});

async function handleConnect() {
  if (!parsed.value.valid) {
    showError('Invalid Format', 'Use format: user@host or user@host:port');
    return;
  }
  
  try {
    const sessionId = await connect({
      host: parsed.value.host,
      port: parsed.value.port,
      username: parsed.value.username,
      authMethod: 'password',
      password: password.value
    });
    
    if (sessionId) {
      emit('connected', sessionId);
    }
  } catch (err) {
    showError('Connection Failed', String(err));
  }
}
</script>

<template>
  <div class="quick-connect p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
    <div class="flex items-center gap-2 mb-4">
      <Zap class="w-5 h-5 text-yellow-500" />
      <h3 class="font-medium text-gray-900 dark:text-white">Quick Connect</h3>
    </div>
    
    <form @submit.prevent="handleConnect" class="space-y-3">
      <div>
        <input
          v-model="connectionString"
          type="text"
          placeholder="user@host:port"
          class="w-full px-3 py-2 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          :class="{ 'border-green-500': parsed.valid && connectionString }"
        />
        <p v-if="connectionString && !parsed.valid" class="mt-1 text-xs text-red-500">
          Invalid format. Use: user@host or user@host:port
        </p>
      </div>
      
      <div class="relative">
        <input
          v-model="password"
          :type="showPassword ? 'text' : 'password'"
          placeholder="Password"
          class="w-full px-3 py-2 pr-10 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
        <button
          type="button"
          @click="showPassword = !showPassword"
          class="absolute right-2 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-gray-600"
        >
          <Eye v-if="!showPassword" class="w-4 h-4" />
          <EyeOff v-else class="w-4 h-4" />
        </button>
      </div>
      
      <div class="flex gap-2">
        <button
          type="submit"
          :disabled="!parsed.valid || isConnecting"
          class="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-lg font-medium transition-colors"
        >
          {{ isConnecting ? 'Connecting...' : 'Connect' }}
        </button>
        <button
          type="button"
          @click="emit('cancel')"
          class="px-4 py-2 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
        >
          Cancel
        </button>
      </div>
    </form>
  </div>
</template>
