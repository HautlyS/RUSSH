<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useConnectionStore } from '@/stores/connections';
import { useNotificationStore } from '@/stores/notifications';
import ConnectionForm from '@/components/connections/ConnectionForm.vue';
import type { ConnectionProfile } from '@/types/ssh';

const router = useRouter();
const connectionStore = useConnectionStore();
const notificationStore = useNotificationStore();

async function handleSubmit(profile: Omit<ConnectionProfile, 'id' | 'useCount'>) {
  try {
    await connectionStore.createProfile(profile);
    notificationStore.success('Connection Created', `${profile.name} has been created`);
    router.push('/connections');
  } catch (e) {
    notificationStore.error('Failed to Create', String(e));
  }
}

function handleCancel() {
  router.back();
}
</script>

<template>
  <div class="p-6 max-w-2xl mx-auto">
    <h1 class="text-2xl font-bold mb-6">New Connection</h1>
    <div class="card p-6">
      <ConnectionForm 
        mode="create"
        @submit="handleSubmit"
        @cancel="handleCancel"
      />
    </div>
  </div>
</template>
