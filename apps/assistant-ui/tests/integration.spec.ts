import { test, expect } from '@playwright/test';

test.describe('Integration Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should handle complete chat flow', async ({ page }) => {
    // Mock AI responses
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => ({
          id: `response-${Date.now()}`,
          content: `AI Response to: ${message}`,
          role: 'assistant',
          timestamp: new Date().toISOString(),
          metadata: {
            citations: ['Source 1', 'Source 2']
          }
        })
      };
    });

    // Start conversation
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Hello, how are you?');
    await messageInput.press('Enter');
    
    // Check if message appears
    await expect(page.locator('text=Hello, how are you?')).toBeVisible();
    await expect(page.locator('text=AI Response to: Hello, how are you?')).toBeVisible();
    
    // Check if citations are shown
    await expect(page.locator('text=2 citations')).toBeVisible();
    
    // Continue conversation
    await messageInput.fill('What can you help me with?');
    await messageInput.press('Enter');
    
    // Check if second message appears
    await expect(page.locator('text=What can you help me with?')).toBeVisible();
    await expect(page.locator('text=AI Response to: What can you help me with?')).toBeVisible();
  });

  test('should handle streaming conversation', async ({ page }) => {
    // Mock streaming response
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        streamChatMessage: async (message: string) => {
          const chunks = ['Hello', ' there!', ' How', ' can', ' I', ' help', ' you', ' today?'];
          return {
            async *[Symbol.asyncIterator]() {
              for (const chunk of chunks) {
                await new Promise(resolve => setTimeout(resolve, 100));
                yield chunk;
              }
            }
          };
        }
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Hello');
    await messageInput.press('Enter');
    
    // Check if streaming response appears
    await expect(page.locator('text=Hello there! How can I help you today?')).toBeVisible();
  });

  test('should handle settings persistence', async ({ page }) => {
    // Go to settings
    await page.locator('button:has-text("Settings")').click();
    
    // Change theme
    await page.locator('select').first().selectOption('dark');
    
    // Change font size
    await page.locator('select').nth(1).selectOption('large');
    
    // Toggle streaming
    await page.locator('button[role="switch"]').first().click();
    
    // Go back to chat
    await page.locator('button:has-text("Chat")').click();
    
    // Check if settings are persisted
    await expect(page.locator('html')).toHaveClass(/dark/);
    await expect(page.locator('body')).toHaveClass(/text-lg/);
  });

  test('should handle keyboard shortcuts integration', async ({ page }) => {
    // Test Cmd/Ctrl + , for settings
    await page.keyboard.press('Meta+,');
    await expect(page.locator('h2:has-text("Settings")')).toBeVisible();
    
    // Test Escape to go back
    await page.keyboard.press('Escape');
    await expect(page.locator('text=Welcome to Aetheris Assistant')).toBeVisible();
  });

  test('should handle copy conversation integration', async ({ page }) => {
    // Mock clipboard API
    await page.addInitScript(() => {
      (navigator as any).clipboard = {
        writeText: async (text: string) => {
          (window as any).lastCopiedText = text;
        }
      };
    });

    // Add messages
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Message 1');
    await messageInput.press('Enter');
    await messageInput.fill('Message 2');
    await messageInput.press('Enter');
    
    // Copy conversation
    await page.locator('button[title*="Copy conversation"]').click();
    
    // Check if conversation was copied
    const lastCopiedText = await page.evaluate(() => (window as any).lastCopiedText);
    expect(lastCopiedText).toContain('Message 1');
    expect(lastCopiedText).toContain('Message 2');
  });

  test('should handle clear conversation integration', async ({ page }) => {
    // Add messages
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Message 1');
    await messageInput.press('Enter');
    await messageInput.fill('Message 2');
    await messageInput.press('Enter');
    
    // Clear conversation
    await page.locator('button[title*="Clear conversation"]').click();
    
    // Check if conversation is cleared
    await expect(page.locator('text=Welcome to Aetheris Assistant')).toBeVisible();
    await expect(page.locator('text=Message 1')).not.toBeVisible();
    await expect(page.locator('text=Message 2')).not.toBeVisible();
  });

  test('should handle error recovery', async ({ page }) => {
    // Mock error response
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => {
          if (message.includes('error')) {
            throw new Error('Test error');
          }
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

    // Send message that causes error
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('This will cause an error');
    await messageInput.press('Enter');
    
    // Check if error is handled
    await expect(page.locator('text=Error')).toBeVisible();
    
    // Send normal message
    await messageInput.fill('This should work');
    await messageInput.press('Enter');
    
    // Check if normal message works
    await expect(page.locator('text=This should work')).toBeVisible();
    await expect(page.locator('text=Response to: This should work')).toBeVisible();
  });

  test('should handle concurrent operations', async ({ page }) => {
    // Mock concurrent responses
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => {
          await new Promise(resolve => setTimeout(resolve, 200));
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
    
    // Check if all messages are handled
    await expect(page.locator('text=Message 1')).toBeVisible();
    await expect(page.locator('text=Message 2')).toBeVisible();
    await expect(page.locator('text=Message 3')).toBeVisible();
  });

  test('should handle window state management', async ({ page }) => {
    // Test window resize
    await page.setViewportSize({ width: 800, height: 600 });
    await expect(page.locator('h1')).toBeVisible();
    
    await page.setViewportSize({ width: 400, height: 600 });
    await expect(page.locator('h1')).toBeVisible();
    
    // Test mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await expect(page.locator('h1')).toBeVisible();
  });

  test('should handle theme switching integration', async ({ page }) => {
    // Go to settings
    await page.locator('button:has-text("Settings")').click();
    
    // Switch to dark theme
    await page.locator('select').first().selectOption('dark');
    await expect(page.locator('html')).toHaveClass(/dark/);
    
    // Switch to light theme
    await page.locator('select').first().selectOption('light');
    await expect(page.locator('html')).not.toHaveClass(/dark/);
    
    // Switch to system theme
    await page.locator('select').first().selectOption('system');
    // System theme behavior depends on OS
  });
});
