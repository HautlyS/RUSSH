import { defineConfig, mergeConfig } from 'vite';
import baseConfig from './vite.config';
import { resolve } from 'path';

export default mergeConfig(
  baseConfig,
  defineConfig({
    resolve: {
      alias: {
        '@tauri-apps/api/core': resolve(__dirname, 'src/mocks/tauri-api.ts'),
        '@tauri-apps/api/event': resolve(__dirname, 'src/mocks/tauri-api.ts'),
        '@tauri-apps/api/window': resolve(__dirname, 'src/mocks/tauri-api.ts'),
        '@tauri-apps/api': resolve(__dirname, 'src/mocks/tauri-api.ts'),
        '@tauri-apps/plugin-os': resolve(__dirname, 'src/mocks/tauri-plugins.ts'),
        '@tauri-apps/plugin-biometric': resolve(__dirname, 'src/mocks/tauri-plugins.ts'),
        '@tauri-apps/plugin-dialog': resolve(__dirname, 'src/mocks/tauri-plugins.ts'),
        '@tauri-apps/plugin-haptics': resolve(__dirname, 'src/mocks/tauri-plugins.ts'),
        '@tauri-apps/plugin-notification': resolve(__dirname, 'src/mocks/tauri-plugins.ts'),
      },
    },
    define: {
      'import.meta.env.VITE_MOCK_MODE': JSON.stringify(true),
    },
  })
);
