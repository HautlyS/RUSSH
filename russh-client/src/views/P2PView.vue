<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { MessageSquare } from 'lucide-vue-next';
import { useP2P } from '@/composables/useP2P';
import { useVisualEffects } from '@/composables/useVisualEffects';
import P2PStatus from '@/components/p2p/P2PStatus.vue';
import PeerList from '@/components/p2p/PeerList.vue';
import QRCodeShare from '@/components/p2p/QRCodeShare.vue';
import Lightning from '@/components/extra/Lightning.vue';
import ElectricBorder from '@/components/extra/ElectricBorder.vue';
import RotatingText from '@/components/extra/RotatingText.vue';
import type { P2PPeer } from '@/types/p2p';

const router = useRouter();

const { nodeInfo, peers, isOnline, initialize, disconnectPeer } = useP2P();
const { 
  isLightningEnabledFor, 
  isElectricBorderEnabled, 
  isRotatingTextEnabled,
  visualEffects 
} = useVisualEffects();

const isLightningEnabled = isLightningEnabledFor('p2p');

const selectedPeer = ref<P2PPeer | null>(null);

const p2pTips = [
  'Share your QR code to connect',
  'Direct connections are faster',
  'NAT traversal enabled',
  'Encrypted peer-to-peer',
];

onMounted(() => {
  initialize();
});

function handleSelectPeer(peer: P2PPeer) {
  selectedPeer.value = peer;
}

function handleDisconnect(peerId: string) {
  disconnectPeer(peerId);
  if (selectedPeer.value?.id === peerId) {
    selectedPeer.value = null;
  }
}
</script>

<template>
  <div class="p2p-view h-full flex flex-col relative">
    <!-- Lightning Background -->
    <div 
      v-if="isLightningEnabled" 
      class="absolute inset-0 z-0 opacity-20"
    >
      <Lightning
        :hue="280"
        :intensity="visualEffects.lightning.intensity"
        :speed="visualEffects.lightning.speed"
        :size="1"
      />
    </div>

    <header class="relative z-10 px-6 py-4 border-b border-gray-200 dark:border-gray-700">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold text-gray-900 dark:text-white">P2P Network</h1>
          <p class="text-sm text-gray-500 dark:text-gray-400 mt-1">
            Connect directly with other devices using peer-to-peer networking
          </p>
        </div>
        <button 
          @click="router.push('/p2p/terminal')"
          class="flex items-center gap-2 px-4 py-2 bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 text-white rounded-lg font-medium transition-all shadow-lg hover:shadow-xl"
        >
          <MessageSquare class="w-5 h-5" />
          Open Terminal
        </button>
      </div>
    </header>
    
    <div class="relative z-10 flex-1 overflow-auto p-6">
      <div class="max-w-4xl mx-auto grid gap-6 lg:grid-cols-2">
        <!-- Status & QR Code -->
        <div class="space-y-6">
          <P2PStatus 
            :is-online="isOnline" 
            :node-id="nodeInfo?.nodeId" 
            :peer-count="peers.length" 
          />
          
          <!-- QR Code with Electric Border -->
          <ElectricBorder
            v-if="isElectricBorderEnabled"
            color="#7df9ff"
            :speed="1.2"
            :chaos="0.5"
            :thickness="2"
            :style="{ borderRadius: '12px' }"
          >
            <QRCodeShare />
          </ElectricBorder>
          <QRCodeShare v-else />
        </div>
        
        <!-- Peer List -->
        <div class="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
          <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
            <h2 class="font-semibold text-gray-900 dark:text-white">Connected Peers</h2>
          </div>
          
          <PeerList 
            :peers="peers" 
            @select="handleSelectPeer" 
            @disconnect="handleDisconnect" 
          />
        </div>
      </div>

      <!-- P2P Tips -->
      <div v-if="isRotatingTextEnabled" class="mt-6 text-center">
        <p class="text-sm text-gray-500 dark:text-gray-400 flex items-center justify-center gap-2">
          <span>💡</span>
          <RotatingText
            :texts="p2pTips"
            :rotation-interval="visualEffects.rotatingText.rotationInterval"
            :stagger-duration="visualEffects.rotatingText.staggerDuration"
            main-class-name="text-gray-500 dark:text-gray-400"
            :transition="{ type: 'spring', damping: 30, stiffness: 400 }"
          />
        </p>
      </div>
    </div>
  </div>
</template>
