<script setup lang="ts">
import { computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { useNotificationStore } from '@/stores/notifications';
import ConnectionForm from '@/components/connections/ConnectionForm.vue';
import type { ConnectionProfile } from '@/types/ssh';

const router = useRouter();
const route = useRoute();
const connectionStore = useConnectionStore();
const notificationStore = useNotificationStore();

const profileId = computed(() => route.params.id as string);
const profile = computed(() => 
  connectionStore.profiles.find(p => p.id === profileId.value)
);

async function handleSubmit(updatedProfile: Omit<ConnectionProfile, 'useCount'> & { id?: string }) {
  try {
    await connectionStore.updateProfile({
      ...updatedProfile,
      id: profileId.value,
      useCount: profile.value?.useCount || 0,
    } as ConnectionProfile);
    notificationStore.success('Connection Updated', `${updatedProfile.name} has been updated`);
    router.push('/connections');
  } catch (e) {
    notificationStore.error('Failed to Update', String(e));
  }
}

function handleCancel() {
  router.back();
}
</script>

<template>
  <div class="p-6 max-w-2xl mx-auto">
    <h1 class="text-2xl font-bold mb-6">Edit Connection</h1>
    <div v-if="profile" class="card p-6">
      <ConnectionForm 
        :profile="profile"
        mode="edit"
        @submit="handleSubmit"
        @cancel="handleCancel"
      />
    </div>
    <div v-else class="text-center py-12">
      <p class="text-gray-500">Connection not found</p>
    </div>
  </div>
</template>
