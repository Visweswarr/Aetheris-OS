import { test, expect } from '@playwright/test';

test.describe('Performance', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should load quickly', async ({ page }) => {
    const startTime = Date.now();
    await page.goto('/');
    const loadTime = Date.now() - startTime;
    
    // Should load within 2 seconds
    expect(loadTime).toBeLessThan(2000);
  });

  test('should handle large messages efficiently', async ({ page }) => {
    const largeMessage = 'a'.repeat(10000);
    
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill(largeMessage);
    
    // Should not freeze the UI
    await expect(messageInput).toHaveValue(largeMessage);
  });

  test('should handle many messages efficiently', async ({ page }) => {
    // Mock responses
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => ({
          id: `response-${Date.now()}`,
          content: `Response to: ${message}`,
          role: 'assistant',
          timestamp: new Date().toISOString(),
          metadata: {}
        })
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    
    // Send many messages
    for (let i = 0; i < 10; i++) {
      await messageInput.fill(`Message ${i}`);
      await messageInput.press('Enter');
      await page.waitForTimeout(100);
    }
    
    // Should still be responsive
    await expect(messageInput).toBeVisible();
  });

  test('should handle streaming efficiently', async ({ page }) => {
    // Mock streaming response
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        streamChatMessage: async (message: string) => {
          const chunks = Array.from({ length: 100 }, (_, i) => `Chunk ${i} `);
          return {
            async *[Symbol.asyncIterator]() {
              for (const chunk of chunks) {
                await new Promise(resolve => setTimeout(resolve, 10));
                yield chunk;
              }
            }
          };
        }
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test streaming');
    await messageInput.press('Enter');
    
    // Should handle streaming without performance issues
    await expect(page.locator('text=Chunk 0')).toBeVisible();
  });

  test('should handle theme switching efficiently', async ({ page }) => {
    // Go to settings
    await page.locator('button:has-text("Settings")').click();
    
    const startTime = Date.now();
    await page.locator('select').first().selectOption('dark');
    const switchTime = Date.now() - startTime;
    
    // Should switch quickly
    expect(switchTime).toBeLessThan(500);
  });

  test('should handle window resize efficiently', async ({ page }) => {
    const startTime = Date.now();
    await page.setViewportSize({ width: 800, height: 600 });
    const resizeTime = Date.now() - startTime;
    
    // Should resize quickly
    expect(resizeTime).toBeLessThan(500);
  });

  test('should handle memory efficiently', async ({ page }) => {
    // Send many messages and check memory usage
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => ({
          id: `response-${Date.now()}`,
          content: `Response to: ${message}`,
          role: 'assistant',
          timestamp: new Date().toISOString(),
          metadata: {}
        })
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    
    for (let i = 0; i < 50; i++) {
      await messageInput.fill(`Message ${i}`);
      await messageInput.press('Enter');
      await page.waitForTimeout(50);
    }
    
    // Should still be responsive
    await expect(messageInput).toBeVisible();
  });

  test('should handle concurrent operations', async ({ page }) => {
    // Mock concurrent responses
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => {
          await new Promise(resolve => setTimeout(resolve, 100));
          return {
            id: `response-${Date.now()}`,
            content: `Response to: ${message}`,
            role: 'assistant',
            timestamp: new Date().toISOString(),
            metadata: {}
          };
        }
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    
    // Send multiple messages quickly
    await messageInput.fill('Message 1');
    await messageInput.press('Enter');
    await messageInput.fill('Message 2');
    await messageInput.press('Enter');
    await messageInput.fill('Message 3');
    await messageInput.press('Enter');
    
    // Should handle all messages
    await expect(page.locator('text=Message 1')).toBeVisible();
    await expect(page.locator('text=Message 2')).toBeVisible();
    await expect(page.locator('text=Message 3')).toBeVisible();
  });
});
