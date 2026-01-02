<script setup lang="ts">
import { ref, onMounted } from 'vue';
import Lightning from '@/components/extra/Lightning.vue';
import DecryptedText from '@/components/extra/DecryptedText.vue';
import ElectricBorder from '@/components/extra/ElectricBorder.vue';
import ClickSpark from '@/components/extra/ClickSpark.vue';
import RotatingText from '@/components/extra/RotatingText.vue';
import GradualBlur from '@/components/extra/GradualBlur.vue';
import Noise from '@/components/extra/Noise.vue';
import { Terminal, Shield, Zap, Globe, Download, Github, Apple, Monitor } from 'lucide-vue-next';

const features = [
  { icon: Terminal, title: 'SSH Terminal', desc: 'Full xterm.js with WebGL rendering' },
  { icon: Shield, title: 'Secure', desc: 'Key-based auth & encrypted sessions' },
  { icon: Zap, title: 'P2P Network', desc: 'Direct peer connections with NAT traversal' },
  { icon: Globe, title: 'Cross-Platform', desc: 'Windows, macOS, Linux, iOS, Android' },
];

const rotatingWords = ['SECURE', 'FAST', 'MODERN', 'POWERFUL'];
const isLoaded = ref(false);

onMounted(() => {
  setTimeout(() => isLoaded.value = true, 100);
});
</script>

<template>
  <ClickSpark spark-color="#27FF64" :spark-count="12" :spark-radius="30" class="min-h-screen">
    <div class="relative min-h-screen bg-black text-white overflow-hidden">
      <!-- Background Effects -->
      <div class="fixed inset-0 z-0">
        <Lightning :hue="140" :intensity="0.6" :speed="0.8" class="opacity-30" />
      </div>
      <Noise :pattern-alpha="8" mix-blend-mode="overlay" class="fixed inset-0 z-10 pointer-events-none" />
      <GradualBlur position="top" height="12rem" :strength="3" class="z-20" />
      <GradualBlur position="bottom" height="16rem" :strength="4" class="z-20" />

      <!-- Content -->
      <div class="relative z-30">
        <!-- Nav -->
        <nav class="flex items-center justify-between px-8 py-6">
          <div class="flex items-center gap-3">
            <div class="w-10 h-10 rounded-lg bg-gradient-to-br from-green-400 to-emerald-600 flex items-center justify-center">
              <Terminal class="w-6 h-6 text-black" />
            </div>
            <span class="text-xl font-bold tracking-tight">RUSSH</span>
          </div>
          <div class="flex items-center gap-6">
            <a href="#features" class="text-gray-400 hover:text-white transition">Features</a>
            <a href="#download" class="text-gray-400 hover:text-white transition">Download</a>
            <a href="https://github.com/HautlyS/RUSSH" target="_blank" 
               class="flex items-center gap-2 px-4 py-2 bg-white/10 rounded-lg hover:bg-white/20 transition">
              <Github class="w-4 h-4" />
              GitHub
            </a>
          </div>
        </nav>

        <!-- Hero -->
        <section class="flex flex-col items-center justify-center min-h-[80vh] px-8 text-center">
          <div class="mb-8">
            <DecryptedText 
              text="RUSSH" 
              :speed="40" 
              :max-iterations="15"
              animate-on="view"
              class-name="text-8xl md:text-9xl font-black tracking-tighter bg-gradient-to-r from-green-400 via-emerald-300 to-cyan-400 bg-clip-text text-transparent"
              encrypted-class-name="text-green-500/50"
            />
          </div>
          
          <p class="text-2xl md:text-3xl text-gray-400 mb-4 flex items-center gap-3">
            The
            <RotatingText 
              :texts="rotatingWords" 
              :rotation-interval="2500"
              :stagger-duration="0.03"
              main-class-name="text-green-400 font-bold"
              split-by="characters"
            />
            SSH Client
          </p>
          
          <p class="text-lg text-gray-500 max-w-2xl mb-12">
            Built with Rust + Tauri for blazing performance. 
            P2P networking, interactive terminal blocks, and stunning visual effects.
          </p>

          <div class="flex flex-wrap gap-4 justify-center">
            <ElectricBorder color="#27FF64" :thickness="2" :speed="1.5" :chaos="0.8" 
                           :style="{ borderRadius: '12px' }">
              <a href="#download" 
                 class="flex items-center gap-3 px-8 py-4 bg-green-500/20 text-green-400 font-semibold rounded-xl hover:bg-green-500/30 transition">
                <Download class="w-5 h-5" />
                Download Now
              </a>
            </ElectricBorder>
            
            <a href="https://github.com/HautlyS/RUSSH" target="_blank"
               class="flex items-center gap-3 px-8 py-4 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 transition">
              <Github class="w-5 h-5" />
              View Source
            </a>
          </div>
        </section>

        <!-- Features -->
        <section id="features" class="py-24 px-8">
          <h2 class="text-4xl font-bold text-center mb-16">
            <DecryptedText text="FEATURES" :speed="30" animate-on="view" 
                          class-name="text-white" encrypted-class-name="text-green-500/30" />
          </h2>
          
          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-6 max-w-6xl mx-auto">
            <ElectricBorder v-for="f in features" :key="f.title" 
                           color="#27FF64" :thickness="1" :speed="0.8" :chaos="0.5"
                           :style="{ borderRadius: '16px' }">
              <div class="p-6 bg-white/5 rounded-2xl h-full">
                <component :is="f.icon" class="w-10 h-10 text-green-400 mb-4" />
                <h3 class="text-xl font-semibold mb-2">{{ f.title }}</h3>
                <p class="text-gray-400">{{ f.desc }}</p>
              </div>
            </ElectricBorder>
          </div>
        </section>

        <!-- Download -->
        <section id="download" class="py-24 px-8">
          <h2 class="text-4xl font-bold text-center mb-4">
            <DecryptedText text="DOWNLOAD" :speed="30" animate-on="view" 
                          class-name="text-white" encrypted-class-name="text-green-500/30" />
          </h2>
          <p class="text-gray-400 text-center mb-12">Available for all platforms</p>
          
          <div class="flex flex-wrap gap-4 justify-center max-w-4xl mx-auto">
            <!-- Windows -->
            <a href="https://github.com/HautlyS/RUSSH/releases/latest" target="_blank"
               class="flex items-center gap-4 px-6 py-4 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 hover:border-green-500/50 transition group min-w-[200px]">
              <Monitor class="w-8 h-8 text-blue-400 group-hover:text-blue-300" />
              <div>
                <div class="font-semibold">Windows</div>
                <div class="text-sm text-gray-500">.msi / .exe</div>
              </div>
            </a>
            
            <!-- macOS -->
            <a href="https://github.com/HautlyS/RUSSH/releases/latest" target="_blank"
               class="flex items-center gap-4 px-6 py-4 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 hover:border-green-500/50 transition group min-w-[200px]">
              <Apple class="w-8 h-8 text-gray-300 group-hover:text-white" />
              <div>
                <div class="font-semibold">macOS</div>
                <div class="text-sm text-gray-500">.dmg (ARM/Intel)</div>
              </div>
            </a>
            
            <!-- Linux -->
            <a href="https://github.com/HautlyS/RUSSH/releases/latest" target="_blank"
               class="flex items-center gap-4 px-6 py-4 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 hover:border-green-500/50 transition group min-w-[200px]">
              <Terminal class="w-8 h-8 text-orange-400 group-hover:text-orange-300" />
              <div>
                <div class="font-semibold">Linux</div>
                <div class="text-sm text-gray-500">.deb / .AppImage</div>
              </div>
            </a>
            
            <!-- iOS -->
            <a href="https://github.com/HautlyS/RUSSH/releases/latest" target="_blank"
               class="flex items-center gap-4 px-6 py-4 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 hover:border-green-500/50 transition group min-w-[200px]">
              <Apple class="w-8 h-8 text-gray-300 group-hover:text-white" />
              <div>
                <div class="font-semibold">iOS</div>
                <div class="text-sm text-gray-500">.ipa (sideload)</div>
              </div>
            </a>
            
            <!-- Android -->
            <a href="https://github.com/HautlyS/RUSSH/releases/latest" target="_blank"
               class="flex items-center gap-4 px-6 py-4 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 hover:border-green-500/50 transition group min-w-[200px]">
              <Globe class="w-8 h-8 text-green-400 group-hover:text-green-300" />
              <div>
                <div class="font-semibold">Android</div>
                <div class="text-sm text-gray-500">.apk</div>
              </div>
            </a>
          </div>
        </section>

        <!-- Tech Stack -->
        <section class="py-24 px-8 border-t border-white/5">
          <div class="max-w-4xl mx-auto text-center">
            <h3 class="text-2xl font-bold mb-8">Built With</h3>
            <div class="flex flex-wrap gap-4 justify-center text-sm">
              <span class="px-4 py-2 bg-orange-500/20 text-orange-400 rounded-full">Rust</span>
              <span class="px-4 py-2 bg-cyan-500/20 text-cyan-400 rounded-full">Tauri 2.0</span>
              <span class="px-4 py-2 bg-green-500/20 text-green-400 rounded-full">Vue 3</span>
              <span class="px-4 py-2 bg-blue-500/20 text-blue-400 rounded-full">TypeScript</span>
              <span class="px-4 py-2 bg-purple-500/20 text-purple-400 rounded-full">Tailwind CSS</span>
              <span class="px-4 py-2 bg-yellow-500/20 text-yellow-400 rounded-full">xterm.js</span>
              <span class="px-4 py-2 bg-pink-500/20 text-pink-400 rounded-full">WebGL</span>
            </div>
          </div>
        </section>

        <!-- Footer -->
        <footer class="py-12 px-8 border-t border-white/5 text-center text-gray-500">
          <div class="flex items-center justify-center gap-6 mb-6">
            <a href="https://github.com/HautlyS/RUSSH" target="_blank" class="hover:text-white transition">
              <Github class="w-6 h-6" />
            </a>
          </div>
          <p>MIT License • Made with ⚡ by HautlyS</p>
        </footer>
      </div>
    </div>
  </ClickSpark>
</template>
