import React from 'react';
import { Settings as SettingsType } from '../types';
import { cn } from '../utils';

interface SettingsProps {
  settings: SettingsType;
  onSettingsChange: (settings: Partial<SettingsType>) => void;
  onClose: () => void;
}

export const Settings: React.FC<SettingsProps> = ({
  settings,
  onSettingsChange,
  onClose
}) => {
  return (
    <div className="flex-1 overflow-y-auto p-6 bg-white dark:bg-gray-900">
      <div className="max-w-2xl mx-auto space-y-8">
        {/* Header */}
        <div className="flex items-center justify-between">
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">Settings</h2>
          <button
            onClick={onClose}
            className="p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors duration-200"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Appearance */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Appearance</h3>
          
          {/* Theme */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Theme
            </label>
            <select
              value={settings.theme}
              onChange={(e) => onSettingsChange({ theme: e.target.value as 'light' | 'dark' | 'auto' })}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="light">Light</option>
              <option value="dark">Dark</option>
              <option value="auto">Auto (System)</option>
            </select>
          </div>

          {/* Font Size */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Font Size
            </label>
            <select
              value={settings.fontSize}
              onChange={(e) => onSettingsChange({ fontSize: e.target.value as 'small' | 'medium' | 'large' })}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="small">Small</option>
              <option value="medium">Medium</option>
              <option value="large">Large</option>
            </select>
          </div>
        </div>

        {/* Behavior */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Behavior</h3>
          
          {/* Auto Hide */}
          <div className="flex items-center justify-between">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Auto Hide
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Automatically hide the assistant when it loses focus
              </p>
            </div>
            <button
              onClick={() => onSettingsChange({ autoHide: !settings.autoHide })}
              className={cn(
                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200",
                settings.autoHide ? "bg-primary-600" : "bg-gray-200 dark:bg-gray-700"
              )}
            >
              <span
                className={cn(
                  "inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200",
                  settings.autoHide ? "translate-x-6" : "translate-x-1"
                )}
              />
            </button>
          </div>

          {/* Max History */}
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Max Message History
            </label>
            <input
              type="number"
              min="10"
              max="1000"
              value={settings.maxHistory}
              onChange={(e) => onSettingsChange({ maxHistory: parseInt(e.target.value) })}
              className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Maximum number of messages to keep in history
            </p>
          </div>
        </div>

        {/* Features */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Features</h3>
          
          {/* Streaming */}
          <div className="flex items-center justify-between">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Streaming Responses
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Show responses as they are generated
              </p>
            </div>
            <button
              onClick={() => onSettingsChange({ enableStreaming: !settings.enableStreaming })}
              className={cn(
                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200",
                settings.enableStreaming ? "bg-primary-600" : "bg-gray-200 dark:bg-gray-700"
              )}
            >
              <span
                className={cn(
                  "inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200",
                  settings.enableStreaming ? "translate-x-6" : "translate-x-1"
                )}
              />
            </button>
          </div>

          {/* Citations */}
          <div className="flex items-center justify-between">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Citations
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Show source citations in responses
              </p>
            </div>
            <button
              onClick={() => onSettingsChange({ enableCitations: !settings.enableCitations })}
              className={cn(
                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200",
                settings.enableCitations ? "bg-primary-600" : "bg-gray-200 dark:bg-gray-700"
              )}
            >
              <span
                className={cn(
                  "inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200",
                  settings.enableCitations ? "translate-x-6" : "translate-x-1"
                )}
              />
            </button>
          </div>

          {/* Tools */}
          <div className="flex items-center justify-between">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Tool Integration
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Allow the assistant to use tools and functions
              </p>
            </div>
            <button
              onClick={() => onSettingsChange({ enableTools: !settings.enableTools })}
              className={cn(
                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200",
                settings.enableTools ? "bg-primary-600" : "bg-gray-200 dark:bg-gray-700"
              )}
            >
              <span
                className={cn(
                  "inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200",
                  settings.enableTools ? "translate-x-6" : "translate-x-1"
                )}
              />
            </button>
          </div>

          {/* Memory */}
          <div className="flex items-center justify-between">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                Memory
              </label>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Remember context across conversations
              </p>
            </div>
            <button
              onClick={() => onSettingsChange({ enableMemory: !settings.enableMemory })}
              className={cn(
                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors duration-200",
                settings.enableMemory ? "bg-primary-600" : "bg-gray-200 dark:bg-gray-700"
              )}
            >
              <span
                className={cn(
                  "inline-block h-4 w-4 transform rounded-full bg-white transition-transform duration-200",
                  settings.enableMemory ? "translate-x-6" : "translate-x-1"
                )}
              />
            </button>
          </div>
        </div>

        {/* Keyboard Shortcuts */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">Keyboard Shortcuts</h3>
          
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-700 dark:text-gray-300">Toggle Assistant</span>
              <kbd className="px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded text-xs font-mono">
                {settings.hotkey}
              </kbd>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-700 dark:text-gray-300">Hide Assistant</span>
              <kbd className="px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded text-xs font-mono">
                Escape
              </kbd>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-700 dark:text-gray-300">New Chat</span>
              <kbd className="px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded text-xs font-mono">
                Cmd+N
              </kbd>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-700 dark:text-gray-300">Settings</span>
              <kbd className="px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded text-xs font-mono">
                Cmd+,
              </kbd>
            </div>
          </div>
        </div>

        {/* About */}
        <div className="space-y-4">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-white">About</h3>
          
          <div className="bg-gray-50 dark:bg-gray-800 rounded-lg p-4 space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-700 dark:text-gray-300">Version</span>
              <span className="text-sm text-gray-900 dark:text-gray-100">1.0.0</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-700 dark:text-gray-300">Platform</span>
              <span className="text-sm text-gray-900 dark:text-gray-100">
                {navigator.platform}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-700 dark:text-gray-300">AI Core</span>
              <span className="text-sm text-gray-900 dark:text-gray-100">Connected</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
