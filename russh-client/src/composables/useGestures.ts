/**
 * Gesture composable - handles touch gestures for mobile
 */

import { ref, onMounted, onUnmounted, type Ref } from 'vue';
import { usePlatform } from './usePlatform';

export interface GestureOptions {
  onSwipeLeft?: () => void;
  onSwipeRight?: () => void;
  onSwipeUp?: () => void;
  onSwipeDown?: () => void;
  onPinch?: (scale: number) => void;
  onPullToRefresh?: () => Promise<void>;
  swipeThreshold?: number;
  pullThreshold?: number;
}

export function useGestures(elementRef: Ref<HTMLElement | undefined>, options: GestureOptions = {}) {
  const { isMobile, hapticFeedback } = usePlatform();
  
  const {
    swipeThreshold = 50,
    pullThreshold = 80,
  } = options;

  // Touch state
  const touchStartX = ref(0);
  const touchStartY = ref(0);
  const touchCurrentX = ref(0);
  const touchCurrentY = ref(0);
  const isSwiping = ref(false);
  const isPulling = ref(false);
  const pullDistance = ref(0);
  
  // Pinch state
  const initialPinchDistance = ref(0);
  const currentScale = ref(1);

  function getTouchDistance(touches: TouchList): number {
    if (touches.length < 2) return 0;
    const dx = touches[0].clientX - touches[1].clientX;
    const dy = touches[0].clientY - touches[1].clientY;
    return Math.sqrt(dx * dx + dy * dy);
  }

  function onTouchStart(e: TouchEvent) {
    if (e.touches.length === 1) {
      touchStartX.value = e.touches[0].clientX;
      touchStartY.value = e.touches[0].clientY;
      isSwiping.value = false;
      
      // Check if at top for pull-to-refresh
      const el = elementRef.value;
      if (el && el.scrollTop === 0 && options.onPullToRefresh) {
        isPulling.value = true;
      }
    } else if (e.touches.length === 2 && options.onPinch) {
      initialPinchDistance.value = getTouchDistance(e.touches);
    }
  }

  function onTouchMove(e: TouchEvent) {
    if (e.touches.length === 1) {
      touchCurrentX.value = e.touches[0].clientX;
      touchCurrentY.value = e.touches[0].clientY;
      
      const dx = touchCurrentX.value - touchStartX.value;
      const dy = touchCurrentY.value - touchStartY.value;
      
      // Pull to refresh
      if (isPulling.value && dy > 0) {
        pullDistance.value = Math.min(dy, pullThreshold * 1.5);
        if (pullDistance.value > pullThreshold) {
          hapticFeedback('light');
        }
        e.preventDefault();
      }
      
      // Detect swipe
      if (Math.abs(dx) > swipeThreshold || Math.abs(dy) > swipeThreshold) {
        isSwiping.value = true;
      }
    } else if (e.touches.length === 2 && options.onPinch && initialPinchDistance.value > 0) {
      const currentDistance = getTouchDistance(e.touches);
      currentScale.value = currentDistance / initialPinchDistance.value;
      options.onPinch(currentScale.value);
    }
  }

  async function onTouchEnd() {
    // Handle pull to refresh
    if (isPulling.value && pullDistance.value > pullThreshold && options.onPullToRefresh) {
      hapticFeedback('medium');
      await options.onPullToRefresh();
    }
    
    // Handle swipe
    if (isSwiping.value) {
      const dx = touchCurrentX.value - touchStartX.value;
      const dy = touchCurrentY.value - touchStartY.value;
      
      if (Math.abs(dx) > Math.abs(dy)) {
        // Horizontal swipe
        if (dx > swipeThreshold && options.onSwipeRight) {
          hapticFeedback('light');
          options.onSwipeRight();
        } else if (dx < -swipeThreshold && options.onSwipeLeft) {
          hapticFeedback('light');
          options.onSwipeLeft();
        }
      } else {
        // Vertical swipe
        if (dy > swipeThreshold && options.onSwipeDown) {
          hapticFeedback('light');
          options.onSwipeDown();
        } else if (dy < -swipeThreshold && options.onSwipeUp) {
          hapticFeedback('light');
          options.onSwipeUp();
        }
      }
    }
    
    // Reset state
    isSwiping.value = false;
    isPulling.value = false;
    pullDistance.value = 0;
    initialPinchDistance.value = 0;
    currentScale.value = 1;
  }

  onMounted(() => {
    const el = elementRef.value;
    if (!el || !isMobile.value) return;
    
    el.addEventListener('touchstart', onTouchStart, { passive: false });
    el.addEventListener('touchmove', onTouchMove, { passive: false });
    el.addEventListener('touchend', onTouchEnd);
  });

  onUnmounted(() => {
    const el = elementRef.value;
    if (!el) return;
    
    el.removeEventListener('touchstart', onTouchStart);
    el.removeEventListener('touchmove', onTouchMove);
    el.removeEventListener('touchend', onTouchEnd);
  });

  return {
    isSwiping,
    isPulling,
    pullDistance,
    currentScale,
  };
}
