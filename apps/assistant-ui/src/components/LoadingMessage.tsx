import React from 'react';
import { Settings } from '../types';

interface LoadingMessageProps {
  streamingMessage: string;
  settings: Settings;
}

export const LoadingMessage: React.FC<LoadingMessageProps> = ({
  streamingMessage,
  settings
}) => {
  return (
    <div className="flex items-start space-x-2">
      {/* Avatar */}
      <div className="w-8 h-8 bg-gradient-to-br from-primary-500 to-primary-600 rounded-full flex items-center justify-center flex-shrink-0">
        <span className="text-white font-bold text-sm">A</span>
      </div>

      {/* Message bubble */}
      <div className="bg-gray-100 dark:bg-gray-800 rounded-lg px-4 py-3 shadow-sm max-w-[80%]">
        {streamingMessage ? (
          <div className="prose prose-sm max-w-none dark:prose-invert">
            <p className="whitespace-pre-wrap">{streamingMessage}</p>
            <div className="inline-block w-2 h-4 bg-primary-600 animate-pulse ml-1" />
          </div>
        ) : (
          <div className="flex items-center space-x-2">
            <div className="flex space-x-1">
              <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
              <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
              <div className="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
            </div>
            <span className="text-sm text-gray-500 dark:text-gray-400">Thinking...</span>
          </div>
        )}
      </div>
    </div>
  );
};
