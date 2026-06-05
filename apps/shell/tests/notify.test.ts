/**
 * @file notify.test.ts
 * @brief Comprehensive tests for the notification system
 */

import { NotificationManager, initializeNotificationManager } from '../src/notify';
import { AICoreBridge } from '../src/bridge/ai-core-bridge';

describe('NotificationManager', () => {
    let notificationManager: NotificationManager;
    let aiBridge: AICoreBridge;

    beforeEach(() => {
        aiBridge = new AICoreBridge();
        notificationManager = initializeNotificationManager(aiBridge);
    });

    afterEach(() => {
        notificationManager.cleanup();
        aiBridge.cleanup();
    });

    describe('Basic Notification Creation', () => {
        test('should create info notification', () => {
            const id = notificationManager.info({
                title: 'Test Info',
                message: 'This is a test info notification'
            });

            expect(id).toBeDefined();
            expect(typeof id).toBe('string');

            const notification = notificationManager.getNotification(id);
            expect(notification).toBeDefined();
            expect(notification?.title).toBe('Test Info');
            expect(notification?.message).toBe('This is a test info notification');
            expect(notification?.type).toBe('info');
        });

        test('should create success notification', () => {
            const id = notificationManager.success({
                title: 'Test Success',
                message: 'This is a test success notification'
            });

            const notification = notificationManager.getNotification(id);
            expect(notification?.type).toBe('success');
        });

        test('should create warning notification', () => {
            const id = notificationManager.warn({
                title: 'Test Warning',
                message: 'This is a test warning notification'
            });

            const notification = notificationManager.getNotification(id);
            expect(notification?.type).toBe('warning');
        });

        test('should create error notification', () => {
            const id = notificationManager.error({
                title: 'Test Error',
                message: 'This is a test error notification'
            });

            const notification = notificationManager.getNotification(id);
            expect(notification?.type).toBe('error');
        });

        test('should create actionable notification', () => {
            const id = notificationManager.actionable({
                title: 'Test Actionable',
                message: 'This is a test actionable notification',
                actions: [
                    {
                        id: 'test_action',
                        label: 'Test Action',
                        toolCall: {
                            toolName: 'test_tool',
                            parameters: { test: 'value' }
                        }
                    }
                ]
            });

            const notification = notificationManager.getNotification(id);
            expect(notification?.actions).toHaveLength(1);
            expect(notification?.actions?.[0].id).toBe('test_action');
        });
    });

    describe('Notification Actions', () => {
        test('should execute tool call action', async () => {
            const id = notificationManager.actionable({
                title: 'Test Action',
                message: 'Test message',
                actions: [
                    {
                        id: 'test_action',
                        label: 'Test Action',
                        toolCall: {
                            toolName: 'open_note',
                            parameters: { noteId: 'test_note' }
                        }
                    }
                ]
            });

            const notification = notificationManager.getNotification(id);
            expect(notification).toBeDefined();

            const action = notification?.actions?.[0];
            expect(action).toBeDefined();

            // Mock the tool call execution
            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                noteId: 'test_note',
                action: 'open'
            });

            // Execute the action
            notificationManager.emit('notification:action', notification!, action!);

            // Wait for async execution
            await new Promise(resolve => setTimeout(resolve, 100));

            expect(mockCallTool).toHaveBeenCalledWith('open_note', { noteId: 'test_note' });
            mockCallTool.mockRestore();
        });

        test('should execute callback action', async () => {
            const mockCallback = jest.fn();
            
            const id = notificationManager.actionable({
                title: 'Test Callback',
                message: 'Test message',
                actions: [
                    {
                        id: 'callback_action',
                        label: 'Callback Action',
                        callback: mockCallback
                    }
                ]
            });

            const notification = notificationManager.getNotification(id);
            const action = notification?.actions?.[0];

            // Execute the action
            notificationManager.emit('notification:action', notification!, action!);

            // Wait for async execution
            await new Promise(resolve => setTimeout(resolve, 100));

            expect(mockCallback).toHaveBeenCalled();
        });

        test('should handle action errors gracefully', async () => {
            const id = notificationManager.actionable({
                title: 'Test Error',
                message: 'Test message',
                actions: [
                    {
                        id: 'error_action',
                        label: 'Error Action',
                        toolCall: {
                            toolName: 'failing_tool',
                            parameters: {}
                        }
                    }
                ]
            });

            const notification = notificationManager.getNotification(id);
            const action = notification?.actions?.[0];

            // Mock the tool call to fail
            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockRejectedValue(
                new Error('Tool call failed')
            );

            const errorSpy = jest.fn();
            notificationManager.on('notification:action-error', errorSpy);

            // Execute the action
            notificationManager.emit('notification:action', notification!, action!);

            // Wait for async execution
            await new Promise(resolve => setTimeout(resolve, 100));

            expect(errorSpy).toHaveBeenCalled();
            mockCallTool.mockRestore();
        });
    });

    describe('Notification Management', () => {
        test('should dismiss notification', () => {
            const id = notificationManager.info({
                title: 'Test Dismiss',
                message: 'This notification will be dismissed'
            });

            expect(notificationManager.getNotification(id)).toBeDefined();

            notificationManager.dismiss(id);

            expect(notificationManager.getNotification(id)).toBeUndefined();
        });

        test('should dismiss all notifications', () => {
            notificationManager.info({ title: 'Test 1', message: 'Message 1' });
            notificationManager.info({ title: 'Test 2', message: 'Message 2' });
            notificationManager.info({ title: 'Test 3', message: 'Message 3' });

            expect(notificationManager.getNotifications()).toHaveLength(3);

            notificationManager.dismissAll();

            expect(notificationManager.getNotifications()).toHaveLength(0);
        });

        test('should auto-dismiss after duration', async () => {
            const id = notificationManager.info({
                title: 'Test Auto Dismiss',
                message: 'This will auto-dismiss',
                duration: 100 // 100ms
            });

            expect(notificationManager.getNotification(id)).toBeDefined();

            // Wait for auto-dismiss
            await new Promise(resolve => setTimeout(resolve, 150));

            expect(notificationManager.getNotification(id)).toBeUndefined();
        });

        test('should not auto-dismiss persistent notifications', async () => {
            const id = notificationManager.info({
                title: 'Test Persistent',
                message: 'This will not auto-dismiss',
                duration: 0 // Persistent
            });

            expect(notificationManager.getNotification(id)).toBeDefined();

            // Wait longer than normal duration
            await new Promise(resolve => setTimeout(resolve, 200));

            expect(notificationManager.getNotification(id)).toBeDefined();
        });
    });

    describe('Specialized Notifications', () => {
        test('should create summary ready notification', () => {
            const id = notificationManager.createSummaryReadyNotification(
                'summary_123',
                'Test Summary Title'
            );

            const notification = notificationManager.getNotification(id);
            expect(notification?.title).toBe('Summary Ready');
            expect(notification?.type).toBe('success');
            expect(notification?.actions).toHaveLength(3);
            
            const actionIds = notification?.actions?.map(a => a.id);
            expect(actionIds).toContain('open_note');
            expect(actionIds).toContain('copy_summary');
            expect(actionIds).toContain('dismiss');
        });

        test('should create file operation notification (success)', () => {
            const id = notificationManager.createFileOperationNotification(
                'save',
                'test_file.txt',
                true
            );

            const notification = notificationManager.getNotification(id);
            expect(notification?.title).toBe('File Operation Complete');
            expect(notification?.type).toBe('success');
            expect(notification?.actions).toHaveLength(2);
            
            const actionIds = notification?.actions?.map(a => a.id);
            expect(actionIds).toContain('open_file');
            expect(actionIds).toContain('show_in_folder');
        });

        test('should create file operation notification (error)', () => {
            const id = notificationManager.createFileOperationNotification(
                'upload',
                'large_file.zip',
                false
            );

            const notification = notificationManager.getNotification(id);
            expect(notification?.title).toBe('File Operation Failed');
            expect(notification?.type).toBe('error');
            expect(notification?.actions).toHaveLength(2);
            
            const actionIds = notification?.actions?.map(a => a.id);
            expect(actionIds).toContain('retry');
            expect(actionIds).toContain('view_error');
        });

        test('should create system update notification', () => {
            const id = notificationManager.createSystemUpdateNotification(
                'Security Update',
                'Critical security patches available'
            );

            const notification = notificationManager.getNotification(id);
            expect(notification?.title).toBe('System Update Available');
            expect(notification?.type).toBe('info');
            expect(notification?.actions).toHaveLength(3);
            
            const actionIds = notification?.actions?.map(a => a.id);
            expect(actionIds).toContain('view_update');
            expect(actionIds).toContain('apply_update');
            expect(actionIds).toContain('schedule_update');
        });

        test('should create backup notification (success)', () => {
            const id = notificationManager.createBackupNotification(
                'Daily Backup',
                true,
                'Backup completed successfully'
            );

            const notification = notificationManager.getNotification(id);
            expect(notification?.title).toBe('Backup Complete');
            expect(notification?.type).toBe('success');
            expect(notification?.actions).toHaveLength(2);
            
            const actionIds = notification?.actions?.map(a => a.id);
            expect(actionIds).toContain('view_backup');
            expect(actionIds).toContain('restore_backup');
        });

        test('should create backup notification (error)', () => {
            const id = notificationManager.createBackupNotification(
                'Daily Backup',
                false,
                'Backup failed due to insufficient space'
            );

            const notification = notificationManager.getNotification(id);
            expect(notification?.title).toBe('Backup Failed');
            expect(notification?.type).toBe('error');
            expect(notification?.actions).toHaveLength(2);
            
            const actionIds = notification?.actions?.map(a => a.id);
            expect(actionIds).toContain('retry_backup');
            expect(actionIds).toContain('view_error');
        });
    });

    describe('Event System', () => {
        test('should emit notification:show event', () => {
            const showSpy = jest.fn();
            notificationManager.on('notification:show', showSpy);

            notificationManager.info({
                title: 'Test Event',
                message: 'Testing event emission'
            });

            expect(showSpy).toHaveBeenCalled();
        });

        test('should emit notification:dismiss event', () => {
            const dismissSpy = jest.fn();
            notificationManager.on('notification:dismiss', dismissSpy);

            const id = notificationManager.info({
                title: 'Test Dismiss Event',
                message: 'Testing dismiss event'
            });

            notificationManager.dismiss(id);

            expect(dismissSpy).toHaveBeenCalled();
        });

        test('should emit notification:actioned event', async () => {
            const actionedSpy = jest.fn();
            notificationManager.on('notification:actioned', actionedSpy);

            const id = notificationManager.actionable({
                title: 'Test Actioned Event',
                message: 'Testing actioned event',
                actions: [
                    {
                        id: 'test_action',
                        label: 'Test Action',
                        callback: jest.fn()
                    }
                ]
            });

            const notification = notificationManager.getNotification(id);
            const action = notification?.actions?.[0];

            notificationManager.emit('notification:action', notification!, action!);

            await new Promise(resolve => setTimeout(resolve, 100));

            expect(actionedSpy).toHaveBeenCalled();
        });

        test('should emit tool:call-success event', async () => {
            const successSpy = jest.fn();
            notificationManager.on('tool:call-success', successSpy);

            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                result: 'test result'
            });

            const id = notificationManager.actionable({
                title: 'Test Tool Success',
                message: 'Testing tool success event',
                actions: [
                    {
                        id: 'test_tool',
                        label: 'Test Tool',
                        toolCall: {
                            toolName: 'test_tool',
                            parameters: {}
                        }
                    }
                ]
            });

            const notification = notificationManager.getNotification(id);
            const action = notification?.actions?.[0];

            notificationManager.emit('notification:action', notification!, action!);

            await new Promise(resolve => setTimeout(resolve, 100));

            expect(successSpy).toHaveBeenCalled();
            mockCallTool.mockRestore();
        });

        test('should emit tool:call-error event', async () => {
            const errorSpy = jest.fn();
            notificationManager.on('tool:call-error', errorSpy);

            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockRejectedValue(
                new Error('Tool call failed')
            );

            const id = notificationManager.actionable({
                title: 'Test Tool Error',
                message: 'Testing tool error event',
                actions: [
                    {
                        id: 'test_tool',
                        label: 'Test Tool',
                        toolCall: {
                            toolName: 'failing_tool',
                            parameters: {}
                        }
                    }
                ]
            });

            const notification = notificationManager.getNotification(id);
            const action = notification?.actions?.[0];

            notificationManager.emit('notification:action', notification!, action!);

            await new Promise(resolve => setTimeout(resolve, 100));

            expect(errorSpy).toHaveBeenCalled();
            mockCallTool.mockRestore();
        });
    });

    describe('Notification Limits and Cleanup', () => {
        test('should limit maximum notifications', () => {
            // Create more notifications than the limit
            const maxNotifications = 10;
            const excessNotifications = 5;

            for (let i = 0; i < maxNotifications + excessNotifications; i++) {
                notificationManager.info({
                    title: `Test ${i}`,
                    message: `Message ${i}`
                });
            }

            const notifications = notificationManager.getNotifications();
            expect(notifications.length).toBeLessThanOrEqual(maxNotifications);
        });

        test('should cleanup expired notifications', async () => {
            // Create notifications with different durations
            notificationManager.info({
                title: 'Short Duration',
                message: 'This will expire quickly',
                duration: 50
            });

            notificationManager.info({
                title: 'Long Duration',
                message: 'This will last longer',
                duration: 200
            });

            expect(notificationManager.getNotifications()).toHaveLength(2);

            // Wait for first notification to expire
            await new Promise(resolve => setTimeout(resolve, 100));

            // The cleanup should have removed the expired notification
            const notifications = notificationManager.getNotifications();
            expect(notifications.length).toBeLessThan(2);
        });
    });

    describe('Convenience Functions', () => {
        test('should provide convenience functions', () => {
            const { notify } = require('../src/notify');

            expect(typeof notify.info).toBe('function');
            expect(typeof notify.success).toBe('function');
            expect(typeof notify.warn).toBe('function');
            expect(typeof notify.error).toBe('function');
            expect(typeof notify.actionable).toBe('function');
            expect(typeof notify.dismiss).toBe('function');
            expect(typeof notify.dismissAll).toBe('function');
            expect(typeof notify.getNotifications).toBe('function');
            expect(typeof notify.getNotification).toBe('function');
        });

        test('should work with convenience functions', () => {
            const { notify } = require('../src/notify');

            const id = notify.info({
                title: 'Convenience Test',
                message: 'Testing convenience function'
            });

            expect(id).toBeDefined();
            expect(notify.getNotification(id)).toBeDefined();
        });
    });
});
