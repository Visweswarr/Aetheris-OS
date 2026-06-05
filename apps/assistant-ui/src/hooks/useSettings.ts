import { useState, useCallback, useEffect } from 'react';
import { Settings } from '../types';

const defaultSettings: Settings = {
  theme: 'auto',
  fontSize: 'medium',
  maxHistory: 100,
  autoHide: true,
  hotkey: 'Cmd+Option+A',
  windowPosition: 'center',
  enableStreaming: true,
  enableCitations: true,
  enableTools: true,
  enableMemory: true
};

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(defaultSettings);

  // Load settings from storage on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        if (window.electronAPI) {
          // In a real implementation, this would load from electron-store
          // For now, we'll use localStorage as a fallback
          const stored = localStorage.getItem('assistant-settings');
          if (stored) {
            const parsedSettings = JSON.parse(stored);
            setSettings({ ...defaultSettings, ...parsedSettings });
          }
        }
      } catch (error) {
        console.error('Failed to load settings:', error);
      }
    };

    loadSettings();
  }, []);

  // Save settings to storage when they change
  useEffect(() => {
    const saveSettings = async () => {
      try {
        if (window.electronAPI) {
          // In a real implementation, this would save to electron-store
          // For now, we'll use localStorage as a fallback
          localStorage.setItem('assistant-settings', JSON.stringify(settings));
        }
      } catch (error) {
        console.error('Failed to save settings:', error);
      }
    };

    saveSettings();
  }, [settings]);

  const updateSettings = useCallback((updates: Partial<Settings>) => {
    setSettings(prev => ({ ...prev, ...updates }));
  }, []);

  const resetSettings = useCallback(() => {
    setSettings(defaultSettings);
  }, []);

  const getTheme = useCallback(() => {
    if (settings.theme === 'auto') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    return settings.theme;
  }, [settings.theme]);

  const toggleTheme = useCallback(() => {
    const currentTheme = getTheme();
    const newTheme = currentTheme === 'light' ? 'dark' : 'light';
    updateSettings({ theme: newTheme });
  }, [getTheme, updateSettings]);

  const getFontSize = useCallback(() => {
    const sizes = {
      small: 'text-sm',
      medium: 'text-base',
      large: 'text-lg'
    };
    return sizes[settings.fontSize];
  }, [settings.fontSize]);

  const getHotkeyDisplay = useCallback(() => {
    const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
    return settings.hotkey
      .replace('Cmd', isMac ? '⌘' : 'Ctrl')
      .replace('Option', isMac ? '⌥' : 'Alt')
      .replace('Shift', isMac ? '⇧' : 'Shift')
      .replace('Ctrl', isMac ? '⌃' : 'Ctrl');
  }, [settings.hotkey]);

  return {
    settings,
    updateSettings,
    resetSettings,
    getTheme,
    toggleTheme,
    getFontSize,
    getHotkeyDisplay
  };
}
