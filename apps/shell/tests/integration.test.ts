/**
 * @file integration.test.ts
 * @brief Integration tests for the notification system
 */

import { NotificationManager, initializeNotificationManager } from '../src/notify';
import { AICoreBridge } from '../src/bridge/ai-core-bridge';

describe('Notification System Integration', () => {
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

    describe('End-to-End Notification Flow', () => {
        test('should handle complete summary ready notification flow', async () => {
            // Create summary ready notification
            const notificationId = notificationManager.createSummaryReadyNotification(
                'summary_meeting_notes',
                'Weekly Team Meeting - January 15, 2024'
            );

            const notification = notificationManager.getNotification(notificationId);
            expect(notification).toBeDefined();
            expect(notification?.title).toBe('Summary Ready');

            // Simulate user clicking "Open Note" action
            const openNoteAction = notification?.actions?.find(a => a.id === 'open_note');
            expect(openNoteAction).toBeDefined();

            // Mock tool call execution
            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                noteId: 'summary_meeting_notes',
                action: 'open',
                message: 'Opened note: summary_meeting_notes'
            });

            // Execute the action
            notificationManager.emit('notification:action', notification!, openNoteAction!);

            // Wait for async execution
            await new Promise(resolve => setTimeout(resolve, 100));

            expect(mockCallTool).toHaveBeenCalledWith('open_note', {
                noteId: 'summary_meeting_notes',
                action: 'open'
            });

            // Verify notification is dismissed after action
            expect(notificationManager.getNotification(notificationId)).toBeUndefined();

            mockCallTool.mockRestore();
        });

        test('should handle file operation success flow', async () => {
            const notificationId = notificationManager.createFileOperationNotification(
                'save',
                'project_proposal.docx',
                true
            );

            const notification = notificationManager.getNotification(notificationId);
            expect(notification?.title).toBe('File Operation Complete');

            // Simulate user clicking "Open File" action
            const openFileAction = notification?.actions?.find(a => a.id === 'open_file');
            expect(openFileAction).toBeDefined();

            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                filePath: 'project_proposal.docx',
                action: 'open',
                message: 'Opened file: project_proposal.docx'
            });

            notificationManager.emit('notification:action', notification!, openFileAction!);

            await new Promise(resolve => setTimeout(resolve, 100));

            expect(mockCallTool).toHaveBeenCalledWith('open_file', {
                filePath: 'project_proposal.docx',
                action: 'open'
            });

            mockCallTool.mockRestore();
        });

        test('should handle file operation error flow', async () => {
            const notificationId = notificationManager.createFileOperationNotification(
                'upload',
                'large_video_file.mp4',
                false
            );

            const notification = notificationManager.getNotification(notificationId);
            expect(notification?.title).toBe('File Operation Failed');

            // Simulate user clicking "Retry" action
            const retryAction = notification?.actions?.find(a => a.id === 'retry');
            expect(retryAction).toBeDefined();

            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                operation: 'upload',
                fileName: 'large_video_file.mp4',
                message: 'Retrying upload on large_video_file.mp4'
            });

            notificationManager.emit('notification:action', notification!, retryAction!);

            await new Promise(resolve => setTimeout(resolve, 100));

            expect(mockCallTool).toHaveBeenCalledWith('retry_file_operation', {
                operation: 'upload',
                fileName: 'large_video_file.mp4'
            });

            mockCallTool.mockRestore();
        });

        test('should handle system update notification flow', async () => {
            const notificationId = notificationManager.createSystemUpdateNotification(
                'Security Update',
                'Critical security patches available for your system'
            );

            const notification = notificationManager.getNotification(notificationId);
            expect(notification?.title).toBe('System Update Available');

            // Simulate user clicking "Apply Update" action
            const applyUpdateAction = notification?.actions?.find(a => a.id === 'apply_update');
            expect(applyUpdateAction).toBeDefined();

            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                updateType: 'Security Update',
                message: 'System update applied successfully'
            });

            notificationManager.emit('notification:action', notification!, applyUpdateAction!);

            await new Promise(resolve => setTimeout(resolve, 100));

            expect(mockCallTool).toHaveBeenCalledWith('apply_system_update', {
                updateType: 'Security Update'
            });

            mockCallTool.mockRestore();
        });

        test('should handle backup notification flow', async () => {
            const notificationId = notificationManager.createBackupNotification(
                'Daily Backup',
                true,
                'Backup completed successfully. 2.3GB of data backed up.'
            );

            const notification = notificationManager.getNotification(notificationId);
            expect(notification?.title).toBe('Backup Complete');

            // Simulate user clicking "View Backup" action
            const viewBackupAction = notification?.actions?.find(a => a.id === 'view_backup');
            expect(viewBackupAction).toBeDefined();

            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                backupType: 'Daily Backup',
                message: 'Backup details displayed'
            });

            notificationManager.emit('notification:action', notification!, viewBackupAction!);

            await new Promise(resolve => setTimeout(resolve, 100));

            expect(mockCallTool).toHaveBeenCalledWith('show_backup_details', {
                backupType: 'Daily Backup'
            });

            mockCallTool.mockRestore();
        });
    });

    describe('Multiple Notification Scenarios', () => {
        test('should handle multiple notifications simultaneously', async () => {
            const notifications = [
                notificationManager.createSummaryReadyNotification('summary1', 'Summary 1'),
                notificationManager.createFileOperationNotification('save', 'file1.txt', true),
                notificationManager.createSystemUpdateNotification('Update 1', 'Details 1'),
                notificationManager.createBackupNotification('Backup 1', true),
                notificationManager.info({ title: 'Info 1', message: 'Message 1' })
            ];

            expect(notificationManager.getNotifications()).toHaveLength(5);

            // Execute actions on different notifications
            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                message: 'Action executed'
            });

            for (const notificationId of notifications) {
                const notification = notificationManager.getNotification(notificationId);
                if (notification?.actions && notification.actions.length > 0) {
                    const action = notification.actions[0];
                    notificationManager.emit('notification:action', notification, action);
                }
            }

            await new Promise(resolve => setTimeout(resolve, 200));

            // All notifications should be dismissed after actions
            expect(notificationManager.getNotifications().length).toBeLessThan(5);

            mockCallTool.mockRestore();
        });

        test('should handle notification priority correctly', () => {
            const lowPriority = notificationManager.info({
                title: 'Low Priority',
                message: 'This is low priority',
                priority: 'low'
            });

            const highPriority = notificationManager.error({
                title: 'High Priority',
                message: 'This is high priority',
                priority: 'high'
            });

            const urgentPriority = notificationManager.warn({
                title: 'Urgent Priority',
                message: 'This is urgent',
                priority: 'urgent'
            });

            const notifications = notificationManager.getNotifications();
            
            // Notifications should be sorted by priority (highest first)
            expect(notifications[0].priority).toBe('urgent');
            expect(notifications[1].priority).toBe('high');
            expect(notifications[2].priority).toBe('low');
        });
    });

    describe('Error Recovery', () => {
        test('should handle tool call failures gracefully', async () => {
            const notificationId = notificationManager.createSummaryReadyNotification(
                'summary_error_test',
                'Error Test Summary'
            );

            const notification = notificationManager.getNotification(notificationId);
            const action = notification?.actions?.[0];

            // Mock tool call to fail
            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockRejectedValue(
                new Error('Tool call failed')
            );

            const errorSpy = jest.fn();
            notificationManager.on('notification:action-error', errorSpy);

            notificationManager.emit('notification:action', notification!, action!);

            await new Promise(resolve => setTimeout(resolve, 100));

            expect(errorSpy).toHaveBeenCalled();
            expect(notificationManager.getNotification(notificationId)).toBeUndefined();

            mockCallTool.mockRestore();
        });

        test('should handle network disconnection', async () => {
            const notificationId = notificationManager.createSummaryReadyNotification(
                'summary_network_test',
                'Network Test Summary'
            );

            const notification = notificationManager.getNotification(notificationId);
            const action = notification?.actions?.[0];

            // Disconnect AI Core
            aiBridge.disconnect();

            const errorSpy = jest.fn();
            notificationManager.on('notification:action-error', errorSpy);

            notificationManager.emit('notification:action', notification!, action!);

            await new Promise(resolve => setTimeout(resolve, 100));

            expect(errorSpy).toHaveBeenCalled();
        });
    });

    describe('Performance and Scalability', () => {
        test('should handle large number of notifications efficiently', () => {
            const startTime = Date.now();
            const notificationCount = 100;

            for (let i = 0; i < notificationCount; i++) {
                notificationManager.info({
                    title: `Notification ${i}`,
                    message: `Message ${i}`
                });
            }

            const endTime = Date.now();
            const duration = endTime - startTime;

            expect(duration).toBeLessThan(1000); // Should complete within 1 second
            expect(notificationManager.getNotifications().length).toBeLessThanOrEqual(10); // Should respect limit
        });

        test('should handle rapid notification creation and dismissal', async () => {
            const notifications = [];

            // Create notifications rapidly
            for (let i = 0; i < 20; i++) {
                const id = notificationManager.info({
                    title: `Rapid ${i}`,
                    message: `Message ${i}`,
                    duration: 50 // Auto-dismiss quickly
                });
                notifications.push(id);
            }

            expect(notificationManager.getNotifications().length).toBeLessThanOrEqual(10);

            // Wait for auto-dismissal
            await new Promise(resolve => setTimeout(resolve, 100));

            expect(notificationManager.getNotifications().length).toBe(0);
        });

        test('should handle concurrent action executions', async () => {
            const notificationIds = [];

            // Create multiple notifications with actions
            for (let i = 0; i < 5; i++) {
                const id = notificationManager.createSummaryReadyNotification(
                    `summary_${i}`,
                    `Summary ${i}`
                );
                notificationIds.push(id);
            }

            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                message: 'Action executed'
            });

            // Execute actions concurrently
            const promises = notificationIds.map(id => {
                const notification = notificationManager.getNotification(id);
                const action = notification?.actions?.[0];
                if (action) {
                    notificationManager.emit('notification:action', notification!, action);
                }
                return Promise.resolve();
            });

            await Promise.all(promises);
            await new Promise(resolve => setTimeout(resolve, 200));

            expect(mockCallTool).toHaveBeenCalledTimes(5);
            mockCallTool.mockRestore();
        });
    });

    describe('Real-world Scenarios', () => {
        test('should handle typical user workflow', async () => {
            // 1. User creates a document
            const saveNotification = notificationManager.createFileOperationNotification(
                'save',
                'important_document.docx',
                true
            );

            // 2. User generates a summary
            const summaryNotification = notificationManager.createSummaryReadyNotification(
                'summary_document',
                'Document Summary: Important Document'
            );

            // 3. System update becomes available
            const updateNotification = notificationManager.createSystemUpdateNotification(
                'Feature Update',
                'New features available for your productivity tools'
            );

            // 4. Daily backup completes
            const backupNotification = notificationManager.createBackupNotification(
                'Daily Backup',
                true,
                'Backup completed successfully'
            );

            expect(notificationManager.getNotifications()).toHaveLength(4);

            // User interacts with notifications
            const mockCallTool = jest.spyOn(aiBridge, 'callTool').mockResolvedValue({
                success: true,
                message: 'Action executed'
            });

            // Open the saved document
            const saveNotif = notificationManager.getNotification(saveNotification);
            const openFileAction = saveNotif?.actions?.find(a => a.id === 'open_file');
            if (openFileAction) {
                notificationManager.emit('notification:action', saveNotif!, openFileAction);
            }

            // Open the summary
            const summaryNotif = notificationManager.getNotification(summaryNotification);
            const openNoteAction = summaryNotif?.actions?.find(a => a.id === 'open_note');
            if (openNoteAction) {
                notificationManager.emit('notification:action', summaryNotif!, openNoteAction);
            }

            // Schedule the update for later
            const updateNotif = notificationManager.getNotification(updateNotification);
            const scheduleAction = updateNotif?.actions?.find(a => a.id === 'schedule_update');
            if (scheduleAction) {
                notificationManager.emit('notification:action', updateNotif!, scheduleAction);
            }

            // View backup details
            const backupNotif = notificationManager.getNotification(backupNotification);
            const viewBackupAction = backupNotif?.actions?.find(a => a.id === 'view_backup');
            if (viewBackupAction) {
                notificationManager.emit('notification:action', backupNotif!, viewBackupAction);
            }

            await new Promise(resolve => setTimeout(resolve, 200));

            // All notifications should be dismissed after actions
            expect(notificationManager.getNotifications().length).toBe(0);
            expect(mockCallTool).toHaveBeenCalledTimes(4);

            mockCallTool.mockRestore();
        });
    });
});
