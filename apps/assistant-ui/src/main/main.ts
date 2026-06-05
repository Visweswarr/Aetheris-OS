import { app, BrowserWindow, globalShortcut, ipcMain, Menu, shell } from 'electron';
import { join } from 'path';
import { isDev } from './utils';
import { AICoreBridge } from '../bridge/ai-core-bridge';

class AssistantApp {
  private mainWindow: BrowserWindow | null = null;
  private aiBridge: AICoreBridge;
  private isVisible = false;

  constructor() {
    this.aiBridge = new AICoreBridge();
    this.setupApp();
  }

  private setupApp(): void {
    // Handle app ready
    app.whenReady().then(() => {
      this.createWindow();
      this.setupGlobalShortcuts();
      this.setupIPC();
      this.setupMenu();
    });

    // Handle window closed
    app.on('window-all-closed', () => {
      // On macOS, keep app running even when all windows are closed
      if (process.platform !== 'darwin') {
        app.quit();
      }
    });

    // Handle app activation (macOS)
    app.on('activate', () => {
      if (BrowserWindow.getAllWindows().length === 0) {
        this.createWindow();
      }
    });

    // Handle app before quit
    app.on('before-quit', () => {
      this.cleanup();
    });
  }

  private createWindow(): void {
    // Create the browser window
    this.mainWindow = new BrowserWindow({
      width: 400,
      height: 600,
      minWidth: 300,
      minHeight: 400,
      maxWidth: 600,
      maxHeight: 800,
      show: false, // Don't show initially
      frame: false, // Frameless window for overlay effect
      transparent: true, // Transparent background
      alwaysOnTop: true, // Always on top
      skipTaskbar: true, // Don't show in taskbar
      resizable: true,
      minimizable: false,
      maximizable: false,
      closable: true,
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
        enableRemoteModule: false,
        preload: join(__dirname, 'preload.js'),
        webSecurity: true,
        allowRunningInsecureContent: false,
        experimentalFeatures: false,
      },
    });

    // Load the app
    if (isDev()) {
      this.mainWindow.loadURL('http://localhost:3000');
      this.mainWindow.webContents.openDevTools();
    } else {
      this.mainWindow.loadFile(join(__dirname, '../renderer/index.html'));
    }

    // Handle window events
    this.mainWindow.on('blur', () => {
      // Hide window when it loses focus
      this.hideWindow();
    });

    this.mainWindow.on('closed', () => {
      this.mainWindow = null;
    });

    // Handle external links
    this.mainWindow.webContents.setWindowOpenHandler(({ url }) => {
      shell.openExternal(url);
      return { action: 'deny' };
    });
  }

  private setupGlobalShortcuts(): void {
    // Register global shortcuts
    const shortcut = process.platform === 'darwin' ? 'Cmd+Option+A' : 'Shift+Alt+A';
    
    const ret = globalShortcut.register(shortcut, () => {
      this.toggleWindow();
    });

    if (!ret) {
      console.error('Failed to register global shortcut');
    }

    // Register escape key to hide window
    const escapeShortcut = 'Escape';
    globalShortcut.register(escapeShortcut, () => {
      this.hideWindow();
    });
  }

  private setupIPC(): void {
    // Handle chat messages
    ipcMain.handle('chat:send', async (event, message: string) => {
      try {
        return await this.aiBridge.sendChatMessage(message);
      } catch (error) {
        console.error('Error sending chat message:', error);
        throw error;
      }
    });

    // Handle streaming responses
    ipcMain.handle('chat:stream', async (event, message: string) => {
      try {
        return await this.aiBridge.streamChatMessage(message);
      } catch (error) {
        console.error('Error streaming chat message:', error);
        throw error;
      }
    });

    // Handle tool calls
    ipcMain.handle('tools:call', async (event, toolName: string, parameters: any) => {
      try {
        return await this.aiBridge.callTool(toolName, parameters);
      } catch (error) {
        console.error('Error calling tool:', error);
        throw error;
      }
    });

    // Handle memory operations
    ipcMain.handle('memory:get', async (event, key: string) => {
      try {
        return await this.aiBridge.getMemory(key);
      } catch (error) {
        console.error('Error getting memory:', error);
        throw error;
      }
    });

    ipcMain.handle('memory:set', async (event, key: string, value: any) => {
      try {
        return await this.aiBridge.setMemory(key, value);
      } catch (error) {
        console.error('Error setting memory:', error);
        throw error;
      }
    });

    // Handle app state
    ipcMain.handle('app:get-state', () => {
      return {
        isVisible: this.isVisible,
        version: app.getVersion(),
        platform: process.platform,
      };
    });

    ipcMain.handle('app:hide', () => {
      this.hideWindow();
    });

    ipcMain.handle('app:show', () => {
      this.showWindow();
    });

    // Handle window positioning
    ipcMain.handle('window:get-bounds', () => {
      return this.mainWindow?.getBounds();
    });

    ipcMain.handle('window:set-bounds', (event, bounds) => {
      this.mainWindow?.setBounds(bounds);
    });
  }

  private setupMenu(): void {
    // Create application menu
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'Aetheris Assistant',
        submenu: [
          {
            label: 'About Aetheris Assistant',
            role: 'about',
          },
          { type: 'separator' },
          {
            label: 'Toggle Assistant',
            accelerator: process.platform === 'darwin' ? 'Cmd+Option+A' : 'Shift+Alt+A',
            click: () => {
              this.toggleWindow();
            },
          },
          { type: 'separator' },
          {
            label: 'Quit',
            accelerator: process.platform === 'darwin' ? 'Cmd+Q' : 'Ctrl+Q',
            click: () => {
              app.quit();
            },
          },
        ],
      },
      {
        label: 'Edit',
        submenu: [
          { label: 'Undo', accelerator: 'CmdOrCtrl+Z', role: 'undo' },
          { label: 'Redo', accelerator: 'Shift+CmdOrCtrl+Z', role: 'redo' },
          { type: 'separator' },
          { label: 'Cut', accelerator: 'CmdOrCtrl+X', role: 'cut' },
          { label: 'Copy', accelerator: 'CmdOrCtrl+C', role: 'copy' },
          { label: 'Paste', accelerator: 'CmdOrCtrl+V', role: 'paste' },
        ],
      },
      {
        label: 'View',
        submenu: [
          { label: 'Reload', accelerator: 'CmdOrCtrl+R', role: 'reload' },
          { label: 'Force Reload', accelerator: 'CmdOrCtrl+Shift+R', role: 'forceReload' },
          { label: 'Toggle Developer Tools', accelerator: 'F12', role: 'toggleDevTools' },
          { type: 'separator' },
          { label: 'Actual Size', accelerator: 'CmdOrCtrl+0', role: 'resetZoom' },
          { label: 'Zoom In', accelerator: 'CmdOrCtrl+Plus', role: 'zoomIn' },
          { label: 'Zoom Out', accelerator: 'CmdOrCtrl+-', role: 'zoomOut' },
          { type: 'separator' },
          { label: 'Toggle Fullscreen', accelerator: 'F11', role: 'togglefullscreen' },
        ],
      },
    ];

    const menu = Menu.buildFromTemplate(template);
    Menu.setApplicationMenu(menu);
  }

  private toggleWindow(): void {
    if (this.isVisible) {
      this.hideWindow();
    } else {
      this.showWindow();
    }
  }

  private showWindow(): void {
    if (!this.mainWindow) {
      this.createWindow();
    }

    if (this.mainWindow) {
      // Center window on screen
      this.mainWindow.center();
      this.mainWindow.show();
      this.mainWindow.focus();
      this.isVisible = true;

      // Notify renderer
      this.mainWindow.webContents.send('app:visibility-changed', true);
    }
  }

  private hideWindow(): void {
    if (this.mainWindow) {
      this.mainWindow.hide();
      this.isVisible = false;

      // Notify renderer
      this.mainWindow.webContents.send('app:visibility-changed', false);
    }
  }

  private cleanup(): void {
    // Unregister all global shortcuts
    globalShortcut.unregisterAll();
    
    // Cleanup AI bridge
    this.aiBridge.cleanup();
  }
}

// Create and run the app
const assistantApp = new AssistantApp();

// Export for testing
export { AssistantApp };
