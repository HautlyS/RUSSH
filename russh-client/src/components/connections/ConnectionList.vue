<script setup lang="ts">
import { computed } from 'vue';
import { useConnectionStore } from '@/stores/connections';
import ConnectionItem from './ConnectionItem.vue';
import type { ConnectionProfile } from '@/types/ssh';

const connectionStore = useConnectionStore();

const props = defineProps<{
  filter?: string;
  showFolders?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', profile: ConnectionProfile): void;
  (e: 'connect', profile: ConnectionProfile): void;
  (e: 'edit', profile: ConnectionProfile): void;
  (e: 'delete', profile: ConnectionProfile): void;
}>();

const filteredProfiles = computed(() => {
  let profiles = connectionStore.profiles;
  if (props.filter) {
    const search = props.filter.toLowerCase();
    profiles = profiles.filter(p => 
      p.name.toLowerCase().includes(search) ||
      p.host.toLowerCase().includes(search) ||
      p.username.toLowerCase().includes(search)
    );
  }
  return profiles;
});

const groupedProfiles = computed(() => {
  if (!props.showFolders) {
    return { '': filteredProfiles.value };
  }
  
  const groups: Record<string, ConnectionProfile[]> = {};
  for (const profile of filteredProfiles.value) {
    const folder = profile.folder || '';
    if (!groups[folder]) groups[folder] = [];
    groups[folder].push(profile);
  }
  return groups;
});

function handleDragStart(e: DragEvent, profile: ConnectionProfile) {
  e.dataTransfer?.setData('profile-id', profile.id);
}

function handleDrop(e: DragEvent, targetFolder: string) {
  const profileId = e.dataTransfer?.getData('profile-id');
  if (profileId) {
    connectionStore.moveToFolder(profileId, targetFolder);
  }
}
</script>

<template>
  <div class="connection-list">
    <div v-if="filteredProfiles.length === 0" class="p-4 text-center text-gray-500 dark:text-gray-400">
      <p v-if="filter">No connections match "{{ filter }}"</p>
      <p v-else>No connections yet</p>
    </div>
    
    <template v-for="(profiles, folder) in groupedProfiles" :key="folder">
      <div 
        v-if="showFolders && folder" 
        class="folder-header px-3 py-2 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider"
        @dragover.prevent
        @drop="handleDrop($event, folder)"
      >
        {{ folder }}
      </div>
      
      <ConnectionItem
        v-for="profile in profiles"
        :key="profile.id"
        :profile="profile"
        draggable="true"
        @dragstart="handleDragStart($event, profile)"
        @click="emit('select', profile)"
        @connect="emit('connect', profile)"
        @edit="emit('edit', profile)"
        @delete="emit('delete', profile)"
      />
    </template>
  </div>
</template>
