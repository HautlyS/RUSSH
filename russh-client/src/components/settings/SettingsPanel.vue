<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { 
  Settings, Terminal, Palette, Keyboard, Sparkles, ChevronRight
} from 'lucide-vue-next';
import GeneralSettings from './GeneralSettings.vue';
import TerminalSettingsPanel from './TerminalSettingsPanel.vue';
import AppearanceSettings from './AppearanceSettings.vue';
import KeyboardSettings from './KeyboardSettings.vue';
import VisualEffectsSettings from './VisualEffectsSettings.vue';
import DecryptedText from '@/components/extra/DecryptedText.vue';
import { useVisualEffects } from '@/composables/useVisualEffects';
import { usePlatform } from '@/composables/usePlatform';

const route = useRoute();
const router = useRouter();
const { isDecryptedTextEnabled, visualEffects } = useVisualEffects();
const { isMobile } = usePlatform();

const activeSection = ref(route.query.tab as string || 'general');

const sections = [
  { 
    id: 'general', 
    label: 'General', 
    icon: Settings,
    description: 'App behavior & defaults'
  },
  { 
    id: 'terminal', 
    label: 'Terminal', 
    icon: Terminal,
    description: 'Shell & display settings'
  },
  { 
    id: 'appearance', 
    label: 'Appearance', 
    icon: Palette,
    description: 'Theme & colors'
  },
  { 
    id: 'keyboard', 
    label: 'Keyboard', 
    icon: Keyboard,
    description: 'Shortcuts & bindings'
  },
  { 
    id: 'visualEffects', 
    label: 'Effects', 
    icon: Sparkles,
    description: 'Animations & visuals'
  },
];

const activeComponent = computed(() => {
  switch (activeSection.value) {
    case 'terminal': return TerminalSettingsPanel;
    case 'appearance': return AppearanceSettings;
    case 'keyboard': return KeyboardSettings;
    case 'visualEffects': return VisualEffectsSettings;
    default: return GeneralSettings;
  }
});

const activeSectionData = computed(() => 
  sections.find(s => s.id === activeSection.value) || sections[0]
);

function setSection(id: string) {
  activeSection.value = id;
  router.replace({ query: { ...route.query, tab: id } });
}
</script>

<template>
  <div class="h-full flex flex-col lg:flex-row">
    <!-- Sidebar Navigation -->
    <nav 
      :class="[
        'bg-gray-950/50 backdrop-blur-glass border-white/5 p-4 space-y-1',
        isMobile 
          ? 'border-b flex overflow-x-auto gap-2 scrollbar-hide' 
          : 'w-64 border-r flex-shrink-0'
      ]"
    >
      <button
        v-for="section in sections"
        :key="section.id"
        @click="setSection(section.id)"
        :class="[
          'flex items-center gap-3 rounded-xl transition-all group',
          isMobile 
            ? 'flex-shrink-0 px-4 py-2.5' 
            : 'w-full px-4 py-3 text-left',
          activeSection === section.id 
            ? 'bg-green-500/10 text-green-400 border border-green-500/20' 
            : 'text-gray-400 hover:bg-white/5 hover:text-white border border-transparent'
        ]"
      >
        <div 
          :class="[
            'w-9 h-9 rounded-lg flex items-center justify-center transition-colors',
            activeSection === section.id 
              ? 'bg-green-500/20' 
              : 'bg-white/5 group-hover:bg-white/10'
          ]"
        >
          <component :is="section.icon" class="w-5 h-5" />
        </div>
        <div v-if="!isMobile" class="flex-1 min-w-0">
          <div class="font-medium text-sm">{{ section.label }}</div>
          <div class="text-xs text-gray-500 truncate">{{ section.description }}</div>
        </div>
        <ChevronRight 
          v-if="!isMobile && activeSection === section.id" 
          class="w-4 h-4 text-green-400" 
        />
      </button>
    </nav>
    
    <!-- Content Area -->
    <div class="flex-1 overflow-auto">
      <div class="p-6 max-w-4xl mx-auto">
        <!-- Section Header -->
        <div class="mb-8">
          <div class="flex items-center gap-3 mb-2">
            <div class="w-10 h-10 rounded-xl bg-green-500/10 flex items-center justify-center">
              <component :is="activeSectionData.icon" class="w-5 h-5 text-green-400" />
            </div>
            <div>
              <h2 class="text-xl font-bold">
                <DecryptedText
                  v-if="isDecryptedTextEnabled"
                  :text="activeSectionData.label"
                  :speed="visualEffects.decryptedText.speed"
                  animate-on="view"
                  class-name="text-white"
                  encrypted-class-name="text-green-500/30"
                />
                <span v-else class="text-white">{{ activeSectionData.label }}</span>
              </h2>
              <p class="text-sm text-gray-500">{{ activeSectionData.description }}</p>
            </div>
          </div>
        </div>
        
        <!-- Settings Content -->
        <div class="space-y-6 animate-slide-up">
          <component :is="activeComponent" />
        </div>
      </div>
    </div>
  </div>
</template>
