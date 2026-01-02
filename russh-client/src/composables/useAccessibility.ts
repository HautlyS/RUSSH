/**
 * Accessibility composable - provides accessibility utilities
 */

import { ref, onMounted, onUnmounted } from 'vue';

export function useAccessibility() {
  const prefersReducedMotion = ref(false);
  const isScreenReaderActive = ref(false);
  const focusVisible = ref(false);

  // Check for reduced motion preference
  function checkReducedMotion() {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    prefersReducedMotion.value = mediaQuery.matches;
    
    mediaQuery.addEventListener('change', (e) => {
      prefersReducedMotion.value = e.matches;
    });
  }

  // Announce message to screen readers
  function announce(message: string, priority: 'polite' | 'assertive' = 'polite') {
    const announcer = document.getElementById('sr-announcer') || createAnnouncer();
    announcer.setAttribute('aria-live', priority);
    announcer.textContent = message;
    
    // Clear after announcement
    setTimeout(() => {
      announcer.textContent = '';
    }, 1000);
  }

  // Create screen reader announcer element
  function createAnnouncer(): HTMLElement {
    const announcer = document.createElement('div');
    announcer.id = 'sr-announcer';
    announcer.setAttribute('aria-live', 'polite');
    announcer.setAttribute('aria-atomic', 'true');
    announcer.className = 'sr-only';
    announcer.style.cssText = `
      position: absolute;
      width: 1px;
      height: 1px;
      padding: 0;
      margin: -1px;
      overflow: hidden;
      clip: rect(0, 0, 0, 0);
      white-space: nowrap;
      border: 0;
    `;
    document.body.appendChild(announcer);
    return announcer;
  }

  // Focus management
  function focusFirst(container: HTMLElement) {
    const focusable = getFocusableElements(container);
    if (focusable.length > 0) {
      (focusable[0] as HTMLElement).focus();
    }
  }

  function focusLast(container: HTMLElement) {
    const focusable = getFocusableElements(container);
    if (focusable.length > 0) {
      (focusable[focusable.length - 1] as HTMLElement).focus();
    }
  }

  function getFocusableElements(container: HTMLElement): NodeListOf<Element> {
    return container.querySelectorAll(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
    );
  }

  // Trap focus within container (for modals)
  function trapFocus(container: HTMLElement) {
    const focusable = getFocusableElements(container);
    if (focusable.length === 0) return;

    const firstFocusable = focusable[0] as HTMLElement;
    const lastFocusable = focusable[focusable.length - 1] as HTMLElement;

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key !== 'Tab') return;

      if (e.shiftKey) {
        if (document.activeElement === firstFocusable) {
          e.preventDefault();
          lastFocusable.focus();
        }
      } else {
        if (document.activeElement === lastFocusable) {
          e.preventDefault();
          firstFocusable.focus();
        }
      }
    }

    container.addEventListener('keydown', handleKeyDown);
    return () => container.removeEventListener('keydown', handleKeyDown);
  }

  // Skip link functionality
  function skipToMain() {
    const main = document.querySelector('main') || document.querySelector('[role="main"]');
    if (main) {
      (main as HTMLElement).focus();
      main.scrollIntoView();
    }
  }

  // Generate unique IDs for ARIA relationships
  let idCounter = 0;
  function generateId(prefix: string = 'a11y'): string {
    return `${prefix}-${++idCounter}`;
  }

  // Check contrast ratio
  function getContrastRatio(color1: string, color2: string): number {
    const lum1 = getLuminance(color1);
    const lum2 = getLuminance(color2);
    const lighter = Math.max(lum1, lum2);
    const darker = Math.min(lum1, lum2);
    return (lighter + 0.05) / (darker + 0.05);
  }

  function getLuminance(color: string): number {
    const rgb = hexToRgb(color);
    if (!rgb) return 0;
    
    const [r, g, b] = [rgb.r, rgb.g, rgb.b].map(c => {
      c = c / 255;
      return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
    });
    
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  }

  function hexToRgb(hex: string): { r: number; g: number; b: number } | null {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16)
    } : null;
  }

  // Check if contrast meets WCAG AA
  function meetsContrastAA(foreground: string, background: string, isLargeText: boolean = false): boolean {
    const ratio = getContrastRatio(foreground, background);
    return isLargeText ? ratio >= 3 : ratio >= 4.5;
  }

  // Detect keyboard navigation
  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      focusVisible.value = true;
    }
  }

  function handleMouseDown() {
    focusVisible.value = false;
  }

  onMounted(() => {
    checkReducedMotion();
    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('mousedown', handleMouseDown);
  });

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown);
    document.removeEventListener('mousedown', handleMouseDown);
  });

  return {
    prefersReducedMotion,
    isScreenReaderActive,
    focusVisible,
    announce,
    focusFirst,
    focusLast,
    getFocusableElements,
    trapFocus,
    skipToMain,
    generateId,
    getContrastRatio,
    meetsContrastAA,
  };
}
