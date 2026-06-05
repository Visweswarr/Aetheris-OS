/**
 * @file notify.ts
 * @brief Aetheris Shell Notification System
 * 
 * This module provides a comprehensive notification system with actionable buttons
 * that can feed back into the AI Core Service via ToolCall.
 */

import { EventEmitter } from 'events';
import { AICoreBridge } from '../bridge/ai-core-bridge';

export interface NotificationAction {
  id: string;
  label: string;
  toolCall?: {
    toolName: string;
    parameters: any;
  };
  callback?: () => void | Promise<void>;
  style?: 'primary' | 'secondary' | 'danger';
  icon?: string;
}

export interface NotificationOptions {
  id?: string;
  title: string;
  message?: string;
  type: 'info' | 'success' | 'warning' | 'error';
  duration?: number; // 0 = persistent
  actions?: NotificationAction[];
  icon?: string;
  sound?: boolean;
  priority?: 'low' | 'normal' | 'high' | 'urgent';
  category?: string;
  metadata?: Record<string, any>;
}

export interface Notification extends NotificationOptions {
  id: string;
  timestamp: Date;
  dismissed: boolean;
  actioned: boolean;
}

export class NotificationManager extends EventEmitter {
  private notifications: Map<string, Notification> = new Map();
  private aiBridge: AICoreBridge;
  private maxNotifications: number = 10;
  private defaultDuration: number = 5000;

  constructor(aiBridge: AICoreBridge) {
    super();
    this.aiBridge = aiBridge;
    this.setupEventHandlers();
  }

  private setupEventHandlers(): void {
    // Handle notification actions
    this.on('notification:action', this.handleNotificationAction.bind(this));
    
    // Handle notification dismissal
    this.on('notification:dismiss', this.handleNotificationDismiss.bind(this));
  }

  /**
   * Show an info notification
   */
  info(options: Omit<NotificationOptions, 'type'>): string {
    return this.show({ ...options, type: 'info' });
  }

  /**
   * Show a success notification
   */
  success(options: Omit<NotificationOptions, 'type'>): string {
    return this.show({ ...options, type: 'success' });
  }

  /**
   * Show a warning notification
   */
  warn(options: Omit<NotificationOptions, 'type'>): string {
    return this.show({ ...options, type: 'warning' });
  }

  /**
   * Show an error notification
   */
  error(options: Omit<NotificationOptions, 'type'>): string {
    return this.show({ ...options, type: 'error' });
  }

  /**
   * Show an actionable notification with buttons that can trigger AI Core tool calls
   */
  actionable(options: Omit<NotificationOptions, 'type'>): string {
    return this.show({ ...options, type: 'info' });
  }

  /**
   * Show a notification
   */
  show(options: NotificationOptions): string {
    const id = options.id || this.generateId();
    const notification: Notification = {
      ...options,
      id,
      timestamp: new Date(),
      dismissed: false,
      actioned: false,
      duration: options.duration ?? this.defaultDuration,
    };

    // Store notification
    this.notifications.set(id, notification);

    // Emit notification event
    this.emit('notification:show', notification);

    // Auto-dismiss if duration is set
    if (notification.duration > 0) {
      setTimeout(() => {
        this.dismiss(id);
      }, notification.duration);
    }

    // Play sound if enabled
    if (notification.sound) {
      this.playNotificationSound(notification.type);
    }

    // Clean up old notifications
    this.cleanupOldNotifications();

    return id;
  }

  /**
   * Dismiss a notification
   */
  dismiss(id: string): void {
    const notification = this.notifications.get(id);
    if (notification && !notification.dismissed) {
      notification.dismissed = true;
      this.emit('notification:dismiss', notification);
      this.notifications.delete(id);
    }
  }

  /**
   * Dismiss all notifications
   */
  dismissAll(): void {
    for (const [id] of this.notifications) {
      this.dismiss(id);
    }
  }

  /**
   * Get all active notifications
   */
  getNotifications(): Notification[] {
    return Array.from(this.notifications.values())
      .filter(n => !n.dismissed)
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
  }

  /**
   * Get notification by ID
   */
  getNotification(id: string): Notification | undefined {
    return this.notifications.get(id);
  }

  /**
   * Handle notification action
   */
  private async handleNotificationAction(notification: Notification, action: NotificationAction): Promise<void> {
    try {
      // Mark notification as actioned
      notification.actioned = true;

      // Execute tool call if specified
      if (action.toolCall) {
        await this.executeToolCall(action.toolCall.toolName, action.toolCall.parameters);
      }

      // Execute callback if specified
      if (action.callback) {
        await action.callback();
      }

      // Emit action event
      this.emit('notification:actioned', notification, action);

      // Dismiss notification after action
      this.dismiss(notification.id);

    } catch (error) {
      console.error('Error handling notification action:', error);
      this.emit('notification:action-error', notification, action, error);
    }
  }

  /**
   * Handle notification dismissal
   */
  private handleNotificationDismiss(notification: Notification): void {
    // Emit dismiss event
    this.emit('notification:dismissed', notification);
  }

  /**
   * Execute tool call via AI Core Service
   */
  private async executeToolCall(toolName: string, parameters: any): Promise<any> {
    try {
      const result = await this.aiBridge.callTool(toolName, parameters);
      this.emit('tool:call-success', toolName, parameters, result);
      return result;
    } catch (error) {
      this.emit('tool:call-error', toolName, parameters, error);
      throw error;
    }
  }

  /**
   * Play notification sound
   */
  private playNotificationSound(type: string): void {
    // In a real implementation, this would play system sounds
    // For now, we'll just emit an event
    this.emit('notification:sound', type);
  }

  /**
   * Clean up old notifications
   */
  private cleanupOldNotifications(): void {
    const notifications = Array.from(this.notifications.values());
    if (notifications.length > this.maxNotifications) {
      // Remove oldest notifications
      const sorted = notifications.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());
      const toRemove = sorted.slice(0, notifications.length - this.maxNotifications);
      
      for (const notification of toRemove) {
        this.dismiss(notification.id);
      }
    }
  }

  /**
   * Generate unique ID
   */
  private generateId(): string {
    return `notif_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Create a summary ready notification with "Open note" action
   */
  createSummaryReadyNotification(summaryId: string, summaryTitle: string): string {
    return this.actionable({
      title: 'Summary Ready',
      message: `"${summaryTitle}" has been generated and is ready to view.`,
      type: 'success',
      duration: 0, // Persistent until actioned
      priority: 'normal',
      category: 'summary',
      metadata: {
        summaryId,
        summaryTitle,
      },
      actions: [
        {
          id: 'open_note',
          label: 'Open Note',
          toolCall: {
            toolName: 'open_note',
            parameters: {
              noteId: summaryId,
              action: 'open'
            }
          },
          style: 'primary',
          icon: '📝'
        },
        {
          id: 'copy_summary',
          label: 'Copy Summary',
          toolCall: {
            toolName: 'copy_to_clipboard',
            parameters: {
              content: `Summary: ${summaryTitle}`,
              format: 'text'
            }
          },
          style: 'secondary',
          icon: '📋'
        },
        {
          id: 'dismiss',
          label: 'Dismiss',
          callback: () => {
            // Just dismiss, no tool call needed
          },
          style: 'secondary',
          icon: '✕'
        }
      ]
    });
  }

  /**
   * Create a file operation notification
   */
  createFileOperationNotification(operation: string, fileName: string, success: boolean): string {
    const type = success ? 'success' : 'error';
    const title = success ? 'File Operation Complete' : 'File Operation Failed';
    const message = `${operation} "${fileName}" ${success ? 'completed successfully' : 'failed'}.`;

    return this.actionable({
      title,
      message,
      type,
      duration: success ? 5000 : 0, // Persistent for errors
      priority: success ? 'normal' : 'high',
      category: 'file-operation',
      metadata: {
        operation,
        fileName,
        success,
      },
      actions: success ? [
        {
          id: 'open_file',
          label: 'Open File',
          toolCall: {
            toolName: 'open_file',
            parameters: {
              filePath: fileName,
              action: 'open'
            }
          },
          style: 'primary',
          icon: '📂'
        },
        {
          id: 'show_in_folder',
          label: 'Show in Folder',
          toolCall: {
            toolName: 'show_in_folder',
            parameters: {
              filePath: fileName
            }
          },
          style: 'secondary',
          icon: '📁'
        }
      ] : [
        {
          id: 'retry',
          label: 'Retry',
          toolCall: {
            toolName: 'retry_file_operation',
            parameters: {
              operation,
              fileName
            }
          },
          style: 'primary',
          icon: '🔄'
        },
        {
          id: 'view_error',
          label: 'View Error',
          toolCall: {
            toolName: 'show_error_details',
            parameters: {
              operation,
              fileName
            }
          },
          style: 'secondary',
          icon: '⚠️'
        }
      ]
    });
  }

  /**
   * Create a system update notification
   */
  createSystemUpdateNotification(updateType: string, details: string): string {
    return this.actionable({
      title: 'System Update Available',
      message: `${updateType}: ${details}`,
      type: 'info',
      duration: 0, // Persistent
      priority: 'high',
      category: 'system-update',
      metadata: {
        updateType,
        details,
      },
      actions: [
        {
          id: 'view_update',
          label: 'View Update',
          toolCall: {
            toolName: 'show_update_details',
            parameters: {
              updateType,
              details
            }
          },
          style: 'primary',
          icon: '📋'
        },
        {
          id: 'apply_update',
          label: 'Apply Update',
          toolCall: {
            toolName: 'apply_system_update',
            parameters: {
              updateType
            }
          },
          style: 'primary',
          icon: '🔄'
        },
        {
          id: 'schedule_update',
          label: 'Schedule for Later',
          toolCall: {
            toolName: 'schedule_update',
            parameters: {
              updateType,
              scheduleTime: 'later'
            }
          },
          style: 'secondary',
          icon: '⏰'
        }
      ]
    });
  }

  /**
   * Create a backup notification
   */
  createBackupNotification(backupType: string, success: boolean, details?: string): string {
    const type = success ? 'success' : 'error';
    const title = success ? 'Backup Complete' : 'Backup Failed';
    const message = success 
      ? `${backupType} backup completed successfully.`
      : `${backupType} backup failed${details ? `: ${details}` : '.'}`;

    return this.actionable({
      title,
      message,
      type,
      duration: success ? 5000 : 0,
      priority: success ? 'normal' : 'high',
      category: 'backup',
      metadata: {
        backupType,
        success,
        details,
      },
      actions: success ? [
        {
          id: 'view_backup',
          label: 'View Backup',
          toolCall: {
            toolName: 'show_backup_details',
            parameters: {
              backupType
            }
          },
          style: 'primary',
          icon: '📦'
        },
        {
          id: 'restore_backup',
          label: 'Restore from Backup',
          toolCall: {
            toolName: 'restore_from_backup',
            parameters: {
              backupType
            }
          },
          style: 'secondary',
          icon: '🔄'
        }
      ] : [
        {
          id: 'retry_backup',
          label: 'Retry Backup',
          toolCall: {
            toolName: 'retry_backup',
            parameters: {
              backupType
            }
          },
          style: 'primary',
          icon: '🔄'
        },
        {
          id: 'view_error',
          label: 'View Error',
          toolCall: {
            toolName: 'show_backup_error',
            parameters: {
              backupType,
              details
            }
          },
          style: 'secondary',
          icon: '⚠️'
        }
      ]
    });
  }

  /**
   * Cleanup resources
   */
  cleanup(): void {
    this.dismissAll();
    this.removeAllListeners();
  }
}

// Export convenience functions
export const notify = {
  info: (options: Omit<NotificationOptions, 'type'>) => notificationManager.info(options),
  success: (options: Omit<NotificationOptions, 'type'>) => notificationManager.success(options),
  warn: (options: Omit<NotificationOptions, 'type'>) => notificationManager.warn(options),
  error: (options: Omit<NotificationOptions, 'type'>) => notificationManager.error(options),
  actionable: (options: Omit<NotificationOptions, 'type'>) => notificationManager.actionable(options),
  dismiss: (id: string) => notificationManager.dismiss(id),
  dismissAll: () => notificationManager.dismissAll(),
  getNotifications: () => notificationManager.getNotifications(),
  getNotification: (id: string) => notificationManager.getNotification(id),
};

// Create global notification manager instance
// In a real implementation, this would be injected as a dependency
let notificationManager: NotificationManager;

export function initializeNotificationManager(aiBridge: AICoreBridge): NotificationManager {
  notificationManager = new NotificationManager(aiBridge);
  return notificationManager;
}

export function getNotificationManager(): NotificationManager {
  if (!notificationManager) {
    throw new Error('NotificationManager not initialized. Call initializeNotificationManager first.');
  }
  return notificationManager;
}
