<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { QrCode, Camera, Copy, Check, RefreshCw } from 'lucide-vue-next';
import { useP2P } from '@/composables/useP2P';

const { nodeInfo, generateQRCode, connectToPeer } = useP2P();

const qrCodeImage = ref<string>('');
const isGenerating = ref(false);
const copied = ref(false);
const showScanner = ref(false);
const peerIdInput = ref('');

async function generateQR() {
  isGenerating.value = true;
  try {
    qrCodeImage.value = await generateQRCode();
  } finally {
    isGenerating.value = false;
  }
}

async function copyNodeId() {
  if (nodeInfo.value?.nodeId) {
    await navigator.clipboard.writeText(nodeInfo.value.nodeId);
    copied.value = true;
    setTimeout(() => copied.value = false, 2000);
  }
}

async function handleConnect() {
  if (peerIdInput.value.trim()) {
    await connectToPeer(peerIdInput.value.trim());
    peerIdInput.value = '';
  }
}

onMounted(generateQR);
</script>

<template>
  <div class="qr-code-share p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
    <div class="flex items-center gap-2 mb-4">
      <QrCode class="w-5 h-5 text-blue-500" />
      <h3 class="font-medium text-gray-900 dark:text-white">Share Connection</h3>
    </div>
    
    <!-- QR Code Display -->
    <div class="flex justify-center mb-4">
      <div class="relative p-4 bg-white rounded-lg">
        <img 
          v-if="qrCodeImage" 
          :src="qrCodeImage" 
          alt="QR Code" 
          class="w-48 h-48"
        />
        <div v-else class="w-48 h-48 flex items-center justify-center bg-gray-100 rounded">
          <RefreshCw v-if="isGenerating" class="w-8 h-8 text-gray-400 animate-spin" />
          <QrCode v-else class="w-8 h-8 text-gray-400" />
        </div>
        
        <button
          @click="generateQR"
          class="absolute top-2 right-2 p-1 bg-white rounded shadow hover:bg-gray-100"
          title="Regenerate QR Code"
        >
          <RefreshCw class="w-4 h-4 text-gray-500" :class="{ 'animate-spin': isGenerating }" />
        </button>
      </div>
    </div>
    
    <!-- Node ID -->
    <div class="mb-4">
      <label class="block text-sm text-gray-500 dark:text-gray-400 mb-1">Your Node ID</label>
      <div class="flex items-center gap-2">
        <code class="flex-1 px-3 py-2 bg-gray-50 dark:bg-gray-900 rounded text-xs font-mono truncate">
          {{ nodeInfo?.nodeId || 'Loading...' }}
        </code>
        <button
          @click="copyNodeId"
          class="p-2 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          :title="copied ? 'Copied!' : 'Copy'"
        >
          <Check v-if="copied" class="w-4 h-4 text-green-500" />
          <Copy v-else class="w-4 h-4 text-gray-500" />
        </button>
      </div>
    </div>
    
    <!-- Connect to Peer -->
    <div>
      <label class="block text-sm text-gray-500 dark:text-gray-400 mb-1">Connect to Peer</label>
      <div class="flex gap-2">
        <input
          v-model="peerIdInput"
          type="text"
          placeholder="Enter peer Node ID"
          class="flex-1 px-3 py-2 bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg text-sm"
        />
        <button
          @click="handleConnect"
          :disabled="!peerIdInput.trim()"
          class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded-lg text-sm font-medium transition-colors"
        >
          Connect
        </button>
      </div>
    </div>
  </div>
</template>
