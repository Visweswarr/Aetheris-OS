import React, { useState, useEffect, useCallback } from 'react';
import { ChatInterface } from './components/ChatInterface';
import { Header } from './components/Header';
import { Settings } from './components/Settings';
import { useAppState } from './hooks/useAppState';
import { useSettings } from './hooks/useSettings';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { useNotifications } from './hooks/useNotifications';
import { AppState, Settings as AppSettings } from '../types';

const App: React.FC = () => {
  const [isVisible, setIsVisible] = useState(false);
  const [currentView, setCurrentView] = useState<'chat' | 'settings'>('chat');
  const { appState, updateAppState } = useAppState();
  const { settings, updateSettings } = useSettings();
  const { showNotification } = useNotifications();

  // Handle app visibility changes
  useEffect(() => {
    const handleVisibilityChange = (isVisible: boolean) => {
      setIsVisible(isVisible);
      updateAppState({ isVisible });
    };

    // Listen for visibility changes from main process
    if (window.electronAPI) {
      window.electronAPI.onVisibilityChanged(handleVisibilityChange);
    }

    // Get initial app state
    const getInitialState = async () => {
      try {
        if (window.electronAPI) {
          const state = await window.electronAPI.getAppState();
          setIsVisible(state.isVisible);
          updateAppState(state);
        }
      } catch (error) {
        console.error('Failed to get initial app state:', error);
        showNotification({
          type: 'error',
          title: 'Connection Error',
          message: 'Failed to connect to AI Core service'
        });
      }
    };

    getInitialState();

    return () => {
      if (window.electronAPI) {
        window.electronAPI.removeAllListeners('app:visibility-changed');
      }
    };
  }, [updateAppState, showNotification]);

  // Handle keyboard shortcuts
  useKeyboardShortcuts({
    onToggleSettings: () => {
      setCurrentView(currentView === 'chat' ? 'settings' : 'chat');
    },
    onEscape: () => {
      if (currentView === 'settings') {
        setCurrentView('chat');
      } else if (window.electronAPI) {
        window.electronAPI.hideApp();
      }
    },
    onNewChat: () => {
      // Clear current conversation
      updateAppState({ messageHistory: [] });
    }
  });

  // Handle AI Core connection status
  useEffect(() => {
    const checkConnection = async () => {
      try {
        if (window.electronAPI) {
          const state = await window.electronAPI.getAppState();
          updateAppState({ isConnected: state.isConnected });
        }
      } catch (error) {
        updateAppState({ isConnected: false });
      }
    };

    // Check connection every 5 seconds
    const interval = setInterval(checkConnection, 5000);
    checkConnection();

    return () => clearInterval(interval);
  }, [updateAppState]);

  // Handle window resize and positioning
  const handleWindowResize = useCallback(() => {
    if (window.electronAPI) {
      const bounds = window.electronAPI.getWindowBounds();
      if (bounds) {
        // Update window position based on settings
        const { windowPosition } = settings;
        // Implementation for positioning logic would go here
      }
    }
  }, [settings]);

  useEffect(() => {
    handleWindowResize();
  }, [handleWindowResize]);

  // Don't render if not visible
  if (!isVisible) {
    return null;
  }

  return (
    <div className={`min-h-screen bg-gradient-to-br from-gray-50 to-gray-100 dark:from-gray-900 dark:to-gray-800 transition-all duration-300 ${isVisible ? 'opacity-100' : 'opacity-0'}`}>
      {/* Main container with glass morphism effect */}
      <div className="h-screen flex flex-col glass-dark rounded-lg shadow-2xl overflow-hidden">
        {/* Header */}
        <Header
          currentView={currentView}
          onViewChange={setCurrentView}
          isConnected={appState.isConnected}
          settings={settings}
        />

        {/* Main content */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {currentView === 'chat' ? (
            <ChatInterface
              messageHistory={appState.messageHistory}
              onMessageHistoryChange={(history) => updateAppState({ messageHistory: history })}
              settings={settings}
            />
          ) : (
            <Settings
              settings={settings}
              onSettingsChange={updateSettings}
              onClose={() => setCurrentView('chat')}
            />
          )}
        </div>
      </div>
    </div>
  );
};

export default App;
