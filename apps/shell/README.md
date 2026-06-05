# Aetheris Shell Notification System

A comprehensive notification system with actionable buttons that feed back into the AI Core Service via ToolCall.

## Features

- **Multiple Notification Types**: Info, success, warning, error, and actionable notifications
- **Action Buttons**: Interactive buttons that can trigger AI Core tool calls
- **Event System**: Comprehensive event handling for notification lifecycle
- **Auto-dismissal**: Configurable duration-based auto-dismissal
- **Priority Support**: Low, normal, high, and urgent priority levels
- **Specialized Notifications**: Pre-built notifications for common scenarios
- **TypeScript Support**: Full type safety and IntelliSense support

## Quick Start

```typescript
import { initializeNotificationManager, notify } from '@aetheris/shell-notifications';
import { AICoreBridge } from './bridge/ai-core-bridge';

// Initialize the notification system
const aiBridge = new AICoreBridge();
const notificationManager = initializeNotificationManager(aiBridge);

// Create a simple notification
notify.info({
  title: 'Hello World',
  message: 'This is a test notification'
});

// Create an actionable notification
notify.actionable({
  title: 'Summary Ready',
  message: 'Your document summary is ready',
  actions: [
    {
      id: 'open_note',
      label: 'Open Note',
      toolCall: {
        toolName: 'open_note',
        parameters: { noteId: 'summary_123' }
      }
    }
  ]
});
```

## API Reference

### NotificationManager

The main class for managing notifications.

#### Methods

- `info(options)` - Create an info notification
- `success(options)` - Create a success notification
- `warn(options)` - Create a warning notification
- `error(options)` - Create an error notification
- `actionable(options)` - Create an actionable notification
- `dismiss(id)` - Dismiss a specific notification
- `dismissAll()` - Dismiss all notifications
- `getNotifications()` - Get all active notifications
- `getNotification(id)` - Get a specific notification

#### Specialized Methods

- `createSummaryReadyNotification(summaryId, title)` - Create summary ready notification
- `createFileOperationNotification(operation, fileName, success)` - Create file operation notification
- `createSystemUpdateNotification(updateType, details)` - Create system update notification
- `createBackupNotification(backupType, success, details?)` - Create backup notification

### NotificationOptions

```typescript
interface NotificationOptions {
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
```

### NotificationAction

```typescript
interface NotificationAction {
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
```

## Examples

### Summary Ready Notification

```typescript
// Create a summary ready notification with action buttons
const notificationId = notificationManager.createSummaryReadyNotification(
  'summary_meeting_notes',
  'Weekly Team Meeting - January 15, 2024'
);

// The notification includes these actions:
// - 📝 Open Note - Opens the summary in your note app
// - 📋 Copy Summary - Copies the summary to clipboard
// - ✕ Dismiss - Dismisses the notification
```

### File Operation Notification

```typescript
// Success notification
const successId = notificationManager.createFileOperationNotification(
  'save',
  'project_proposal.docx',
  true
);

// Error notification
const errorId = notificationManager.createFileOperationNotification(
  'upload',
  'large_video_file.mp4',
  false
);
```

### System Update Notification

```typescript
const updateId = notificationManager.createSystemUpdateNotification(
  'Security Update',
  'Critical security patches available for your system'
);

// Actions include:
// - View Update - Shows update details
// - Apply Update - Applies the update immediately
// - Schedule for Later - Schedules the update
```

### Custom Actionable Notification

```typescript
const customId = notificationManager.actionable({
  title: 'Custom Action',
  message: 'This notification has custom actions',
  actions: [
    {
      id: 'custom_action',
      label: 'Custom Action',
      toolCall: {
        toolName: 'custom_tool',
        parameters: { custom: 'value' }
      },
      style: 'primary',
      icon: '🎯'
    },
    {
      id: 'callback_action',
      label: 'Callback Action',
      callback: () => {
        console.log('Custom callback executed!');
      },
      style: 'secondary'
    }
  ]
});
```

## Event System

The notification system emits various events for monitoring and integration:

```typescript
// Listen for notification events
notificationManager.on('notification:show', (notification) => {
  console.log('Notification shown:', notification.title);
});

notificationManager.on('notification:dismiss', (notification) => {
  console.log('Notification dismissed:', notification.title);
});

notificationManager.on('notification:actioned', (notification, action) => {
  console.log('Action executed:', action.label);
});

// Listen for tool call events
notificationManager.on('tool:call-success', (toolName, parameters, result) => {
  console.log('Tool call successful:', toolName);
});

notificationManager.on('tool:call-error', (toolName, parameters, error) => {
  console.log('Tool call failed:', toolName, error);
});
```

## AI Core Integration

The notification system integrates with the AI Core Service through tool calls:

### Supported Tools

- `open_note` - Open a note or document
- `copy_to_clipboard` - Copy content to clipboard
- `open_file` - Open a file
- `show_in_folder` - Show file in folder
- `retry_file_operation` - Retry a failed file operation
- `show_error_details` - Show detailed error information
- `show_update_details` - Show update details
- `apply_system_update` - Apply system update
- `schedule_update` - Schedule update for later
- `show_backup_details` - Show backup details
- `restore_from_backup` - Restore from backup
- `retry_backup` - Retry failed backup
- `show_backup_error` - Show backup error details

### Tool Call Flow

1. User clicks action button in notification
2. Notification system calls `aiBridge.callTool(toolName, parameters)`
3. AI Core Service executes the tool
4. Result is returned and notification is dismissed
5. Events are emitted for monitoring

## Testing

Run the test suite:

```bash
npm test
```

Run tests with coverage:

```bash
npm run test:coverage
```

Run tests in watch mode:

```bash
npm run test:watch
```

## Development

Build the project:

```bash
npm run build
```

Run the example:

```bash
npm run example
```

Lint the code:

```bash
npm run lint
```

Format the code:

```bash
npm run format
```

## Architecture

The notification system consists of:

1. **NotificationManager** - Main notification management class
2. **AICoreBridge** - Bridge to AI Core Service for tool calls
3. **Event System** - Event-driven architecture for loose coupling
4. **Type System** - Full TypeScript support with comprehensive types

## Security

- All tool calls are validated and sanitized
- Notification actions are executed in a controlled environment
- Error handling prevents information leakage
- Capability tokens are validated for sensitive operations

## Performance

- Efficient notification storage and retrieval
- Automatic cleanup of expired notifications
- Configurable limits to prevent memory issues
- Optimized event handling for high-frequency scenarios

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

MIT License - see LICENSE file for details.
