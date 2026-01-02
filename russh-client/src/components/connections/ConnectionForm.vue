<script setup lang="ts">
import { ref, computed, reactive, watch } from 'vue';
import { useConnectionStore } from '@/stores/connections';
import { Server, Key, Lock, Folder, Tag } from 'lucide-vue-next';
import type { ConnectionProfile } from '@/types/ssh';

const props = defineProps<{
  profile?: ConnectionProfile;
  mode: 'create' | 'edit';
}>();

const emit = defineEmits<{
  submit: [profile: Omit<ConnectionProfile, 'id' | 'useCount'> & { id?: string }];
  cancel: [];
}>();

const connectionStore = useConnectionStore();

const form = reactive({
  name: props.profile?.name || '',
  host: props.profile?.host || '',
  port: props.profile?.port || 22,
  username: props.profile?.username || '',
  authType: props.profile?.authType || 'password',
  keyPath: props.profile?.keyPath || '',
  folder: props.profile?.folder || '',
  color: props.profile?.color || '',
  tags: props.profile?.tags || [],
  autoReconnect: props.profile?.autoReconnect ?? true,
});

const newTag = ref('');
const newFolder = ref('');
const showNewFolder = ref(false);

const colors = [
  '#ef4444', '#f97316', '#eab308', '#22c55e', 
  '#06b6d4', '#3b82f6', '#8b5cf6', '#ec4899'
];

const folders = computed(() => connectionStore.folders);

const isValid = computed(() => 
  form.name.trim() && 
  form.host.trim() && 
  form.username.trim() &&
  form.port >= 1 && form.port <= 65535 &&
  (form.authType !== 'key' || form.keyPath.trim())
);

function addTag() {
  const tag = newTag.value.trim();
  if (tag && !form.tags.includes(tag)) {
    form.tags.push(tag);
    newTag.value = '';
  }
}

function removeTag(tag: string) {
  form.tags = form.tags.filter(t => t !== tag);
}

function createFolder() {
  if (newFolder.value.trim()) {
    form.folder = newFolder.value.trim();
    showNewFolder.value = false;
    newFolder.value = '';
  }
}

function handleSubmit() {
  if (!isValid.value) return;
  
  emit('submit', {
    id: props.profile?.id,
    name: form.name.trim(),
    host: form.host.trim(),
    port: form.port,
    username: form.username.trim(),
    authType: form.authType as 'password' | 'key' | 'agent',
    keyPath: form.keyPath || undefined,
    folder: form.folder || undefined,
    color: form.color || undefined,
    tags: form.tags,
    autoReconnect: form.autoReconnect,
    lastConnected: props.profile?.lastConnected,
  });
}

// Watch for profile changes (edit mode)
watch(() => props.profile, (newProfile) => {
  if (newProfile) {
    Object.assign(form, {
      name: newProfile.name,
      host: newProfile.host,
      port: newProfile.port,
      username: newProfile.username,
      authType: newProfile.authType,
      keyPath: newProfile.keyPath || '',
      folder: newProfile.folder || '',
      color: newProfile.color || '',
      tags: [...newProfile.tags],
      autoReconnect: newProfile.autoReconnect,
    });
  }
}, { immediate: true });
</script>

<template>
  <form @submit.prevent="handleSubmit" class="space-y-6">
    <!-- Basic Info Section -->
    <section>
      <h3 class="text-lg font-medium mb-4 flex items-center gap-2">
        <Server class="w-5 h-5" />
        Basic Information
      </h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="md:col-span-2">
          <label class="block text-sm font-medium mb-1">Connection Name *</label>
          <input 
            v-model="form.name"
            type="text"
            required
            placeholder="My Server"
            class="input"
          />
        </div>
        
        <div>
          <label class="block text-sm font-medium mb-1">Folder</label>
          <div v-if="!showNewFolder" class="flex gap-2">
            <select 
              v-model="form.folder"
              class="input flex-1"
            >
              <option value="">No folder</option>
              <option v-for="folder in folders" :key="folder" :value="folder">{{ folder }}</option>
            </select>
            <button 
              type="button"
              @click="showNewFolder = true"
              class="btn-secondary"
            >
              <Folder class="w-4 h-4" />
            </button>
          </div>
          <div v-else class="flex gap-2">
            <input 
              v-model="newFolder"
              type="text"
              placeholder="New folder name"
              class="input flex-1"
              @keydown.enter.prevent="createFolder"
            />
            <button type="button" @click="createFolder" class="btn-primary">Add</button>
            <button type="button" @click="showNewFolder = false" class="btn-secondary">Cancel</button>
          </div>
        </div>
        
        <div>
          <label class="block text-sm font-medium mb-1">Color</label>
          <div class="flex gap-2">
            <button 
              v-for="color in colors" 
              :key="color"
              type="button"
              @click="form.color = form.color === color ? '' : color"
              :class="[
                'w-8 h-8 rounded-full border-2 transition-transform hover:scale-110',
                form.color === color ? 'border-gray-900 dark:border-white scale-110' : 'border-transparent'
              ]"
              :style="{ backgroundColor: color }"
            />
          </div>
        </div>
        
        <div class="md:col-span-2">
          <label class="block text-sm font-medium mb-1">Tags</label>
          <div class="flex flex-wrap gap-2 mb-2">
            <span 
              v-for="tag in form.tags" 
              :key="tag"
              class="inline-flex items-center gap-1 px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded-full text-sm"
            >
              {{ tag }}
              <button type="button" @click="removeTag(tag)" class="hover:text-red-500">&times;</button>
            </span>
          </div>
          <div class="flex gap-2">
            <input 
              v-model="newTag"
              type="text"
              placeholder="Add tag..."
              class="input flex-1"
              @keydown.enter.prevent="addTag"
            />
            <button type="button" @click="addTag" class="btn-secondary">
              <Tag class="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>
    </section>
    
    <!-- Connection Section -->
    <section>
      <h3 class="text-lg font-medium mb-4 flex items-center gap-2">
        <Server class="w-5 h-5" />
        Connection
      </h3>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="md:col-span-2">
          <label class="block text-sm font-medium mb-1">Host *</label>
          <input 
            v-model="form.host"
            type="text"
            required
            placeholder="example.com or 192.168.1.1"
            class="input"
          />
        </div>
        
        <div>
          <label class="block text-sm font-medium mb-1">Port *</label>
          <input 
            v-model.number="form.port"
            type="number"
            required
            min="1"
            max="65535"
            class="input"
          />
        </div>
      </div>
    </section>
    
    <!-- Authentication Section -->
    <section>
      <h3 class="text-lg font-medium mb-4 flex items-center gap-2">
        <Lock class="w-5 h-5" />
        Authentication
      </h3>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium mb-1">Username *</label>
          <input 
            v-model="form.username"
            type="text"
            required
            placeholder="root"
            class="input"
          />
        </div>
        
        <div>
          <label class="block text-sm font-medium mb-2">Authentication Method</label>
          <div class="flex gap-4">
            <label class="flex items-center gap-2 cursor-pointer">
              <input 
                v-model="form.authType" 
                type="radio" 
                value="password"
                class="w-4 h-4 text-blue-500"
              />
              <span>Password</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input 
                v-model="form.authType" 
                type="radio" 
                value="key"
                class="w-4 h-4 text-blue-500"
              />
              <span>SSH Key</span>
            </label>
            <label class="flex items-center gap-2 cursor-pointer">
              <input 
                v-model="form.authType" 
                type="radio" 
                value="agent"
                class="w-4 h-4 text-blue-500"
              />
              <span>SSH Agent</span>
            </label>
          </div>
        </div>
        
        <div v-if="form.authType === 'key'">
          <label class="block text-sm font-medium mb-1">Private Key Path *</label>
          <div class="flex gap-2">
            <input 
              v-model="form.keyPath"
              type="text"
              placeholder="~/.ssh/id_rsa"
              class="input flex-1"
            />
            <button type="button" class="btn-secondary">
              <Key class="w-4 h-4" />
              Browse
            </button>
          </div>
        </div>
      </div>
    </section>
    
    <!-- Options Section -->
    <section>
      <h3 class="text-lg font-medium mb-4">Options</h3>
      <label class="flex items-center gap-2 cursor-pointer">
        <input 
          v-model="form.autoReconnect"
          type="checkbox"
          class="w-4 h-4 text-blue-500 rounded"
        />
        <span>Auto-reconnect on disconnect</span>
      </label>
    </section>
    
    <!-- Actions -->
    <div class="flex justify-end gap-3 pt-4 border-t border-gray-200 dark:border-gray-700">
      <button type="button" @click="$emit('cancel')" class="btn-secondary">
        Cancel
      </button>
      <button type="submit" :disabled="!isValid" class="btn-primary">
        {{ mode === 'create' ? 'Create Connection' : 'Save Changes' }}
      </button>
    </div>
  </form>
</template>
