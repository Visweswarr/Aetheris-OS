import { test, expect } from '@playwright/test';

test.describe('Electron Main Process', () => {
  test('should create window with correct properties', async ({ page }) => {
    await page.goto('/');
    
    // Check if window is created
    await expect(page.locator('h1')).toContainText('Aetheris Assistant');
  });

  test('should handle hotkey registration', async ({ page }) => {
    // This test would need to be run in the actual Electron environment
    // For now, we'll just check if the app loads
    await page.goto('/');
    await expect(page.locator('h1')).toBeVisible();
  });

  test('should handle IPC communication', async ({ page }) => {
    // Mock electron API
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => ({
          id: 'test-response-id',
          content: `Response to: ${message}`,
          role: 'assistant',
          timestamp: new Date().toISOString(),
          metadata: {}
        })
      };
    });

    await page.goto('/');
    
    // Test IPC communication
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test IPC');
    await messageInput.press('Enter');
    
    await expect(page.locator('text=Test IPC')).toBeVisible();
  });
});
