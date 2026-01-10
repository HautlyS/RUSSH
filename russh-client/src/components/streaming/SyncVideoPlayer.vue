<script setup lang="ts">
/**
 * SyncVideoPlayer - Synchronized video player with P2P sync support
 */
import { ref, watch, onMounted, onUnmounted, computed } from 'vue';
import { Play, Pause, Volume2, VolumeX, Maximize, Users, Link } from 'lucide-vue-next';
import { useStreaming } from '@/composables/useStreaming';
import type { StreamSource } from '@/types/streaming';

defineProps<{
  source?: StreamSource;
  roomId?: string;
}>();

const emit = defineEmits<{
  (e: 'timeUpdate', position: number): void;
  (e: 'ended'): void;
}>();

const {
  room,
  isHost,
  isPlaying,
  currentPosition,
  playbackSpeed,
  peerCount,
  shareLink,
  syncOffset,
  play,
  pause,
  seek,
  setSpeed,
  updatePosition,
  getExpectedPosition,
} = useStreaming();

const videoRef = ref<HTMLVideoElement | null>(null);
const containerRef = ref<HTMLDivElement | null>(null);
const volume = ref(1);
const isMuted = ref(false);
const isFullscreen = ref(false);
const showControls = ref(true);
const duration = ref(0);
const buffered = ref(0);
const showShareDialog = ref(false);
const linkCopied = ref(false);

let controlsTimeout: ReturnType<typeof setTimeout> | null = null;
let syncCheckInterval: ReturnType<typeof setInterval> | null = null;

const videoSrc = computed(() => {
  if (!room.value?.source) return '';
  if (room.value.source.type === 'url') {
    return room.value.source.url;
  }
  return '';
});

const formattedTime = computed(() => {
  const pos = currentPosition.value;
  const dur = duration.value;
  return `${formatTime(pos)} / ${formatTime(dur)}`;
});

const progress = computed(() => {
  if (duration.value === 0) return 0;
  return (currentPosition.value / duration.value) * 100;
});

function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  
  if (h > 0) {
    return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }
  return `${m}:${s.toString().padStart(2, '0')}`;
}

async function togglePlay() {
  if (isPlaying.value) {
    await pause();
    videoRef.value?.pause();
  } else {
    await play();
    videoRef.value?.play();
  }
}

async function handleSeek(e: MouseEvent) {
  const target = e.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  const percent = (e.clientX - rect.left) / rect.width;
  const newPosition = percent * duration.value;
  
  await seek(newPosition);
  if (videoRef.value) {
    videoRef.value.currentTime = newPosition;
  }
}

function handleVolumeChange(e: Event) {
  const target = e.target as HTMLInputElement;
  volume.value = parseFloat(target.value);
  if (videoRef.value) {
    videoRef.value.volume = volume.value;
  }
  isMuted.value = volume.value === 0;
}

function toggleMute() {
  isMuted.value = !isMuted.value;
  if (videoRef.value) {
    videoRef.value.muted = isMuted.value;
  }
}

async function toggleFullscreen() {
  if (!containerRef.value) return;
  
  if (!document.fullscreenElement) {
    await containerRef.value.requestFullscreen();
    isFullscreen.value = true;
  } else {
    await document.exitFullscreen();
    isFullscreen.value = false;
  }
}

function handleMouseMove() {
  showControls.value = true;
  
  if (controlsTimeout) {
    clearTimeout(controlsTimeout);
  }
  
  controlsTimeout = setTimeout(() => {
    if (isPlaying.value) {
      showControls.value = false;
    }
  }, 3000);
}

function handleTimeUpdate() {
  if (!videoRef.value) return;
  
  const pos = videoRef.value.currentTime;
  updatePosition(pos);
  emit('timeUpdate', pos);
}

function handleLoadedMetadata() {
  if (!videoRef.value) return;
  duration.value = videoRef.value.duration;
}

function handleProgress() {
  if (!videoRef.value) return;
  
  const buf = videoRef.value.buffered;
  if (buf.length > 0) {
    buffered.value = (buf.end(buf.length - 1) / duration.value) * 100;
  }
}

async function copyShareLink() {
  await navigator.clipboard.writeText(shareLink.value);
  linkCopied.value = true;
  setTimeout(() => {
    linkCopied.value = false;
  }, 2000);
}

// Sync with remote playback state
watch(isPlaying, (playing) => {
  if (!videoRef.value) return;
  
  if (playing && videoRef.value.paused) {
    videoRef.value.play();
  } else if (!playing && !videoRef.value.paused) {
    videoRef.value.pause();
  }
});

watch(currentPosition, (pos) => {
  if (!videoRef.value || isHost.value) return;
  
  // Only sync if difference is significant (> 2 seconds)
  const diff = Math.abs(videoRef.value.currentTime - pos);
  if (diff > 2) {
    videoRef.value.currentTime = pos;
  }
});

watch(playbackSpeed, (speed) => {
  if (videoRef.value) {
    videoRef.value.playbackRate = speed;
  }
});

// Periodic sync check for non-hosts
onMounted(() => {
  if (!isHost.value) {
    syncCheckInterval = setInterval(async () => {
      if (!videoRef.value || !isPlaying.value) return;
      
      const expected = await getExpectedPosition();
      const diff = Math.abs(videoRef.value.currentTime - expected);
      
      // Correct if more than 1 second off
      if (diff > 1) {
        videoRef.value.currentTime = expected;
      }
    }, 5000);
  }
});

onUnmounted(() => {
  if (controlsTimeout) clearTimeout(controlsTimeout);
  if (syncCheckInterval) clearInterval(syncCheckInterval);
});
</script>

<template>
  <div
    ref="containerRef"
    class="relative bg-black rounded-lg overflow-hidden group"
    @mousemove="handleMouseMove"
    @mouseleave="showControls = false"
  >
    <!-- Video Element -->
    <video
      ref="videoRef"
      class="w-full h-full"
      :src="videoSrc"
      @timeupdate="handleTimeUpdate"
      @loadedmetadata="handleLoadedMetadata"
      @progress="handleProgress"
      @ended="emit('ended')"
      @click="togglePlay"
    />

    <!-- Controls Overlay -->
    <div
      class="absolute inset-0 flex flex-col justify-end transition-opacity duration-300"
      :class="showControls ? 'opacity-100' : 'opacity-0'"
    >
      <!-- Gradient Background -->
      <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent pointer-events-none" />

      <!-- Top Bar -->
      <div class="absolute top-0 left-0 right-0 p-4 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Users class="w-4 h-4 text-white/80" />
          <span class="text-white/80 text-sm">{{ peerCount + 1 }} watching</span>
        </div>
        
        <button
          v-if="isHost"
          class="flex items-center gap-2 px-3 py-1.5 bg-white/10 hover:bg-white/20 rounded-full text-white text-sm transition-colors"
          @click="showShareDialog = true"
        >
          <Link class="w-4 h-4" />
          Share
        </button>
      </div>

      <!-- Progress Bar -->
      <div class="px-4 mb-2">
        <div
          class="relative h-1 bg-white/20 rounded-full cursor-pointer group/progress"
          @click="handleSeek"
        >
          <!-- Buffered -->
          <div
            class="absolute h-full bg-white/30 rounded-full"
            :style="{ width: `${buffered}%` }"
          />
          <!-- Progress -->
          <div
            class="absolute h-full bg-blue-500 rounded-full"
            :style="{ width: `${progress}%` }"
          />
          <!-- Hover indicator -->
          <div
            class="absolute h-3 w-3 bg-blue-500 rounded-full -top-1 transform -translate-x-1/2 opacity-0 group-hover/progress:opacity-100 transition-opacity"
            :style="{ left: `${progress}%` }"
          />
        </div>
      </div>

      <!-- Control Buttons -->
      <div class="px-4 pb-4 flex items-center gap-4">
        <!-- Play/Pause -->
        <button
          class="p-2 hover:bg-white/10 rounded-full transition-colors"
          @click="togglePlay"
        >
          <component
            :is="isPlaying ? Pause : Play"
            class="w-6 h-6 text-white"
          />
        </button>

        <!-- Volume -->
        <div class="flex items-center gap-2">
          <button
            class="p-2 hover:bg-white/10 rounded-full transition-colors"
            @click="toggleMute"
          >
            <component
              :is="isMuted ? VolumeX : Volume2"
              class="w-5 h-5 text-white"
            />
          </button>
          <input
            type="range"
            min="0"
            max="1"
            step="0.1"
            :value="volume"
            class="w-20 accent-blue-500"
            @input="handleVolumeChange"
          />
        </div>

        <!-- Time -->
        <span class="text-white text-sm">{{ formattedTime }}</span>

        <!-- Spacer -->
        <div class="flex-1" />

        <!-- Sync indicator -->
        <div
          v-if="Math.abs(syncOffset) > 0.5"
          class="text-yellow-400 text-xs"
        >
          Syncing...
        </div>

        <!-- Speed -->
        <select
          :value="playbackSpeed"
          class="bg-transparent text-white text-sm border border-white/20 rounded px-2 py-1"
          @change="(e) => setSpeed(parseFloat((e.target as HTMLSelectElement).value))"
        >
          <option value="0.5">0.5x</option>
          <option value="1">1x</option>
          <option value="1.5">1.5x</option>
          <option value="2">2x</option>
        </select>

        <!-- Fullscreen -->
        <button
          class="p-2 hover:bg-white/10 rounded-full transition-colors"
          @click="toggleFullscreen"
        >
          <Maximize class="w-5 h-5 text-white" />
        </button>
      </div>
    </div>

    <!-- Share Dialog -->
    <div
      v-if="showShareDialog"
      class="absolute inset-0 bg-black/80 flex items-center justify-center"
      @click.self="showShareDialog = false"
    >
      <div class="bg-zinc-900 rounded-lg p-6 max-w-md w-full mx-4">
        <h3 class="text-white text-lg font-semibold mb-4">Share Watch Party</h3>
        
        <p class="text-zinc-400 text-sm mb-4">
          Share this link with friends to watch together in sync:
        </p>
        
        <div class="flex gap-2">
          <input
            type="text"
            :value="shareLink"
            readonly
            class="flex-1 bg-zinc-800 text-white px-3 py-2 rounded text-sm"
          />
          <button
            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded text-sm transition-colors"
            @click="copyShareLink"
          >
            {{ linkCopied ? 'Copied!' : 'Copy' }}
          </button>
        </div>
        
        <button
          class="mt-4 w-full py-2 bg-zinc-800 hover:bg-zinc-700 text-white rounded text-sm transition-colors"
          @click="showShareDialog = false"
        >
          Close
        </button>
      </div>
    </div>
  </div>
</template>
