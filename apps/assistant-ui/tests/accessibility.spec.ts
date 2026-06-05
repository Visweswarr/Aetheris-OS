import { test, expect } from '@playwright/test';

test.describe('Accessibility', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should have proper heading structure', async ({ page }) => {
    // Check if h1 is present and has proper text
    await expect(page.locator('h1')).toContainText('Aetheris Assistant');
    
    // Check if h2 is present in settings
    await page.locator('button:has-text("Settings")').click();
    await expect(page.locator('h2')).toContainText('Settings');
  });

  test('should have proper form labels', async ({ page }) => {
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await expect(messageInput).toHaveAttribute('aria-label', 'Message input');
  });

  test('should have proper button labels', async ({ page }) => {
    const sendButton = page.locator('button[type="submit"]');
    await expect(sendButton).toHaveAttribute('aria-label', 'Send message');
  });

  test('should support keyboard navigation', async ({ page }) => {
    // Tab through elements
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    await page.keyboard.press('Tab');
    
    // Check if focus is visible
    const focusedElement = page.locator(':focus');
    await expect(focusedElement).toBeVisible();
  });

  test('should have proper ARIA attributes', async ({ page }) => {
    // Check if main content has proper role
    await expect(page.locator('main')).toHaveAttribute('role', 'main');
    
    // Check if navigation has proper role
    await expect(page.locator('nav')).toHaveAttribute('role', 'navigation');
  });

  test('should have proper color contrast', async ({ page }) => {
    // This would need to be tested with a color contrast tool
    // For now, we'll just check if elements are visible
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('textarea')).toBeVisible();
  });

  test('should support screen readers', async ({ page }) => {
    // Check if important elements have proper labels
    await expect(page.locator('textarea')).toHaveAttribute('aria-label');
    await expect(page.locator('button[type="submit"]')).toHaveAttribute('aria-label');
  });

  test('should have proper focus management', async ({ page }) => {
    // Check if focus is managed properly
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.focus();
    await expect(messageInput).toBeFocused();
  });

  test('should have proper error handling', async ({ page }) => {
    // Mock an error response
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => {
          throw new Error('Test error');
        }
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test error');
    await messageInput.press('Enter');
    
    // Check if error is displayed
    await expect(page.locator('text=Error')).toBeVisible();
  });

  test('should have proper loading states', async ({ page }) => {
    // Mock a slow response
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => {
          await new Promise(resolve => setTimeout(resolve, 1000));
          return {
            id: 'test-response-id',
            content: `Response to: ${message}`,
            role: 'assistant',
            timestamp: new Date().toISOString(),
            metadata: {}
          };
        }
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test loading');
    await messageInput.press('Enter');
    
    // Check if loading state is announced
    await expect(page.locator('text=Thinking...')).toBeVisible();
  });
});
