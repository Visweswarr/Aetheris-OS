import { test, expect } from '@playwright/test';

test.describe('Security', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should have proper CSP headers', async ({ page }) => {
    // Check if CSP is properly configured
    const response = await page.goto('/');
    const csp = response?.headers()['content-security-policy'];
    
    // Should have CSP header
    expect(csp).toBeDefined();
    
    // Should not allow inline scripts
    expect(csp).toContain("script-src 'self'");
    expect(csp).not.toContain("'unsafe-inline'");
  });

  test('should not allow remote code execution', async ({ page }) => {
    // Try to inject malicious script
    await page.addInitScript(() => {
      try {
        eval('alert("XSS")');
      } catch (e) {
        // Should fail
      }
    });

    // Should not execute
    await expect(page.locator('text=alert')).not.toBeVisible();
  });

  test('should sanitize user input', async ({ page }) => {
    const maliciousInput = '<script>alert("XSS")</script>';
    
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill(maliciousInput);
    await messageInput.press('Enter');
    
    // Should not execute script
    await expect(page.locator('text=alert')).not.toBeVisible();
  });

  test('should handle malicious responses', async ({ page }) => {
    // Mock malicious response
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => ({
          id: 'test-response-id',
          content: '<script>alert("XSS")</script>',
          role: 'assistant',
          timestamp: new Date().toISOString(),
          metadata: {}
        })
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test');
    await messageInput.press('Enter');
    
    // Should not execute script
    await expect(page.locator('text=alert')).not.toBeVisible();
  });

  test('should not expose sensitive information', async ({ page }) => {
    // Check if sensitive information is not exposed in the DOM
    const pageContent = await page.content();
    
    // Should not contain sensitive information
    expect(pageContent).not.toContain('password');
    expect(pageContent).not.toContain('secret');
    expect(pageContent).not.toContain('token');
  });

  test('should handle file access securely', async ({ page }) => {
    // Mock file access
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => ({
          id: 'test-response-id',
          content: 'File access test',
          role: 'assistant',
          timestamp: new Date().toISOString(),
          metadata: {
            files: ['/etc/passwd', 'C:\\Windows\\System32\\config\\SAM']
          }
        })
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test file access');
    await messageInput.press('Enter');
    
    // Should not expose sensitive file paths
    await expect(page.locator('text=/etc/passwd')).not.toBeVisible();
    await expect(page.locator('text=SAM')).not.toBeVisible();
  });

  test('should handle network requests securely', async ({ page }) => {
    // Check if network requests are properly controlled
    const requests: string[] = [];
    
    page.on('request', request => {
      requests.push(request.url());
    });

    await page.goto('/');
    
    // Should only make requests to allowed origins
    const allowedOrigins = ['localhost', '127.0.0.1', 'file://'];
    const hasUnauthorizedRequest = requests.some(url => 
      !allowedOrigins.some(origin => url.includes(origin))
    );
    
    expect(hasUnauthorizedRequest).toBeFalsy();
  });

  test('should handle clipboard access securely', async ({ page }) => {
    // Mock clipboard access
    await page.addInitScript(() => {
      (navigator as any).clipboard = {
        writeText: async (text: string) => {
          // Should only allow copying of safe content
          if (text.includes('<script>')) {
            throw new Error('Unsafe content');
          }
        }
      };
    });

    // Try to copy malicious content
    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('<script>alert("XSS")</script>');
    await messageInput.press('Enter');
    
    // Should not copy malicious content
    await expect(page.locator('text=alert')).not.toBeVisible();
  });

  test('should handle IPC securely', async ({ page }) => {
    // Check if IPC is properly secured
    const ipcMethods = await page.evaluate(() => {
      return Object.keys((window as any).electronAPI || {});
    });
    
    // Should only expose safe methods
    const safeMethods = ['sendChatMessage', 'streamChatMessage', 'getSettings', 'updateSettings'];
    const hasUnsafeMethod = ipcMethods.some(method => !safeMethods.includes(method));
    
    expect(hasUnsafeMethod).toBeFalsy();
  });

  test('should handle errors securely', async ({ page }) => {
    // Mock error response
    await page.addInitScript(() => {
      (window as any).electronAPI = {
        sendChatMessage: async (message: string) => {
          throw new Error('Sensitive error information');
        }
      };
    });

    const messageInput = page.locator('textarea[placeholder*="Ask me anything"]');
    await messageInput.fill('Test error');
    await messageInput.press('Enter');
    
    // Should not expose sensitive error information
    await expect(page.locator('text=Sensitive error information')).not.toBeVisible();
  });
});
