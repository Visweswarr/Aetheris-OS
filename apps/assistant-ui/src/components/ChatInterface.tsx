import React, { useState, useRef, useEffect, useCallback } from 'react';
import { ChatMessage, Settings } from '../types';
import { MessageList } from './MessageList';
import { MessageInput } from './MessageInput';
import { Toolbar } from './Toolbar';
import { useNotifications } from '../hooks/useNotifications';
import { generateId, formatTimestamp } from '../utils';

interface ChatInterfaceProps {
  messageHistory: ChatMessage[];
  onMessageHistoryChange: (history: ChatMessage[]) => void;
  settings: Settings;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({
  messageHistory,
  onMessageHistoryChange,
  settings
}) => {
  const [isLoading, setIsLoading] = useState(false);
  const [streamingMessage, setStreamingMessage] = useState<string>('');
  const [currentStreamId, setCurrentStreamId] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const { showNotification, showError } = useNotifications();

  // Auto-scroll to bottom when new messages arrive
  const scrollToBottom = useCallback(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messageHistory, streamingMessage, scrollToBottom]);

  // Handle sending a message
  const handleSendMessage = useCallback(async (content: string) => {
    if (!content.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      id: generateId(),
      content: content.trim(),
      role: 'user',
      timestamp: new Date()
    };

    // Add user message to history
    const newHistory = [...messageHistory, userMessage];
    onMessageHistoryChange(newHistory);

    setIsLoading(true);
    setStreamingMessage('');
    setCurrentStreamId(null);

    try {
      if (settings.enableStreaming) {
        // Handle streaming response
        await handleStreamingResponse(content.trim());
      } else {
        // Handle regular response
        await handleRegularResponse(content.trim());
      }
    } catch (error) {
      console.error('Error sending message:', error);
      showError('Failed to send message', error instanceof Error ? error.message : 'Unknown error');
    } finally {
      setIsLoading(false);
      setStreamingMessage('');
      setCurrentStreamId(null);
    }
  }, [messageHistory, onMessageHistoryChange, isLoading, settings.enableStreaming, showError]);

  // Handle regular (non-streaming) response
  const handleRegularResponse = useCallback(async (content: string) => {
    if (!window.electronAPI) {
      throw new Error('Electron API not available');
    }

    const response = await window.electronAPI.sendChatMessage(content);
    
    const assistantMessage: ChatMessage = {
      id: response.id || generateId(),
      content: response.content,
      role: 'assistant',
      timestamp: new Date(response.timestamp || Date.now()),
      metadata: {
        citations: response.citations,
        tools_used: response.tools_used,
        confidence: response.metadata?.confidence,
        processing_time_ms: response.metadata?.processing_time_ms
      }
    };

    onMessageHistoryChange([...messageHistory, assistantMessage]);
  }, [messageHistory, onMessageHistoryChange]);

  // Handle streaming response
  const handleStreamingResponse = useCallback(async (content: string) => {
    if (!window.electronAPI) {
      throw new Error('Electron API not available');
    }

    const streamId = generateId();
    setCurrentStreamId(streamId);

    // Create initial assistant message
    const assistantMessage: ChatMessage = {
      id: streamId,
      content: '',
      role: 'assistant',
      timestamp: new Date(),
      metadata: {}
    };

    const newHistory = [...messageHistory, assistantMessage];
    onMessageHistoryChange(newHistory);

    try {
      // Start streaming
      const stream = await window.electronAPI.streamChatMessage(content);
      
      let fullContent = '';
      for await (const chunk of stream) {
        if (currentStreamId === streamId) {
          fullContent += chunk;
          setStreamingMessage(fullContent);
          
          // Update the message in history
          const updatedHistory = newHistory.map(msg =>
            msg.id === streamId ? { ...msg, content: fullContent } : msg
          );
          onMessageHistoryChange(updatedHistory);
        }
      }
    } catch (error) {
      console.error('Error streaming response:', error);
      throw error;
    }
  }, [messageHistory, onMessageHistoryChange, currentStreamId]);

  // Handle clearing conversation
  const handleClearConversation = useCallback(() => {
    onMessageHistoryChange([]);
    showNotification({
      type: 'info',
      title: 'Conversation Cleared',
      message: 'All messages have been removed'
    });
  }, [onMessageHistoryChange, showNotification]);

  // Handle copying conversation
  const handleCopyConversation = useCallback(async () => {
    const conversationText = messageHistory
      .map(msg => `${msg.role === 'user' ? 'You' : 'Assistant'}: ${msg.content}`)
      .join('\n\n');

    try {
      await navigator.clipboard.writeText(conversationText);
      showNotification({
        type: 'success',
        title: 'Copied to Clipboard',
        message: 'Conversation has been copied to clipboard'
      });
    } catch (error) {
      showError('Failed to copy conversation', 'Could not copy to clipboard');
    }
  }, [messageHistory, showNotification, showError]);

  // Handle exporting conversation
  const handleExportConversation = useCallback(() => {
    const conversationText = messageHistory
      .map(msg => `${msg.role === 'user' ? 'You' : 'Assistant'}: ${msg.content}`)
      .join('\n\n');

    const blob = new Blob([conversationText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `aetheris-conversation-${new Date().toISOString().split('T')[0]}.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    showNotification({
      type: 'success',
      title: 'Conversation Exported',
      message: 'Conversation has been saved to your downloads'
    });
  }, [messageHistory, showNotification]);

  return (
    <div className="flex flex-col h-full bg-white dark:bg-gray-900">
      {/* Toolbar */}
      <Toolbar
        onClearConversation={handleClearConversation}
        onCopyConversation={handleCopyConversation}
        onExportConversation={handleExportConversation}
        messageCount={messageHistory.length}
        isLoading={isLoading}
      />

      {/* Messages */}
      <div className="flex-1 overflow-hidden">
        <MessageList
          messages={messageHistory}
          streamingMessage={streamingMessage}
          isLoading={isLoading}
          settings={settings}
        />
        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900">
        <MessageInput
          onSendMessage={handleSendMessage}
          isLoading={isLoading}
          placeholder="Ask me anything..."
          maxLength={4000}
        />
      </div>
    </div>
  );
};
