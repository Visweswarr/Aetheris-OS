import React from 'react';
import { ChatMessage, Settings } from '../types';
import { MessageItem } from './MessageItem';
import { LoadingMessage } from './LoadingMessage';

interface MessageListProps {
  messages: ChatMessage[];
  streamingMessage: string;
  isLoading: boolean;
  settings: Settings;
}

export const MessageList: React.FC<MessageListProps> = ({
  messages,
  streamingMessage,
  isLoading,
  settings
}) => {
  if (messages.length === 0 && !isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center p-8">
        <div className="text-center max-w-md">
          <div className="w-16 h-16 bg-gradient-to-br from-primary-500 to-primary-600 rounded-full flex items-center justify-center mx-auto mb-4">
            <span className="text-white font-bold text-xl">A</span>
          </div>
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
            Welcome to Aetheris Assistant
          </h3>
          <p className="text-gray-600 dark:text-gray-400 mb-6">
            I'm here to help you with questions, tasks, and conversations. 
            What would you like to know?
          </p>
          <div className="grid grid-cols-1 gap-3">
            <div className="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg text-sm text-gray-700 dark:text-gray-300">
              💡 <strong>Tip:</strong> Press <kbd className="px-1.5 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs">Cmd+,</kbd> to open settings
            </div>
            <div className="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg text-sm text-gray-700 dark:text-gray-300">
              ⌨️ <strong>Shortcut:</strong> Press <kbd className="px-1.5 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-xs">Escape</kbd> to hide the assistant
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto p-4 space-y-4">
      {messages.map((message) => (
        <MessageItem
          key={message.id}
          message={message}
          settings={settings}
        />
      ))}
      
      {isLoading && (
        <LoadingMessage
          streamingMessage={streamingMessage}
          settings={settings}
        />
      )}
    </div>
  );
};
