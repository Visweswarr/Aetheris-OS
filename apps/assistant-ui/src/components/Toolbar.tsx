import React from 'react';
import { cn } from '../utils';

interface ToolbarProps {
  onClearConversation: () => void;
  onCopyConversation: () => void;
  onExportConversation: () => void;
  messageCount: number;
  isLoading: boolean;
}

export const Toolbar: React.FC<ToolbarProps> = ({
  onClearConversation,
  onCopyConversation,
  onExportConversation,
  messageCount,
  isLoading
}) => {
  return (
    <div className="flex items-center justify-between p-3 border-b border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800">
      {/* Left side - Message count */}
      <div className="flex items-center space-x-2">
        <span className="text-sm text-gray-600 dark:text-gray-400">
          {messageCount} message{messageCount !== 1 ? 's' : ''}
        </span>
        {isLoading && (
          <div className="flex items-center space-x-1 text-sm text-primary-600 dark:text-primary-400">
            <div className="w-2 h-2 bg-primary-500 rounded-full animate-pulse" />
            <span>Processing...</span>
          </div>
        )}
      </div>

      {/* Right side - Action buttons */}
      <div className="flex items-center space-x-1">
        {/* Copy conversation */}
        <button
          onClick={onCopyConversation}
          disabled={messageCount === 0 || isLoading}
          className={cn(
            "p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200",
            "hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors duration-200",
            "disabled:opacity-50 disabled:cursor-not-allowed",
            "focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2"
          )}
          title="Copy conversation to clipboard"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
          </svg>
        </button>

        {/* Export conversation */}
        <button
          onClick={onExportConversation}
          disabled={messageCount === 0 || isLoading}
          className={cn(
            "p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200",
            "hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors duration-200",
            "disabled:opacity-50 disabled:cursor-not-allowed",
            "focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2"
          )}
          title="Export conversation as text file"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
        </button>

        {/* Clear conversation */}
        <button
          onClick={onClearConversation}
          disabled={messageCount === 0 || isLoading}
          className={cn(
            "p-2 text-gray-500 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400",
            "hover:bg-gray-200 dark:hover:bg-gray-700 rounded-lg transition-colors duration-200",
            "disabled:opacity-50 disabled:cursor-not-allowed",
            "focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2"
          )}
          title="Clear conversation"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      </div>
    </div>
  );
};
