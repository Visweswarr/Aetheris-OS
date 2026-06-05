import { contextBridge, ipcRenderer } from 'electron';

// Expose protected methods that allow the renderer process to use
// the ipcRenderer without exposing the entire object
contextBridge.exposeInMainWorld('electronAPI', {
  // Chat operations
  sendChatMessage: (message: string) => ipcRenderer.invoke('chat:send', message),
  streamChatMessage: (message: string) => ipcRenderer.invoke('chat:stream', message),
  
  // Tool operations
  callTool: (toolName: string, parameters: any) => ipcRenderer.invoke('tools:call', toolName, parameters),
  
  // Memory operations
  getMemory: (key: string) => ipcRenderer.invoke('memory:get', key),
  setMemory: (key: string, value: any) => ipcRenderer.invoke('memory:set', key, value),
  
  // App operations
  getAppState: () => ipcRenderer.invoke('app:get-state'),
  hideApp: () => ipcRenderer.invoke('app:hide'),
  showApp: () => ipcRenderer.invoke('app:show'),
  
  // Window operations
  getWindowBounds: () => ipcRenderer.invoke('window:get-bounds'),
  setWindowBounds: (bounds: any) => ipcRenderer.invoke('window:set-bounds', bounds),
  
  // Event listeners
  onVisibilityChanged: (callback: (isVisible: boolean) => void) => {
    ipcRenderer.on('app:visibility-changed', (event, isVisible) => callback(isVisible));
  },
  
  onChatResponse: (callback: (response: any) => void) => {
    ipcRenderer.on('chat:response', (event, response) => callback(response));
  },
  
  onChatStream: (callback: (chunk: any) => void) => {
    ipcRenderer.on('chat:stream-chunk', (event, chunk) => callback(chunk));
  },
  
  onToolResult: (callback: (result: any) => void) => {
    ipcRenderer.on('tools:result', (event, result) => callback(result));
  },
  
  // Remove listeners
  removeAllListeners: (channel: string) => {
    ipcRenderer.removeAllListeners(channel);
  },
});

// Type definitions for the exposed API
declare global {
  interface Window {
    electronAPI: {
      sendChatMessage: (message: string) => Promise<any>;
      streamChatMessage: (message: string) => Promise<any>;
      callTool: (toolName: string, parameters: any) => Promise<any>;
      getMemory: (key: string) => Promise<any>;
      setMemory: (key: string, value: any) => Promise<any>;
      getAppState: () => Promise<any>;
      hideApp: () => Promise<void>;
      showApp: () => Promise<void>;
      getWindowBounds: () => Promise<any>;
      setWindowBounds: (bounds: any) => Promise<void>;
      onVisibilityChanged: (callback: (isVisible: boolean) => void) => void;
      onChatResponse: (callback: (response: any) => void) => void;
      onChatStream: (callback: (chunk: any) => void) => void;
      onToolResult: (callback: (result: any) => void) => void;
      removeAllListeners: (channel: string) => void;
    };
  }
}
