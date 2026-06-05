import { useState, useCallback } from 'react';
import { AppState, ChatMessage } from '../types';

const initialState: AppState = {
  isVisible: false,
  version: '1.0.0',
  platform: 'unknown',
  isConnected: false,
  messageHistory: []
};

export function useAppState() {
  const [appState, setAppState] = useState<AppState>(initialState);

  const updateAppState = useCallback((updates: Partial<AppState>) => {
    setAppState(prev => ({ ...prev, ...updates }));
  }, []);

  const addMessage = useCallback((message: ChatMessage) => {
    setAppState(prev => ({
      ...prev,
      messageHistory: [...prev.messageHistory, message]
    }));
  }, []);

  const updateMessage = useCallback((messageId: string, updates: Partial<ChatMessage>) => {
    setAppState(prev => ({
      ...prev,
      messageHistory: prev.messageHistory.map(msg =>
        msg.id === messageId ? { ...msg, ...updates } : msg
      )
    }));
  }, []);

  const removeMessage = useCallback((messageId: string) => {
    setAppState(prev => ({
      ...prev,
      messageHistory: prev.messageHistory.filter(msg => msg.id !== messageId)
    }));
  }, []);

  const clearMessageHistory = useCallback(() => {
    setAppState(prev => ({
      ...prev,
      messageHistory: []
    }));
  }, []);

  const setConnectionStatus = useCallback((isConnected: boolean) => {
    setAppState(prev => ({ ...prev, isConnected }));
  }, []);

  const setVisibility = useCallback((isVisible: boolean) => {
    setAppState(prev => ({ ...prev, isVisible }));
  }, []);

  return {
    appState,
    updateAppState,
    addMessage,
    updateMessage,
    removeMessage,
    clearMessageHistory,
    setConnectionStatus,
    setVisibility
  };
}
