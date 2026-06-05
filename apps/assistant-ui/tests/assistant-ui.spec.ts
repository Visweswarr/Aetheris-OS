import { test, expect } from '@playwright/test';

test.describe('Aetheris Assistant UI', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display the main interface', async ({ page }) => {
    // Check if the main elements are present
    await expect(page.locator('h1')).toContainText('Aetheris Assistant');
    await expect(page.locator('[data-testid="chat-interface"]')).toBeVisible();
  });

  test('should show welcome message when no messages', async ({ page }) => {
    await expect(page.locator('text=Welcome to Aetheris Assistant')).toBeVisible();
    await expect(page.locator('text=What would you like to know?')).toBeVisible();
  });

  test('should allow typing in message input', async ({ page }) => {
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Hello, this is a test message');
    await expect(messageInput).toHaveValue('Hello, this is a test message');
  });

  test('should show send button when message is typed', async ({ page }) => {
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    const sendButton = page.locator('button[type="submit"]');
    
    // Initially disabled
    await expect(sendButton).toBeDisabled();
    
    // Type a message
    await messageInput.fill('Test message');
    await expect(sendButton).toBeEnabled();
  });

  test('should handle Enter key to send message', async ({ page }) => {
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test message');
    
    // Mock the electron API
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => ({
          id: 'test-response-id',
          content: `Response to: ${message}`,
          role: 'assistant',
          timestamp: new Date().toISOString(),
          metadata: {}
        }),
        streamChatMessage: async (message: string) => {
          const chunks = [`Response to: ${message}`.split('')];
          return {
            async *[Symbol.asyncIterator]() {
              for (const chunk of chunks) {
                yield chunk.join('');
              }
            }
          };
        }
      };
    });

    // Send message with Enter key
    await messageInput.press('Enter');
    
    // Check if message appears in chat
    await expect(page.locator('text=Test message')).toBeVisible();
  });

  test('should handle Shift+Enter for new line', async ({ page }) => {
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Line 1');
    await messageInput.press('Shift+Enter');
    await messageInput.type('Line 2');
    
    await expect(messageInput).toHaveValue('Line 1\nLine 2');
  });

  test('should show character count', async ({ page }) => {
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Hello');
    
    await expect(page.locator('text=5/4000')).toBeVisible();
  });

  test('should respect max length', async ({ page }) => {
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    const longMessage = 'a'.repeat(4001);
    
    await messageInput.fill(longMessage);
    await expect(messageInput).toHaveValue('a'.repeat(4000));
  });

  test('should toggle between chat and settings views', async ({ page }) => {
    // Click settings button
    await page.locator('button:has-text("Settings")').click();
    await expect(page.locator('h2:has-text("Settings")')).toBeVisible();
    
    // Click chat button
    await page.locator('button:has-text("Chat")').click();
    await expect(page.locator('text=Welcome to Aetheris Assistant')).toBeVisible();
  });

  test('should show connection status', async ({ page }) => {
    const connectionStatus = page.locator('text=Connected, text=Disconnected').first();
    await expect(connectionStatus).toBeVisible();
  });

  test('should handle keyboard shortcuts', async ({ page }) => {
    // Test Cmd/Ctrl + , for settings
    await page.keyboard.press('Meta+,');
    await expect(page.locator('h2:has-text("Settings")')).toBeVisible();
    
    // Test Escape to go back to chat
    await page.keyboard.press('Escape');
    await expect(page.locator('text=Welcome to Aetheris Assistant')).toBeVisible();
  });

  test('should show toolbar actions', async ({ page }) => {
    // Check if toolbar is present
    await expect(page.locator('[data-testid="toolbar"]')).toBeVisible();
    
    // Check message count
    await expect(page.locator('text=0 messages')).toBeVisible();
  });

  test('should handle copy conversation', async ({ page }) => {
    // Mock clipboard API
    await page.addInitScript(() => {
      (navigator as any).clipboard = {
        writeText: async (text: string) => {
          (window as any).lastCopiedText = text;
        }
      };
    });

    // Add a test message first
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test message');
    await messageInput.press('Enter');
    
    // Wait for message to appear
    await expect(page.locator('text=Test message')).toBeVisible();
    
    // Click copy button
    await page.locator('button[title*="Copy conversation"]').click();
    
    // Check if clipboard was called
    const lastCopiedText = await page.evaluate(() => (window as any).lastCopiedText);
    expect(lastCopiedText).toContain('Test message');
  });

  test('should handle clear conversation', async ({ page }) => {
    // Add a test message first
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test message');
    await messageInput.press('Enter');
    
    // Wait for message to appear
    await expect(page.locator('text=Test message')).toBeVisible();
    
    // Click clear button
    await page.locator('button[title*="Clear conversation"]').click();
    
    // Check if conversation is cleared
    await expect(page.locator('text=Welcome to Aetheris Assistant')).toBeVisible();
  });

  test('should show loading state', async ({ page }) => {
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
    await messageInput.fill('Test message');
    await messageInput.press('Enter');
    
    // Check if loading state is shown
    await expect(page.locator('text=Thinking...')).toBeVisible();
    await expect(page.locator('text=AI is responding...')).toBeVisible();
  });

  test('should handle streaming responses', async ({ page }) => {
    // Mock streaming response
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        streamChatMessage: async (message: string) => {
          const chunks = ['Hello', ' there', '! How', ' can I', ' help', ' you?'];
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
    await expect(page.locator('text=Hello there! How can I help you?')).toBeVisible();
  });

  test('should show citations when present', async ({ page }) => {
    // Mock response with citations
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => ({
          id: 'test-response-id',
          content: 'This is a response with citations [1] and [2].',
          role: 'assistant',
          timestamp: new Date().toISOString(),
          metadata: {
            citations: ['Citation 1', 'Citation 2']
          }
        })
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test message');
    await messageInput.press('Enter');
    
    // Check if citations are shown
    await expect(page.locator('text=2 citations')).toBeVisible();
  });

  test('should handle theme switching', async ({ page }) => {
    // Go to settings
    await page.locator('button:has-text("Settings")').click();
    
    // Change theme
    await page.locator('select').first().selectOption('dark');
    
    // Check if dark theme is applied
    await expect(page.locator('html')).toHaveClass(/dark/);
  });

  test('should handle font size changes', async ({ page }) => {
    // Go to settings
    await page.locator('button:has-text("Settings")').click();
    
    // Change font size
    await page.locator('select').nth(1).selectOption('large');
    
    // Check if large font size is applied
    await expect(page.locator('body')).toHaveClass(/text-lg/);
  });

  test('should handle feature toggles', async ({ page }) => {
    // Go to settings
    await page.locator('button:has-text("Settings")').click();
    
    // Toggle streaming
    await page.locator('button[role="switch"]').first().click();
    
    // Check if toggle state changed
    await expect(page.locator('button[role="switch"]').first()).toHaveAttribute('aria-checked', 'false');
  });

  test('should show keyboard shortcuts help', async ({ page }) => {
    // Go to settings
    await page.locator('button:has-text("Settings")').click();
    
    // Check if keyboard shortcuts section is visible
    await expect(page.locator('h3:has-text("Keyboard Shortcuts")')).toBeVisible();
    await expect(page.locator('text=Toggle Assistant')).toBeVisible();
    await expect(page.locator('text=Hide Assistant')).toBeVisible();
  });

  test('should show about information', async ({ page }) => {
    // Go to settings
    await page.locator('button:has-text("Settings")').click();
    
    // Check if about section is visible
    await expect(page.locator('h3:has-text("About")')).toBeVisible();
    await expect(page.locator('text=Version')).toBeVisible();
    await expect(page.locator('text=Platform')).toBeVisible();
  });

  test('should handle window resize', async ({ page }) => {
    // Test responsive behavior
    await page.setViewportSize({ width: 800, height: 600 });
    await expect(page.locator('h1')).toBeVisible();
    
    await page.setViewportSize({ width: 400, height: 600 });
    await expect(page.locator('h1')).toBeVisible();
  });

  test('should handle mobile viewport', async ({ page }) => {
    // Set mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    
    // Check if interface adapts
    await expect(page.locator('h1')).toBeVisible();
    await expect(page.locator('textarea[placeholder*="Ask me anything"]')).toBeVisible();
  });
});
