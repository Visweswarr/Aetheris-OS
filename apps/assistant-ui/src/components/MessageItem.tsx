import React, { useState } from 'react';
import { ChatMessage, Settings } from '../types';
import { formatTimestamp, copyToClipboard, hasCode, extractCodeBlocks } from '../utils';
import { cn } from '../utils';

interface MessageItemProps {
  message: ChatMessage;
  settings: Settings;
}

export const MessageItem: React.FC<MessageItemProps> = ({ message, settings }) => {
  const [isCopied, setIsCopied] = useState(false);
  const [showCitations, setShowCitations] = useState(false);

  const handleCopy = async () => {
    const success = await copyToClipboard(message.content);
    if (success) {
      setIsCopied(true);
      setTimeout(() => setIsCopied(false), 2000);
    }
  };

  const handleCitationClick = (citationId: string) => {
    // Handle citation click - could open a modal or scroll to reference
    console.log('Citation clicked:', citationId);
  };

  const renderContent = () => {
    if (hasCode(message)) {
      return (
        <div className="space-y-2">
          {extractCodeBlocks(message).map((codeBlock, index) => (
            <div key={index} className="relative">
              <pre className="bg-gray-100 dark:bg-gray-800 rounded-lg p-3 overflow-x-auto text-sm">
                <code>{codeBlock}</code>
              </pre>
              <button
                onClick={() => copyToClipboard(codeBlock)}
                className="absolute top-2 right-2 p-1.5 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200 bg-white dark:bg-gray-700 rounded shadow-sm opacity-0 group-hover:opacity-100 transition-opacity"
                title="Copy code"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
              </button>
            </div>
          ))}
        </div>
      );
    }

    return (
      <div className="prose prose-sm max-w-none dark:prose-invert">
        <p className="whitespace-pre-wrap">{message.content}</p>
      </div>
    );
  };

  return (
    <div className={cn(
      "group flex flex-col space-y-2",
      message.role === 'user' ? 'items-end' : 'items-start'
    )}>
      {/* Message bubble */}
      <div className={cn(
        "max-w-[80%] rounded-lg px-4 py-3 shadow-sm",
        message.role === 'user'
          ? "bg-primary-600 text-white"
          : "bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-gray-100"
      )}>
        {renderContent()}
        
        {/* Citations */}
        {message.metadata?.citations && message.metadata.citations.length > 0 && (
          <div className="mt-3 pt-3 border-t border-gray-200 dark:border-gray-600">
            <button
              onClick={() => setShowCitations(!showCitations)}
              className="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 flex items-center space-x-1"
            >
              <span>{message.metadata.citations.length} citation{message.metadata.citations.length > 1 ? 's' : ''}</span>
              <svg 
                className={cn("w-3 h-3 transition-transform", showCitations && "rotate-180")} 
                fill="none" 
                stroke="currentColor" 
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
              </svg>
            </button>
            
            {showCitations && (
              <div className="mt-2 space-y-2">
                {message.metadata.citations.map((citation, index) => (
                  <div 
                    key={index}
                    className="p-2 bg-gray-50 dark:bg-gray-700 rounded text-xs cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-600"
                    onClick={() => handleCitationClick(citation)}
                  >
                    <div className="font-medium text-gray-900 dark:text-gray-100">
                      {citation}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
        
        {/* Tools used */}
        {message.metadata?.tools_used && message.metadata.tools_used.length > 0 && (
          <div className="mt-2 flex flex-wrap gap-1">
            {message.metadata.tools_used.map((tool, index) => (
              <span
                key={index}
                className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200"
              >
                🔧 {tool}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Message metadata */}
      <div className="flex items-center space-x-2 text-xs text-gray-500 dark:text-gray-400">
        <span>{formatTimestamp(message.timestamp)}</span>
        
        {message.metadata?.processing_time_ms && (
          <span>• {message.metadata.processing_time_ms}ms</span>
        )}
        
        {message.metadata?.confidence && (
          <span>• {Math.round(message.metadata.confidence * 100)}% confidence</span>
        )}
        
        <button
          onClick={handleCopy}
          className={cn(
            "opacity-0 group-hover:opacity-100 transition-opacity p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded",
            isCopied && "opacity-100"
          )}
          title="Copy message"
        >
          {isCopied ? (
            <svg className="w-3 h-3 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
          ) : (
            <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          )}
        </button>
      </div>
    </div>
  );
};
