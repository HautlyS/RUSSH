import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import ConnectionCard from '@/components/connections/ConnectionCard.vue';
import type { ConnectionProfile } from '@/types/ssh';

// Mock composables
vi.mock('@/composables/useVisualEffects', () => ({
  useVisualEffects: () => ({
    isMagnetEnabled: { value: false },
    isDecryptedTextEnabled: { value: false },
    visualEffects: { magnet: {}, decryptedText: { speed: 50 } },
  }),
}));

vi.mock('@/composables/usePlatform', () => ({
  usePlatform: () => ({
    isMobile: { value: false },
    isTouchDevice: () => false,
  }),
}));

describe('ConnectionCard', () => {
  const mockProfile: ConnectionProfile = {
    id: 'test-1',
    name: 'Test Server',
    host: 'example.com',
    port: 22,
    username: 'testuser',
    authType: 'password',
    tags: ['production', 'web'],
    autoReconnect: false,
    useCount: 5,
  };

  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('renders profile information', () => {
    const wrapper = mount(ConnectionCard, {
      props: { profile: mockProfile },
    });

    expect(wrapper.text()).toContain('Test Server');
    expect(wrapper.text()).toContain('testuser@example.com:22');
  });

  it('renders tags', () => {
    const wrapper = mount(ConnectionCard, {
      props: { profile: mockProfile },
    });

    expect(wrapper.text()).toContain('production');
    expect(wrapper.text()).toContain('web');
  });

  it('emits connect event when connect button clicked', async () => {
    const wrapper = mount(ConnectionCard, {
      props: { profile: mockProfile },
    });

    await wrapper.find('button').trigger('click');
    expect(wrapper.emitted('connect')).toBeTruthy();
  });

  it('emits edit event when edit button clicked', async () => {
    const wrapper = mount(ConnectionCard, {
      props: { profile: mockProfile },
    });

    const editButton = wrapper.findAll('button').find(b => 
      b.attributes('aria-label')?.includes('Edit')
    );
    await editButton?.trigger('click');
    expect(wrapper.emitted('edit')).toBeTruthy();
  });

  it('emits delete event when delete button clicked', async () => {
    const wrapper = mount(ConnectionCard, {
      props: { profile: mockProfile },
    });

    const deleteButton = wrapper.findAll('button').find(b => 
      b.attributes('aria-label')?.includes('Delete')
    );
    await deleteButton?.trigger('click');
    expect(wrapper.emitted('delete')).toBeTruthy();
  });

  it('has proper ARIA labels', () => {
    const wrapper = mount(ConnectionCard, {
      props: { profile: mockProfile },
    });

    const article = wrapper.find('article');
    expect(article.attributes('aria-label')).toContain('Test Server');
    expect(article.attributes('role')).toBe('article');
  });

  it('has accessible buttons with aria-labels', () => {
    const wrapper = mount(ConnectionCard, {
      props: { profile: mockProfile },
    });

    const buttons = wrapper.findAll('button');
    buttons.forEach(button => {
      expect(button.attributes('aria-label')).toBeTruthy();
    });
  });
});
