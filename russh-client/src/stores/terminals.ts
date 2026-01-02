/**
 * Terminal store - manages terminal tabs and state
 */

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Terminal } from 'xterm';

export interface TerminalTab {
  id: string;
  sessionId: string;
  profileId: string;
  title: string;
  isActive: boolean;
  terminal?: Terminal;
}

export const useTerminalStore = defineStore('terminals', () => {
  // State
  const tabs = ref<TerminalTab[]>([]);
  const activeTabId = ref<string | null>(null);

  // Getters
  const activeTab = computed(() => 
    tabs.value.find(t => t.id === activeTabId.value)
  );

  const tabsBySession = computed(() => {
    const map = new Map<string, TerminalTab[]>();
    tabs.value.forEach(tab => {
      const existing = map.get(tab.sessionId) || [];
      existing.push(tab);
      map.set(tab.sessionId, existing);
    });
    return map;
  });

  // Actions
  function createTab(sessionId: string, profileId: string, title: string): string {
    const id = crypto.randomUUID();
    
    // Deactivate other tabs
    tabs.value.forEach(t => t.isActive = false);
    
    tabs.value.push({
      id,
      sessionId,
      profileId,
      title,
      isActive: true,
    });
    
    activeTabId.value = id;
    return id;
  }

  function closeTab(tabId: string) {
    const index = tabs.value.findIndex(t => t.id === tabId);
    if (index === -1) return;

    // Dispose terminal if exists
    const tab = tabs.value[index];
    tab.terminal?.dispose();

    tabs.value.splice(index, 1);
    
    // Update active tab if needed
    if (activeTabId.value === tabId) {
      const newIndex = Math.max(0, index - 1);
      activeTabId.value = tabs.value[newIndex]?.id || null;
      if (tabs.value[newIndex]) {
        tabs.value[newIndex].isActive = true;
      }
    }
  }

  function closeTabsBySession(sessionId: string) {
    const sessionTabs = tabs.value.filter(t => t.sessionId === sessionId);
    sessionTabs.forEach(tab => closeTab(tab.id));
  }

  function setActiveTab(tabId: string) {
    tabs.value.forEach(t => t.isActive = t.id === tabId);
    activeTabId.value = tabId;
  }

  function updateTabTitle(tabId: string, title: string) {
    const tab = tabs.value.find(t => t.id === tabId);
    if (tab) tab.title = title;
  }

  function setTerminalInstance(tabId: string, terminal: Terminal) {
    const tab = tabs.value.find(t => t.id === tabId);
    if (tab) tab.terminal = terminal;
  }

  function getTerminalInstance(tabId: string): Terminal | undefined {
    const tab = tabs.value.find(t => t.id === tabId);
    return tab?.terminal;
  }

  function nextTab() {
    if (tabs.value.length <= 1) return;
    
    const currentIndex = tabs.value.findIndex(t => t.id === activeTabId.value);
    const nextIndex = (currentIndex + 1) % tabs.value.length;
    setActiveTab(tabs.value[nextIndex].id);
  }

  function previousTab() {
    if (tabs.value.length <= 1) return;
    
    const currentIndex = tabs.value.findIndex(t => t.id === activeTabId.value);
    const prevIndex = (currentIndex - 1 + tabs.value.length) % tabs.value.length;
    setActiveTab(tabs.value[prevIndex].id);
  }

  return {
    // State
    tabs,
    activeTabId,
    // Getters
    activeTab,
    tabsBySession,
    // Actions
    createTab,
    closeTab,
    closeTabsBySession,
    setActiveTab,
    updateTabTitle,
    setTerminalInstance,
    getTerminalInstance,
    nextTab,
    previousTab,
  };
});
