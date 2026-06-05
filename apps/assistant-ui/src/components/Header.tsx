import React from 'react';
import { Settings, Settings as SettingsType } from '../types';
import { cn } from '../utils';

interface HeaderProps {
  currentView: 'chat' | 'settings';
  onViewChange: (view: 'chat' | 'settings') => void;
  isConnected: boolean;
  settings: SettingsType;
}

export const Header: React.FC<HeaderProps> = ({
  currentView,
  onViewChange,
  isConnected,
  settings
}) => {
  return (
    <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm">
      {/* Left side - Logo and title */}
      <div className="flex items-center space-x-3">
        <div className="w-8 h-8 bg-gradient-to-br from-primary-500 to-primary-600 rounded-lg flex items-center justify-center">
          <span className="text-white font-bold text-sm">A</span>
        </div>
        <div>
          <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
            Aetheris Assistant
          </h1>
          <div className="flex items-center space-x-2">
            <div className={cn(
              "w-2 h-2 rounded-full",
              isConnected ? "bg-green-500" : "bg-red-500"
            )} />
            <span className="text-xs text-gray-500 dark:text-gray-400">
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
          </div>
        </div>
      </div>

      {/* Right side - Navigation and actions */}
      <div className="flex items-center space-x-2">
        {/* View toggle buttons */}
        <div className="flex bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
          <button
            onClick={() => onViewChange('chat')}
            className={cn(
              "px-3 py-1.5 text-sm font-medium rounded-md transition-all duration-200",
              currentView === 'chat'
                ? "bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm"
                : "text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white"
            )}
          >
            Chat
          </button>
          <button
            onClick={() => onViewChange('settings')}
            className={cn(
              "px-3 py-1.5 text-sm font-medium rounded-md transition-all duration-200",
              currentView === 'settings'
                ? "bg-white dark:bg-gray-600 text-gray-900 dark:text-white shadow-sm"
                : "text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white"
            )}
          >
            Settings
          </button>
        </div>

        {/* Action buttons */}
        <div className="flex items-center space-x-1">
          {/* Minimize button */}
          <button
            onClick={() => {
              if (window.electronAPI) {
                window.electronAPI.hideApp();
              }
            }}
            className="p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors duration-200"
            title="Hide Assistant (Escape)"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
            </svg>
          </button>

          {/* Settings button */}
          <button
            onClick={() => onViewChange(currentView === 'chat' ? 'settings' : 'chat')}
            className="p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors duration-200"
            title="Settings (Cmd+,)"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
};
