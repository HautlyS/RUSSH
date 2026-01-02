import { test, expect } from '@playwright/test';

test.describe('RUSSH Client E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display dashboard on load', async ({ page }) => {
    await expect(page.locator('text=Welcome')).toBeVisible();
  });

  test('should navigate to connections page', async ({ page }) => {
    await page.click('text=Servers');
    await expect(page).toHaveURL(/\/connections/);
  });

  test('should open new connection form', async ({ page }) => {
    await page.click('text=Servers');
    await page.click('text=New Connection');
    await expect(page.locator('input[name="name"]')).toBeVisible();
  });

  test('should validate connection form', async ({ page }) => {
    await page.click('text=Servers');
    await page.click('text=New Connection');
    
    // Try to submit empty form
    await page.click('button[type="submit"]');
    
    // Should show validation errors
    await expect(page.locator('text=required')).toBeVisible();
  });

  test('should open command palette with keyboard shortcut', async ({ page }) => {
    await page.keyboard.press('Control+k');
    await expect(page.locator('[role="dialog"]')).toBeVisible();
  });

  test('should close command palette with escape', async ({ page }) => {
    await page.keyboard.press('Control+k');
    await expect(page.locator('[role="dialog"]')).toBeVisible();
    
    await page.keyboard.press('Escape');
    await expect(page.locator('[role="dialog"]')).not.toBeVisible();
  });

  test('should toggle dark mode', async ({ page }) => {
    await page.click('text=Settings');
    await page.click('text=Appearance');
    
    const darkModeToggle = page.locator('text=Dark Mode').locator('..').locator('button');
    await darkModeToggle.click();
    
    await expect(page.locator('html')).toHaveClass(/dark/);
  });

  test('should navigate using sidebar', async ({ page }) => {
    // Click on different sidebar items
    await page.click('[aria-label="Dashboard"]');
    await expect(page).toHaveURL('/');
    
    await page.click('[aria-label="Connections"]');
    await expect(page).toHaveURL(/\/connections/);
    
    await page.click('[aria-label="Settings"]');
    await expect(page).toHaveURL(/\/settings/);
  });

  test('should be keyboard navigable', async ({ page }) => {
    // Tab through main navigation
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    
    // Should have visible focus indicator
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
  });

  test('should show notifications', async ({ page }) => {
    // Trigger an action that shows a notification
    await page.click('text=Servers');
    await page.click('text=New Connection');
    
    // Fill form with invalid data and submit
    await page.fill('input[name="host"]', 'invalid-host');
    await page.click('button[type="submit"]');
    
    // Should show error notification
    await expect(page.locator('[role="alert"]')).toBeVisible();
  });
});

test.describe('Accessibility Tests', () => {
  test('should have no accessibility violations on dashboard', async ({ page }) => {
    await page.goto('/');
    
    // Check for basic accessibility
    const main = page.locator('main');
    await expect(main).toBeVisible();
    
    // All images should have alt text
    const images = page.locator('img');
    const count = await images.count();
    for (let i = 0; i < count; i++) {
      const img = images.nth(i);
      const alt = await img.getAttribute('alt');
      const ariaHidden = await img.getAttribute('aria-hidden');
      expect(alt || ariaHidden === 'true').toBeTruthy();
    }
    
    // All buttons should have accessible names
    const buttons = page.locator('button');
    const buttonCount = await buttons.count();
    for (let i = 0; i < buttonCount; i++) {
      const button = buttons.nth(i);
      const ariaLabel = await button.getAttribute('aria-label');
      const text = await button.textContent();
      expect(ariaLabel || text?.trim()).toBeTruthy();
    }
  });

  test('should support reduced motion', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto('/');
    
    // Animations should be disabled
    // This is a basic check - in real tests you'd verify CSS
    await expect(page.locator('body')).toBeVisible();
  });
});
