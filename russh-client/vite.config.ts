import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import { resolve } from 'path';

const isTauri = process.env.TAURI_PLATFORM !== undefined;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'es2020',
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      // Externalize Tauri plugins for web builds
      external: isTauri ? [] : [
        '@tauri-apps/api',
        '@tauri-apps/api/core',
        '@tauri-apps/plugin-os',
        '@tauri-apps/plugin-biometric',
        '@tauri-apps/plugin-dialog',
        '@tauri-apps/plugin-haptics',
        '@tauri-apps/plugin-notification',
      ],
      output: {
        manualChunks: {
          'vue-vendor': ['vue', 'vue-router', 'pinia'],
          'xterm-vendor': ['xterm', 'xterm-addon-fit', 'xterm-addon-webgl', 'xterm-addon-search'],
          'ui-vendor': ['lucide-vue-next'],
          'animation-vendor': ['gsap', 'motion-v'],
        },
      },
    },
    chunkSizeWarningLimit: 600,
  },
  optimizeDeps: {
    include: ['vue', 'vue-router', 'pinia', 'xterm'],
  },
});
