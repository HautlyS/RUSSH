<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import DecryptedText from '@/components/extra/DecryptedText.vue';
import RotatingText from '@/components/extra/RotatingText.vue';
import { 
  Terminal, Shield, Zap, Globe, Download, Github, Apple, Monitor,
  Code, Cpu, Layers, ArrowRight, GitBranch
} from 'lucide-vue-next';

const features = [
  { icon: Terminal, title: 'SSH Terminal', desc: 'Full xterm.js with WebGL rendering for blazing fast performance' },
  { icon: Shield, title: 'Secure by Default', desc: 'Key-based authentication & end-to-end encrypted sessions' },
  { icon: Zap, title: 'P2P Networking', desc: 'Direct peer connections with automatic NAT traversal' },
  { icon: Globe, title: 'Cross-Platform', desc: 'Native apps for Windows, macOS, Linux, iOS & Android' },
  { icon: Code, title: 'Interactive Blocks', desc: 'Share code snippets, files, and widgets with peers' },
  { icon: Cpu, title: 'Rust Powered', desc: 'Built with Rust for memory safety and performance' },
];

const stats = [
  { value: '10x', label: 'Faster than Electron' },
  { value: '<50MB', label: 'App Size' },
  { value: '100%', label: 'Open Source' },
];

const rotatingWords = ['SECURE', 'FAST', 'MODERN', 'POWERFUL'];
const isVisible = ref(false);
const scrollY = ref(0);

function handleScroll() {
  scrollY.value = window.scrollY;
}

onMounted(() => {
  isVisible.value = true;
  window.addEventListener('scroll', handleScroll, { passive: true });
});

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll);
});
</script>

<template>
  <div class="min-h-screen bg-black text-white">
    <!-- Gradient Background (CSS only - no WebGL) -->
    <div class="fixed inset-0 overflow-hidden pointer-events-none">
      <div class="absolute inset-0 bg-gradient-to-br from-black via-gray-950 to-black" />
      <div 
        class="absolute top-0 left-1/4 w-[600px] h-[600px] bg-green-500/10 rounded-full blur-[120px]"
        :style="{ transform: `translateY(${scrollY * 0.1}px)` }"
      />
      <div 
        class="absolute bottom-0 right-1/4 w-[500px] h-[500px] bg-emerald-500/8 rounded-full blur-[100px]"
        :style="{ transform: `translateY(${-scrollY * 0.05}px)` }"
      />
      <!-- Grid pattern -->
      <div 
        class="absolute inset-0 opacity-[0.03]"
        style="background-image: linear-gradient(rgba(255,255,255,.1) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,.1) 1px, transparent 1px); background-size: 60px 60px;"
      />
    </div>

    <!-- Navigation - Fixed with proper z-index -->
    <nav class="fixed top-0 left-0 right-0 z-50 backdrop-blur-md bg-black/50 border-b border-white/5">
      <div class="max-w-7xl mx-auto flex items-center justify-between px-6 py-4">
        <div class="flex items-center gap-3">
          <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-green-400 to-emerald-600 flex items-center justify-center shadow-lg shadow-green-500/20">
            <Terminal class="w-5 h-5 text-black" />
          </div>
          <span class="text-xl font-bold tracking-tight">RUSSH</span>
        </div>
        
        <div class="hidden md:flex items-center gap-8">
          <a href="#features" class="text-gray-400 hover:text-white transition-colors text-sm">Features</a>
          <a href="#tech" class="text-gray-400 hover:text-white transition-colors text-sm">Tech Stack</a>
          <a href="#download" class="text-gray-400 hover:text-white transition-colors text-sm">Download</a>
        </div>
        
        <div class="flex items-center gap-3">
          <a 
            href="https://github.com/HautlyS/RUSSH" 
            target="_blank"
            class="flex items-center gap-2 px-4 py-2 text-sm bg-white/5 hover:bg-white/10 border border-white/10 rounded-lg transition-colors"
          >
            <Github class="w-4 h-4" />
            <span class="hidden sm:inline">GitHub</span>
          </a>
          <a 
            href="#download"
            class="flex items-center gap-2 px-4 py-2 text-sm bg-green-500 hover:bg-green-400 text-black font-medium rounded-lg transition-colors"
          >
            <Download class="w-4 h-4" />
            <span class="hidden sm:inline">Download</span>
          </a>
        </div>
      </div>
    </nav>

    <!-- Hero Section -->
    <section class="relative min-h-screen flex flex-col items-center justify-center px-6 pt-20">
      <div 
        class="text-center transition-all duration-1000"
        :class="isVisible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-8'"
      >
        <!-- Badge -->
        <div class="inline-flex items-center gap-2 px-4 py-2 mb-8 text-sm bg-green-500/10 border border-green-500/20 rounded-full text-green-400">
          <Zap class="w-4 h-4" />
          Built with Rust + Tauri 2.0
        </div>

        <!-- Main Title -->
        <h1 class="mb-6">
          <DecryptedText 
            text="RUSSH" 
            :speed="50" 
            :max-iterations="12"
            animate-on="view"
            class-name="text-7xl sm:text-8xl md:text-9xl font-black tracking-tighter bg-gradient-to-r from-green-400 via-emerald-300 to-cyan-400 bg-clip-text text-transparent"
            encrypted-class-name="text-green-500/40"
          />
        </h1>
        
        <!-- Subtitle with rotating text -->
        <p class="text-xl sm:text-2xl md:text-3xl text-gray-400 mb-6 flex flex-wrap items-center justify-center gap-2">
          <span>The</span>
          <RotatingText 
            :texts="rotatingWords" 
            :rotation-interval="2500"
            :stagger-duration="0.025"
            main-class-name="text-green-400 font-bold inline-block min-w-[140px]"
            split-by="characters"
          />
          <span>SSH Client</span>
        </p>
        
        <p class="text-base sm:text-lg text-gray-500 max-w-2xl mx-auto mb-10 leading-relaxed">
          A modern, cross-platform SSH client with P2P networking, 
          interactive terminal blocks, and stunning visual effects.
        </p>

        <!-- CTA Buttons -->
        <div class="flex flex-col sm:flex-row gap-4 justify-center">
          <a 
            href="#download"
            class="group flex items-center justify-center gap-3 px-8 py-4 bg-gradient-to-r from-green-500 to-emerald-500 hover:from-green-400 hover:to-emerald-400 text-black font-semibold rounded-xl transition-all shadow-lg shadow-green-500/25 hover:shadow-green-500/40"
          >
            <Download class="w-5 h-5" />
            Download Free
            <ArrowRight class="w-4 h-4 group-hover:translate-x-1 transition-transform" />
          </a>
          
          <a 
            href="https://github.com/HautlyS/RUSSH" 
            target="_blank"
            class="flex items-center justify-center gap-3 px-8 py-4 bg-white/5 hover:bg-white/10 border border-white/10 hover:border-white/20 rounded-xl transition-all"
          >
            <Github class="w-5 h-5" />
            View Source
          </a>
        </div>

        <!-- Stats -->
        <div class="flex flex-wrap justify-center gap-8 sm:gap-16 mt-16 pt-8 border-t border-white/5">
          <div v-for="stat in stats" :key="stat.label" class="text-center">
            <div class="text-3xl sm:text-4xl font-bold text-green-400">{{ stat.value }}</div>
            <div class="text-sm text-gray-500 mt-1">{{ stat.label }}</div>
          </div>
        </div>
      </div>

      <!-- Scroll indicator -->
      <div class="absolute bottom-8 left-1/2 -translate-x-1/2 animate-bounce">
        <div class="w-6 h-10 border-2 border-white/20 rounded-full flex justify-center pt-2">
          <div class="w-1 h-2 bg-white/40 rounded-full" />
        </div>
      </div>
    </section>

    <!-- Features Section -->
    <section id="features" class="relative py-24 sm:py-32 px-6">
      <div class="max-w-6xl mx-auto">
        <div class="text-center mb-16">
          <h2 class="text-3xl sm:text-4xl font-bold mb-4">
            <DecryptedText 
              text="POWERFUL FEATURES" 
              :speed="30" 
              animate-on="view"
              class-name="text-white"
              encrypted-class-name="text-green-500/30"
            />
          </h2>
          <p class="text-gray-400 max-w-xl mx-auto">
            Everything you need for secure, efficient remote server management
          </p>
        </div>
        
        <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
          <div 
            v-for="(feature, i) in features" 
            :key="feature.title"
            class="group p-6 bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 hover:border-green-500/30 rounded-2xl transition-all duration-300"
            :style="{ transitionDelay: `${i * 50}ms` }"
          >
            <div class="w-12 h-12 mb-4 rounded-xl bg-green-500/10 flex items-center justify-center group-hover:bg-green-500/20 transition-colors">
              <component :is="feature.icon" class="w-6 h-6 text-green-400" />
            </div>
            <h3 class="text-lg font-semibold mb-2 text-white">{{ feature.title }}</h3>
            <p class="text-gray-400 text-sm leading-relaxed">{{ feature.desc }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- Terminal Preview Section -->
    <section class="relative py-24 px-6 overflow-hidden">
      <div class="max-w-5xl mx-auto">
        <div class="relative rounded-2xl overflow-hidden border border-white/10 shadow-2xl shadow-green-500/5">
          <!-- Terminal Header -->
          <div class="flex items-center gap-2 px-4 py-3 bg-gray-900 border-b border-white/5">
            <div class="flex gap-2">
              <div class="w-3 h-3 rounded-full bg-red-500/80" />
              <div class="w-3 h-3 rounded-full bg-yellow-500/80" />
              <div class="w-3 h-3 rounded-full bg-green-500/80" />
            </div>
            <span class="ml-4 text-sm text-gray-500 font-mono">russh — production-server</span>
          </div>
          
          <!-- Terminal Content -->
          <div class="p-6 bg-gray-950 font-mono text-sm leading-relaxed">
            <div class="text-green-400">admin@production-server:~$</div>
            <div class="text-gray-300 mt-1">russh connect production --key ~/.ssh/id_rsa</div>
            <div class="text-gray-500 mt-3">Connecting to 192.168.1.100:22...</div>
            <div class="text-green-400 mt-1">✓ Connected via P2P relay (latency: 12ms)</div>
            <div class="text-gray-500 mt-1">✓ Session encrypted with AES-256-GCM</div>
            <div class="mt-4 text-green-400">admin@production-server:~$</div>
            <div class="text-gray-300 mt-1">docker ps --format "table &#123;&#123;.Names&#125;&#125;\t&#123;&#123;.Status&#125;&#125;"</div>
            <div class="text-gray-400 mt-2">
              <div>NAMES              STATUS</div>
              <div>nginx-proxy        Up 3 days</div>
              <div>api-server         Up 3 days</div>
              <div>postgres-db        Up 5 days</div>
            </div>
            <div class="mt-4 flex items-center">
              <span class="text-green-400">admin@production-server:~$</span>
              <span class="ml-2 w-2 h-5 bg-green-400 animate-pulse" />
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Tech Stack Section -->
    <section id="tech" class="relative py-24 sm:py-32 px-6 border-t border-white/5">
      <div class="max-w-4xl mx-auto text-center">
        <h2 class="text-3xl sm:text-4xl font-bold mb-4">Built With Modern Tech</h2>
        <p class="text-gray-400 mb-12">Leveraging the best tools for performance and developer experience</p>
        
        <div class="flex flex-wrap justify-center gap-3">
          <span class="px-5 py-2.5 bg-orange-500/10 text-orange-400 border border-orange-500/20 rounded-full text-sm font-medium">Rust</span>
          <span class="px-5 py-2.5 bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 rounded-full text-sm font-medium">Tauri 2.0</span>
          <span class="px-5 py-2.5 bg-green-500/10 text-green-400 border border-green-500/20 rounded-full text-sm font-medium">Vue 3</span>
          <span class="px-5 py-2.5 bg-blue-500/10 text-blue-400 border border-blue-500/20 rounded-full text-sm font-medium">TypeScript</span>
          <span class="px-5 py-2.5 bg-sky-500/10 text-sky-400 border border-sky-500/20 rounded-full text-sm font-medium">Tailwind CSS</span>
          <span class="px-5 py-2.5 bg-yellow-500/10 text-yellow-400 border border-yellow-500/20 rounded-full text-sm font-medium">xterm.js</span>
          <span class="px-5 py-2.5 bg-purple-500/10 text-purple-400 border border-purple-500/20 rounded-full text-sm font-medium">WebGL</span>
          <span class="px-5 py-2.5 bg-pink-500/10 text-pink-400 border border-pink-500/20 rounded-full text-sm font-medium">Tokio</span>
        </div>
      </div>
    </section>

    <!-- Download Section -->
    <section id="download" class="relative py-24 sm:py-32 px-6">
      <div class="max-w-4xl mx-auto">
        <div class="text-center mb-12">
          <h2 class="text-3xl sm:text-4xl font-bold mb-4">
            <DecryptedText 
              text="DOWNLOAD NOW" 
              :speed="30" 
              animate-on="view"
              class-name="text-white"
              encrypted-class-name="text-green-500/30"
            />
          </h2>
          <p class="text-gray-400">Free and open source. Available for all platforms.</p>
        </div>
        
        <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-4">
          <!-- Windows -->
          <a 
            href="https://github.com/HautlyS/RUSSH/releases/latest" 
            target="_blank"
            class="group flex items-center gap-4 p-5 bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 hover:border-blue-500/30 rounded-xl transition-all"
          >
            <div class="w-12 h-12 rounded-xl bg-blue-500/10 flex items-center justify-center group-hover:bg-blue-500/20 transition-colors">
              <Monitor class="w-6 h-6 text-blue-400" />
            </div>
            <div>
              <div class="font-semibold text-white">Windows</div>
              <div class="text-sm text-gray-500">.msi / .exe</div>
            </div>
          </a>
          
          <!-- macOS -->
          <a 
            href="https://github.com/HautlyS/RUSSH/releases/latest" 
            target="_blank"
            class="group flex items-center gap-4 p-5 bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 hover:border-gray-400/30 rounded-xl transition-all"
          >
            <div class="w-12 h-12 rounded-xl bg-gray-500/10 flex items-center justify-center group-hover:bg-gray-500/20 transition-colors">
              <Apple class="w-6 h-6 text-gray-300" />
            </div>
            <div>
              <div class="font-semibold text-white">macOS</div>
              <div class="text-sm text-gray-500">.dmg (Universal)</div>
            </div>
          </a>
          
          <!-- Linux -->
          <a 
            href="https://github.com/HautlyS/RUSSH/releases/latest" 
            target="_blank"
            class="group flex items-center gap-4 p-5 bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 hover:border-orange-500/30 rounded-xl transition-all"
          >
            <div class="w-12 h-12 rounded-xl bg-orange-500/10 flex items-center justify-center group-hover:bg-orange-500/20 transition-colors">
              <Terminal class="w-6 h-6 text-orange-400" />
            </div>
            <div>
              <div class="font-semibold text-white">Linux</div>
              <div class="text-sm text-gray-500">.deb / .AppImage</div>
            </div>
          </a>
          
          <!-- iOS -->
          <a 
            href="https://github.com/HautlyS/RUSSH/releases/latest" 
            target="_blank"
            class="group flex items-center gap-4 p-5 bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 hover:border-gray-400/30 rounded-xl transition-all"
          >
            <div class="w-12 h-12 rounded-xl bg-gray-500/10 flex items-center justify-center group-hover:bg-gray-500/20 transition-colors">
              <Apple class="w-6 h-6 text-gray-300" />
            </div>
            <div>
              <div class="font-semibold text-white">iOS</div>
              <div class="text-sm text-gray-500">.ipa (TestFlight)</div>
            </div>
          </a>
          
          <!-- Android -->
          <a 
            href="https://github.com/HautlyS/RUSSH/releases/latest" 
            target="_blank"
            class="group flex items-center gap-4 p-5 bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 hover:border-green-500/30 rounded-xl transition-all"
          >
            <div class="w-12 h-12 rounded-xl bg-green-500/10 flex items-center justify-center group-hover:bg-green-500/20 transition-colors">
              <Layers class="w-6 h-6 text-green-400" />
            </div>
            <div>
              <div class="font-semibold text-white">Android</div>
              <div class="text-sm text-gray-500">.apk</div>
            </div>
          </a>
          
          <!-- Source -->
          <a 
            href="https://github.com/HautlyS/RUSSH" 
            target="_blank"
            class="group flex items-center gap-4 p-5 bg-white/[0.02] hover:bg-white/[0.05] border border-white/5 hover:border-purple-500/30 rounded-xl transition-all"
          >
            <div class="w-12 h-12 rounded-xl bg-purple-500/10 flex items-center justify-center group-hover:bg-purple-500/20 transition-colors">
              <GitBranch class="w-6 h-6 text-purple-400" />
            </div>
            <div>
              <div class="font-semibold text-white">Source Code</div>
              <div class="text-sm text-gray-500">Build from source</div>
            </div>
          </a>
        </div>
      </div>
    </section>

    <!-- Footer -->
    <footer class="relative py-12 px-6 border-t border-white/5">
      <div class="max-w-6xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-6">
        <div class="flex items-center gap-3">
          <div class="w-8 h-8 rounded-lg bg-gradient-to-br from-green-400 to-emerald-600 flex items-center justify-center">
            <Terminal class="w-4 h-4 text-black" />
          </div>
          <span class="font-semibold">RUSSH</span>
        </div>
        
        <div class="flex items-center gap-6">
          <a href="https://github.com/HautlyS/RUSSH" target="_blank" class="text-gray-400 hover:text-white transition-colors">
            <Github class="w-5 h-5" />
          </a>
        </div>
        
        <p class="text-sm text-gray-500">MIT License • Made with ⚡ by HautlyS</p>
      </div>
    </footer>
  </div>
</template>
