<script setup lang="ts">
/**
 * StreamingView - Watch party / synchronized streaming view
 */
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { Film, Plus, Link, Users, ArrowLeft } from 'lucide-vue-next';
import SyncVideoPlayer from '@/components/streaming/SyncVideoPlayer.vue';
import { useStreaming } from '@/composables/useStreaming';
import type { CreateStreamRequest } from '@/types/streaming';

const route = useRoute();
const router = useRouter();

const {
  room,
  isHost,
  isLoading,
  error,
  peerCount,
  shareLink,
  createRoom,
  joinRoom,
  leaveRoom,
} = useStreaming();

const showCreateDialog = ref(false);
const showJoinDialog = ref(false);
const newRoomName = ref('');
const newRoomUrl = ref('');
const joinLink = ref('');

async function handleCreateRoom() {
  if (!newRoomName.value || !newRoomUrl.value) return;
  
  const request: CreateStreamRequest = {
    name: newRoomName.value,
    sourceType: 'url',
    url: newRoomUrl.value,
  };
  
  await createRoom(request);
  showCreateDialog.value = false;
  newRoomName.value = '';
  newRoomUrl.value = '';
}

async function handleJoinRoom() {
  if (!joinLink.value) return;
  
  // Parse link: russh://stream/{roomId}?host={hostId}
  const match = joinLink.value.match(/russh:\/\/stream\/([^?]+)\?host=(.+)/);
  if (!match) {
    error.value = 'Invalid share link';
    return;
  }
  
  const [, roomId, hostId] = match;
  await joinRoom(roomId, hostId);
  showJoinDialog.value = false;
  joinLink.value = '';
}

async function handleLeave() {
  await leaveRoom();
}

// Check for join link in URL
onMounted(() => {
  const roomId = route.query.room as string;
  const hostId = route.query.host as string;
  
  if (roomId && hostId) {
    joinRoom(roomId, hostId);
  }
});
</script>

<template>
  <div class="min-h-screen bg-zinc-950 text-white">
    <!-- Header -->
    <header class="border-b border-zinc-800 px-6 py-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-4">
          <button
            class="p-2 hover:bg-zinc-800 rounded-lg transition-colors"
            @click="router.back()"
          >
            <ArrowLeft class="w-5 h-5" />
          </button>
          <div class="flex items-center gap-2">
            <Film class="w-6 h-6 text-blue-500" />
            <h1 class="text-xl font-semibold">Watch Party</h1>
          </div>
        </div>
        
        <div v-if="!room" class="flex items-center gap-3">
          <button
            class="flex items-center gap-2 px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
            @click="showJoinDialog = true"
          >
            <Link class="w-4 h-4" />
            Join Party
          </button>
          <button
            class="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
            @click="showCreateDialog = true"
          >
            <Plus class="w-4 h-4" />
            Create Party
          </button>
        </div>
        
        <div v-else class="flex items-center gap-4">
          <div class="flex items-center gap-2 text-zinc-400">
            <Users class="w-4 h-4" />
            <span>{{ peerCount + 1 }} watching</span>
          </div>
          <button
            class="px-4 py-2 bg-red-600/20 hover:bg-red-600/30 text-red-400 rounded-lg transition-colors"
            @click="handleLeave"
          >
            Leave Party
          </button>
        </div>
      </div>
    </header>

    <!-- Main Content -->
    <main class="p-6">
      <!-- No Room State -->
      <div
        v-if="!room"
        class="flex flex-col items-center justify-center min-h-[60vh] text-center"
      >
        <Film class="w-16 h-16 text-zinc-700 mb-4" />
        <h2 class="text-2xl font-semibold mb-2">Watch Together</h2>
        <p class="text-zinc-400 max-w-md mb-6">
          Create a watch party to stream videos with friends in perfect sync.
          Share a link and everyone watches together!
        </p>
        <div class="flex gap-3">
          <button
            class="flex items-center gap-2 px-6 py-3 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
            @click="showJoinDialog = true"
          >
            <Link class="w-5 h-5" />
            Join with Link
          </button>
          <button
            class="flex items-center gap-2 px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
            @click="showCreateDialog = true"
          >
            <Plus class="w-5 h-5" />
            Create Party
          </button>
        </div>
      </div>

      <!-- Active Room -->
      <div v-else class="max-w-6xl mx-auto">
        <div class="mb-4">
          <h2 class="text-xl font-semibold">{{ room.name }}</h2>
          <p class="text-zinc-400 text-sm">
            {{ isHost ? 'You are the host' : `Hosted by ${room.hostId.slice(0, 8)}...` }}
          </p>
        </div>

        <!-- Video Player -->
        <div class="aspect-video bg-zinc-900 rounded-lg overflow-hidden">
          <SyncVideoPlayer />
        </div>

        <!-- Room Info -->
        <div class="mt-4 p-4 bg-zinc-900 rounded-lg">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-zinc-400">Share Link</p>
              <p class="text-sm font-mono text-zinc-300 truncate max-w-md">
                {{ shareLink }}
              </p>
            </div>
            <button
              class="px-4 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg text-sm transition-colors"
              @click="navigator.clipboard.writeText(shareLink)"
            >
              Copy Link
            </button>
          </div>
        </div>
      </div>
    </main>

    <!-- Create Dialog -->
    <div
      v-if="showCreateDialog"
      class="fixed inset-0 bg-black/80 flex items-center justify-center z-50"
      @click.self="showCreateDialog = false"
    >
      <div class="bg-zinc-900 rounded-lg p-6 max-w-md w-full mx-4">
        <h3 class="text-lg font-semibold mb-4">Create Watch Party</h3>
        
        <div class="space-y-4">
          <div>
            <label class="block text-sm text-zinc-400 mb-1">Party Name</label>
            <input
              v-model="newRoomName"
              type="text"
              placeholder="Movie Night"
              class="w-full bg-zinc-800 text-white px-3 py-2 rounded-lg"
            />
          </div>
          
          <div>
            <label class="block text-sm text-zinc-400 mb-1">Video URL</label>
            <input
              v-model="newRoomUrl"
              type="url"
              placeholder="https://example.com/video.mp4"
              class="w-full bg-zinc-800 text-white px-3 py-2 rounded-lg"
            />
          </div>
        </div>
        
        <div class="flex gap-3 mt-6">
          <button
            class="flex-1 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
            @click="showCreateDialog = false"
          >
            Cancel
          </button>
          <button
            class="flex-1 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
            :disabled="isLoading || !newRoomName || !newRoomUrl"
            @click="handleCreateRoom"
          >
            {{ isLoading ? 'Creating...' : 'Create' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Join Dialog -->
    <div
      v-if="showJoinDialog"
      class="fixed inset-0 bg-black/80 flex items-center justify-center z-50"
      @click.self="showJoinDialog = false"
    >
      <div class="bg-zinc-900 rounded-lg p-6 max-w-md w-full mx-4">
        <h3 class="text-lg font-semibold mb-4">Join Watch Party</h3>
        
        <div>
          <label class="block text-sm text-zinc-400 mb-1">Share Link</label>
          <input
            v-model="joinLink"
            type="text"
            placeholder="russh://stream/..."
            class="w-full bg-zinc-800 text-white px-3 py-2 rounded-lg"
          />
        </div>
        
        <p v-if="error" class="text-red-400 text-sm mt-2">{{ error }}</p>
        
        <div class="flex gap-3 mt-6">
          <button
            class="flex-1 py-2 bg-zinc-800 hover:bg-zinc-700 rounded-lg transition-colors"
            @click="showJoinDialog = false"
          >
            Cancel
          </button>
          <button
            class="flex-1 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg transition-colors"
            :disabled="isLoading || !joinLink"
            @click="handleJoinRoom"
          >
            {{ isLoading ? 'Joining...' : 'Join' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
