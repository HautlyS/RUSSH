<script setup lang="ts">
import { computed } from 'vue';
import { Server, Play, Edit, Trash2, Star, Terminal } from 'lucide-vue-next';
import { useConnectionStore } from '@/stores/connections';
import { usePlatform } from '@/composables/usePlatform';
import type { ConnectionProfile } from '@/types/ssh';

const props = defineProps<{
  profile: ConnectionProfile;
}>();

const emit = defineEmits<{
  (e: 'connect'): void;
  (e: 'edit'): void;
  (e: 'delete'): void;
}>();

const connectionStore = useConnectionStore();
const { hapticFeedback } = usePlatform();

const isConnected = computed(() => 
  connectionStore.activeConnections.has(props.profile.id)
);

const connectionState = computed(() => 
  connectionStore.connectionStates.get(props.profile.id)
);

const isFavorite = computed(() => 
  props.profile.tags?.includes('favorite')
);

const status = computed(() => connectionState.value?.status || 'idle');

function handleConnect() {
  hapticFeedback('medium');
  emit('connect');
}

function handleEdit() {
  hapticFeedback('light');
  emit('edit');
}

function handleDelete() {
  hapticFeedback('warning');
  emit('delete');
}
</script>

<template>
  <article class="card">
    <!-- Status bar -->
    <div :class="['card-status', `card-status--${status}`]" />
    
    <div class="card-body">
      <!-- Header -->
      <div class="card-header">
        <div class="card-icon" :style="{ '--accent': profile.color || 'var(--pixel-blue)' }">
          <Server class="w-4 h-4" />
        </div>
        <div class="card-info">
          <h3 class="card-title">
            {{ profile.name }}
            <Star v-if="isFavorite" class="card-star" />
          </h3>
          <code class="card-host">{{ profile.username }}@{{ profile.host }}:{{ profile.port }}</code>
        </div>
        <div :class="['card-dot', `card-dot--${status}`]" />
      </div>
      
      <!-- Tags -->
      <div v-if="profile.tags?.filter(t => t !== 'favorite').length" class="card-tags">
        <span v-for="tag in profile.tags.filter(t => t !== 'favorite')" :key="tag" class="card-tag">
          {{ tag }}
        </span>
      </div>
      
      <!-- Actions -->
      <div class="card-actions">
        <button v-if="!isConnected" @click="handleConnect" class="card-btn card-btn--primary">
          <Play class="w-3.5 h-3.5" />
          <span>CONNECT</span>
        </button>
        <button v-else @click="handleConnect" class="card-btn card-btn--active">
          <Terminal class="w-3.5 h-3.5" />
          <span>TERMINAL</span>
        </button>
        
        <button @click="handleEdit" class="card-btn card-btn--icon" aria-label="Edit">
          <Edit class="w-4 h-4" />
        </button>
        
        <button @click="handleDelete" class="card-btn card-btn--danger" aria-label="Delete">
          <Trash2 class="w-4 h-4" />
        </button>
      </div>
    </div>
  </article>
</template>

<style scoped>
.card {
  background: var(--pixel-dark);
  border: 3px solid var(--pixel-border);
  box-shadow: inset -3px -3px 0 var(--pixel-mid), 4px 4px 0 var(--pixel-black);
  transition: transform 100ms, box-shadow 100ms;
}

.card:hover {
  transform: translate(-2px, -2px);
  box-shadow: inset -3px -3px 0 var(--pixel-mid), 6px 6px 0 var(--pixel-black);
}

@media (hover: none) {
  .card:hover { transform: none; box-shadow: inset -3px -3px 0 var(--pixel-mid), 4px 4px 0 var(--pixel-black); }
}

.card-status {
  height: 4px;
  width: 100%;
}

.card-status--connected { background: var(--pixel-green); box-shadow: 0 0 8px var(--pixel-green); }
.card-status--connecting { background: var(--pixel-yellow); animation: pulse 1s infinite; }
.card-status--error { background: var(--pixel-red); }
.card-status--idle { background: var(--pixel-border); }

.card-body {
  padding: 14px;
}

@media (min-width: 640px) {
  .card-body { padding: 16px; }
}

.card-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 12px;
}

.card-icon {
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  background: color-mix(in srgb, var(--accent) 20%, transparent);
  border: 2px solid var(--accent);
  color: var(--accent);
}

@media (min-width: 640px) {
  .card-icon { width: 38px; height: 38px; }
}

.card-info {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 11px;
  color: var(--pixel-white);
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 4px;
}

@media (min-width: 640px) {
  .card-title { font-size: 10px; }
}

.card-star {
  width: 12px;
  height: 12px;
  color: var(--pixel-yellow);
  fill: var(--pixel-yellow);
  flex-shrink: 0;
}

.card-host {
  font-family: 'Press Start 2P', monospace;
  font-size: 9px;
  color: var(--pixel-text);
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (min-width: 640px) {
  .card-host { font-size: 8px; }
}

.card-dot {
  width: 10px;
  height: 10px;
  flex-shrink: 0;
  margin-top: 4px;
  border: 2px solid;
}

.card-dot--connected { background: var(--pixel-green); border-color: var(--pixel-green-dark); box-shadow: 0 0 8px var(--pixel-green); }
.card-dot--connecting { background: var(--pixel-yellow); border-color: var(--pixel-orange); animation: blink 0.5s steps(2) infinite; }
.card-dot--error { background: var(--pixel-red); border-color: var(--pixel-red-dark); }
.card-dot--idle { background: var(--pixel-muted); border-color: var(--pixel-border); }

.card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}

.card-tag {
  padding: 4px 8px;
  font-family: 'Press Start 2P', monospace;
  font-size: 7px;
  background: var(--pixel-mid);
  border: 2px solid var(--pixel-border);
  color: var(--pixel-text);
}

.card-actions {
  display: flex;
  gap: 8px;
}

.card-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 42px;
  padding: 0 14px;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  border: 2px solid var(--pixel-border);
  background: var(--pixel-mid);
  color: var(--pixel-white);
  -webkit-tap-highlight-color: transparent;
}

@media (min-width: 640px) {
  .card-btn { height: 34px; padding: 0 12px; font-size: 7px; }
}

.card-btn:active {
  transform: translate(2px, 2px);
}

.card-btn--primary {
  flex: 1;
  background: var(--pixel-green-dark);
  border-color: var(--pixel-green);
  color: var(--pixel-black);
  font-weight: bold;
}

.card-btn--primary:hover {
  background: var(--pixel-green);
}

.card-btn--active {
  flex: 1;
  background: rgba(0, 255, 136, 0.15);
  border-color: var(--pixel-green);
  color: var(--pixel-green);
}

.card-btn--icon {
  width: 42px;
  padding: 0;
  flex-shrink: 0;
}

@media (min-width: 640px) {
  .card-btn--icon { width: 34px; }
}

.card-btn--danger {
  width: 42px;
  padding: 0;
  flex-shrink: 0;
  background: transparent;
  border-color: var(--pixel-border);
  color: var(--pixel-muted);
}

@media (min-width: 640px) {
  .card-btn--danger { width: 34px; }
}

.card-btn--danger:hover {
  background: rgba(255, 71, 87, 0.2);
  border-color: var(--pixel-red);
  color: var(--pixel-red);
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}
</style>
