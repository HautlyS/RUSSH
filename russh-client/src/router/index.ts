import { createRouter, createWebHistory } from 'vue-router';
import type { RouteRecordRaw } from 'vue-router';

// Check if running in Tauri or web
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => isTauri 
      ? import('@/views/DashboardView.vue') 
      : import('@/views/LandingView.vue'),
  },
  {
    path: '/dashboard',
    name: 'dashboard',
    component: () => import('@/views/DashboardView.vue'),
  },
  {
    path: '/connections',
    name: 'connections',
    component: () => import('@/views/ConnectionsView.vue'),
  },
  {
    path: '/connections/new',
    name: 'new-connection',
    component: () => import('@/views/NewConnectionView.vue'),
  },
  {
    path: '/connections/:id/edit',
    name: 'edit-connection',
    component: () => import('@/views/EditConnectionView.vue'),
    props: true,
  },
  {
    path: '/terminal/:sessionId?',
    name: 'terminal',
    component: () => import('@/views/TerminalView.vue'),
    props: true,
  },
  {
    path: '/files/:sessionId?',
    name: 'files',
    component: () => import('@/views/FilesView.vue'),
    props: true,
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/views/SettingsView.vue'),
  },
  {
    path: '/p2p',
    name: 'p2p',
    component: () => import('@/views/P2PView.vue'),
  },
  {
    path: '/p2p/terminal',
    name: 'p2p-terminal',
    component: () => import('@/views/P2PTerminalView.vue'),
  },
  {
    path: '/streaming',
    name: 'streaming',
    component: () => import('@/views/StreamingView.vue'),
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
