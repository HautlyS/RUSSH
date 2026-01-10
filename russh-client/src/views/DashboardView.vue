<script setup lang="ts">
import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { usePlatform } from '@/composables/usePlatform';
import { Plus, Clock, Star, Server, ArrowRight, Zap, Activity, Shield, Terminal } from 'lucide-vue-next';

const router = useRouter();
const connectionStore = useConnectionStore();
const { isMobile, hapticFeedback } = usePlatform();

const recentProfiles = computed(() => connectionStore.recentProfiles);
const favoriteProfiles = computed(() => connectionStore.favoriteProfiles);
const connectedCount = computed(() => connectionStore.connectedProfiles.length);
const totalProfiles = computed(() => connectionStore.profiles.length);

const stats = [
  { icon: Activity, value: connectedCount, label: 'ACTIVE', color: 'green' },
  { icon: Star, value: computed(() => favoriteProfiles.value.length), label: 'STARRED', color: 'yellow' },
  { icon: Server, value: totalProfiles, label: 'TOTAL', color: 'blue' },
  { icon: Clock, value: computed(() => recentProfiles.value.length), label: 'RECENT', color: 'purple' },
];

const quickActions = [
  { icon: Plus, title: 'NEW', desc: 'SSH profile', path: '/connections/new' },
  { icon: Zap, title: 'P2P', desc: 'NAT traversal', path: '/p2p' },
  { icon: Terminal, title: 'QUICK', desc: 'One-time', path: '/connections?quick=true' },
  { icon: Shield, title: 'KEYS', desc: 'Manage', path: '/settings?tab=keys' },
];

function getStatValue(stat: typeof stats[0]) {
  return typeof stat.value === 'object' ? stat.value.value : stat.value;
}

function navigate(path: string) {
  hapticFeedback('light');
  router.push(path);
}

function connectToProfile(id: string) {
  hapticFeedback('medium');
  connectionStore.connect(id);
}
</script>

<template>
  <div class="dash">
    <!-- Header -->
    <header class="dash-header">
      <div>
        <h1 class="dash-title">DASHBOARD</h1>
        <p class="dash-subtitle">SSH SESSION MANAGER</p>
      </div>
      <button @click="navigate('/connections/new')" class="dash-btn-new">
        <Plus class="w-4 h-4" />
        <span class="hidden sm:inline">NEW</span>
      </button>
    </header>
    
    <!-- Stats -->
    <div class="dash-stats">
      <div v-for="stat in stats" :key="stat.label" class="dash-stat">
        <div class="dash-stat-icon" :style="{ '--c': `var(--pixel-${stat.color})` }">
          <component :is="stat.icon" class="w-4 h-4" />
        </div>
        <div class="dash-stat-info">
          <div class="dash-stat-value">{{ getStatValue(stat) }}</div>
          <div class="dash-stat-label">{{ stat.label }}</div>
        </div>
      </div>
    </div>
    
    <!-- Quick Actions -->
    <section class="dash-section">
      <h2 class="dash-section-title">QUICK ACTIONS</h2>
      <div class="dash-actions">
        <button v-for="action in quickActions" :key="action.path" @click="navigate(action.path)" class="dash-action">
          <div class="dash-action-icon">
            <component :is="action.icon" class="w-4 h-4" />
          </div>
          <div class="dash-action-info">
            <div class="dash-action-title">{{ action.title }}</div>
            <div class="dash-action-desc">{{ action.desc }}</div>
          </div>
          <ArrowRight class="w-3 h-3 dash-action-arrow" />
        </button>
      </div>
    </section>
    
    <!-- Recent -->
    <section v-if="recentProfiles.length > 0" class="dash-section">
      <div class="dash-section-header">
        <h2 class="dash-section-title">RECENT SESSIONS</h2>
        <button @click="navigate('/connections')" class="dash-link">
          VIEW ALL <ArrowRight class="w-3 h-3" />
        </button>
      </div>
      
      <div class="dash-recent-list">
        <button v-for="profile in recentProfiles.slice(0, isMobile ? 4 : 5)" :key="profile.id" @click="connectToProfile(profile.id)" class="dash-recent">
          <div class="dash-recent-icon" :style="{ '--c': profile.color || 'var(--pixel-blue)' }">
            <Server class="w-4 h-4" />
          </div>
          <div class="dash-recent-info">
            <div class="dash-recent-name">{{ profile.name }}</div>
            <code class="dash-recent-host">{{ profile.username }}@{{ profile.host }}</code>
          </div>
          <div class="dash-recent-dot" />
        </button>
      </div>
    </section>
    
    <!-- Empty -->
    <div v-if="totalProfiles === 0" class="dash-empty">
      <div class="dash-empty-icon">
        <Server class="w-6 h-6" />
      </div>
      <h3 class="dash-empty-title">NO SESSIONS YET</h3>
      <p class="dash-empty-text">CREATE YOUR FIRST SSH CONNECTION</p>
      <button @click="navigate('/connections/new')" class="dash-btn-new">
        <Plus class="w-4 h-4" />
        NEW SESSION
      </button>
    </div>
  </div>
</template>

<style scoped>
.dash {
  padding: 16px;
  max-width: 900px;
  margin: 0 auto;
}

@media (min-width: 640px) {
  .dash { padding: 24px; }
}

.dash-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.dash-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 14px;
  color: var(--pixel-green);
  text-shadow: 0 0 10px var(--pixel-green);
}

@media (min-width: 640px) {
  .dash-title { font-size: 12px; }
}

.dash-subtitle {
  font-family: 'Press Start 2P', monospace;
  font-size: 9px;
  color: var(--pixel-text);
  margin-top: 4px;
}

@media (min-width: 640px) {
  .dash-subtitle { font-size: 8px; }
}

.dash-btn-new {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  height: 42px;
  padding: 0 16px;
  font-family: 'Press Start 2P', monospace;
  font-size: 9px;
  background: var(--pixel-green-dark);
  border: 3px solid var(--pixel-green);
  color: var(--pixel-black);
  font-weight: bold;
  box-shadow: 3px 3px 0 var(--pixel-black);
  -webkit-tap-highlight-color: transparent;
}

@media (min-width: 640px) {
  .dash-btn-new { height: 34px; padding: 0 14px; font-size: 8px; }
}

.dash-btn-new:active {
  transform: translate(2px, 2px);
  box-shadow: 1px 1px 0 var(--pixel-black);
}

.dash-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
  margin-bottom: 20px;
}

@media (min-width: 640px) {
  .dash-stats { gap: 12px; }
}

.dash-stat {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px;
  background: var(--pixel-dark);
  border: 3px solid var(--pixel-border);
  box-shadow: inset -3px -3px 0 var(--pixel-mid);
}

@media (min-width: 640px) {
  .dash-stat { padding: 12px; gap: 10px; }
}

.dash-stat-icon {
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--c) 20%, transparent);
  border: 2px solid var(--c);
  color: var(--c);
  flex-shrink: 0;
}

@media (min-width: 640px) {
  .dash-stat-icon { width: 38px; height: 38px; }
}

.dash-stat-info {
  flex: 1;
  min-width: 0;
}

.dash-stat-value {
  font-family: 'Press Start 2P', monospace;
  font-size: 20px;
  color: var(--pixel-white);
  line-height: 1;
}

@media (min-width: 640px) {
  .dash-stat-value { font-size: 18px; }
}

.dash-stat-label {
  font-family: 'Press Start 2P', monospace;
  font-size: 7px;
  color: var(--pixel-text);
  margin-top: 6px;
}

.dash-section {
  margin-bottom: 20px;
}

.dash-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.dash-section-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: var(--pixel-text);
}

@media (min-width: 640px) {
  .dash-section-title { font-size: 9px; }
}

.dash-link {
  display: flex;
  align-items: center;
  gap: 4px;
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--pixel-cyan);
}

.dash-link:hover {
  color: var(--pixel-white);
}

.dash-actions {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

@media (min-width: 640px) {
  .dash-actions { grid-template-columns: repeat(4, 1fr); }
}

.dash-action {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  background: var(--pixel-dark);
  border: 2px solid var(--pixel-border);
  text-align: left;
  -webkit-tap-highlight-color: transparent;
}

@media (min-width: 640px) {
  .dash-action { padding: 10px; gap: 8px; }
}

.dash-action:hover {
  background: var(--pixel-mid);
  border-color: var(--pixel-text);
}

.dash-action-icon {
  width: 38px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--pixel-mid);
  border: 2px solid var(--pixel-border);
  color: var(--pixel-text);
  flex-shrink: 0;
}

@media (min-width: 640px) {
  .dash-action-icon { width: 34px; height: 34px; }
}

.dash-action-info {
  flex: 1;
  min-width: 0;
}

.dash-action-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: var(--pixel-white);
}

@media (min-width: 640px) {
  .dash-action-title { font-size: 9px; }
}

.dash-action-desc {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--pixel-muted);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (min-width: 640px) {
  .dash-action-desc { font-size: 7px; }
}

.dash-action-arrow {
  color: var(--pixel-muted);
  flex-shrink: 0;
}

.dash-recent-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.dash-recent {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 12px;
  background: var(--pixel-dark);
  border: 2px solid var(--pixel-border);
  text-align: left;
  -webkit-tap-highlight-color: transparent;
}

@media (min-width: 640px) {
  .dash-recent { padding: 10px; gap: 10px; }
}

.dash-recent:hover {
  background: var(--pixel-mid);
  border-color: var(--pixel-text);
}

.dash-recent-icon {
  width: 42px;
  height: 42px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--c) 20%, transparent);
  border: 2px solid var(--c);
  color: var(--c);
  flex-shrink: 0;
}

@media (min-width: 640px) {
  .dash-recent-icon { width: 34px; height: 34px; }
}

.dash-recent-info {
  flex: 1;
  min-width: 0;
}

.dash-recent-name {
  font-family: 'Press Start 2P', monospace;
  font-size: 10px;
  color: var(--pixel-white);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (min-width: 640px) {
  .dash-recent-name { font-size: 9px; }
}

.dash-recent-host {
  font-family: 'Press Start 2P', monospace;
  font-size: 8px;
  color: var(--pixel-text);
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-top: 2px;
}

@media (min-width: 640px) {
  .dash-recent-host { font-size: 7px; }
}

.dash-recent-dot {
  width: 10px;
  height: 10px;
  background: var(--pixel-border);
  border: 2px solid var(--pixel-muted);
  flex-shrink: 0;
}

.dash-recent:hover .dash-recent-dot {
  background: var(--pixel-green);
  border-color: var(--pixel-green-dark);
  box-shadow: 0 0 8px var(--pixel-green);
}

.dash-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 16px;
  text-align: center;
}

.dash-empty-icon {
  width: 60px;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--pixel-dark);
  border: 3px solid var(--pixel-border);
  color: var(--pixel-muted);
  margin-bottom: 16px;
}

.dash-empty-title {
  font-family: 'Press Start 2P', monospace;
  font-size: 12px;
  color: var(--pixel-white);
  margin-bottom: 8px;
}

@media (min-width: 640px) {
  .dash-empty-title { font-size: 11px; }
}

.dash-empty-text {
  font-family: 'Press Start 2P', monospace;
  font-size: 9px;
  color: var(--pixel-text);
  margin-bottom: 20px;
}

@media (min-width: 640px) {
  .dash-empty-text { font-size: 8px; }
}
</style>
