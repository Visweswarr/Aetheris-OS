/**
 * @file ai-core-bridge.test.ts
 * @brief Tests for AI Core Bridge
 */

import { AICoreBridge } from '../src/bridge/ai-core-bridge';

describe('AICoreBridge', () => {
    let aiBridge: AICoreBridge;

    beforeEach(() => {
        aiBridge = new AICoreBridge();
    });

    afterEach(() => {
        aiBridge.cleanup();
    });

    describe('Connection Management', () => {
        test('should initialize with connection', () => {
            expect(aiBridge.isAICoreConnected()).toBe(true);
        });

        test('should emit connected event on initialization', (done) => {
            const newBridge = new AICoreBridge();
            
            newBridge.on('connected', () => {
                expect(true).toBe(true);
                newBridge.cleanup();
                done();
            });
        });

        test('should disconnect and emit disconnected event', (done) => {
            aiBridge.on('disconnected', () => {
                expect(aiBridge.isAICoreConnected()).toBe(false);
                done();
            });

            aiBridge.disconnect();
        });
    });

    describe('Tool Call Execution', () => {
        test('should execute open_note tool call', async () => {
            const result = await aiBridge.callTool('open_note', {
                noteId: 'test_note_123',
                action: 'open'
            });

            expect(result).toEqual({
                success: true,
                noteId: 'test_note_123',
                action: 'open',
                message: 'Opened note: test_note_123'
            });
        });

        test('should execute copy_to_clipboard tool call', async () => {
            const result = await aiBridge.callTool('copy_to_clipboard', {
                content: 'Test content',
                format: 'text'
            });

            expect(result).toEqual({
                success: true,
                content: 'Test content',
                format: 'text',
                message: 'Content copied to clipboard'
            });
        });

        test('should execute open_file tool call', async () => {
            const result = await aiBridge.callTool('open_file', {
                filePath: '/path/to/file.txt',
                action: 'open'
            });

            expect(result).toEqual({
                success: true,
                filePath: '/path/to/file.txt',
                action: 'open',
                message: 'Opened file: /path/to/file.txt'
            });
        });

        test('should execute show_in_folder tool call', async () => {
            const result = await aiBridge.callTool('show_in_folder', {
                filePath: '/path/to/file.txt'
            });

            expect(result).toEqual({
                success: true,
                filePath: '/path/to/file.txt',
                message: 'Showing file in folder: /path/to/file.txt'
            });
        });

        test('should execute retry_file_operation tool call', async () => {
            const result = await aiBridge.callTool('retry_file_operation', {
                operation: 'upload',
                fileName: 'test.txt'
            });

            expect(result).toEqual({
                success: true,
                operation: 'upload',
                fileName: 'test.txt',
                message: 'Retrying upload on test.txt'
            });
        });

        test('should execute show_error_details tool call', async () => {
            const result = await aiBridge.callTool('show_error_details', {
                operation: 'upload',
                fileName: 'test.txt'
            });

            expect(result).toEqual({
                success: true,
                operation: 'upload',
                fileName: 'test.txt',
                errorDetails: 'Detailed error information would be shown here'
            });
        });

        test('should execute show_update_details tool call', async () => {
            const result = await aiBridge.callTool('show_update_details', {
                updateType: 'Security Update',
                details: 'Critical security patches available'
            });

            expect(result).toEqual({
                success: true,
                updateType: 'Security Update',
                details: 'Critical security patches available',
                message: 'Update details displayed'
            });
        });

        test('should execute apply_system_update tool call', async () => {
            const result = await aiBridge.callTool('apply_system_update', {
                updateType: 'Security Update'
            });

            expect(result).toEqual({
                success: true,
                updateType: 'Security Update',
                message: 'System update applied successfully'
            });
        });

        test('should execute schedule_update tool call', async () => {
            const result = await aiBridge.callTool('schedule_update', {
                updateType: 'Security Update',
                scheduleTime: 'later'
            });

            expect(result).toEqual({
                success: true,
                updateType: 'Security Update',
                scheduleTime: 'later',
                message: 'Update scheduled for later'
            });
        });

        test('should execute show_backup_details tool call', async () => {
            const result = await aiBridge.callTool('show_backup_details', {
                backupType: 'Daily Backup'
            });

            expect(result).toEqual({
                success: true,
                backupType: 'Daily Backup',
                message: 'Backup details displayed'
            });
        });

        test('should execute restore_from_backup tool call', async () => {
            const result = await aiBridge.callTool('restore_from_backup', {
                backupType: 'Daily Backup'
            });

            expect(result).toEqual({
                success: true,
                backupType: 'Daily Backup',
                message: 'Restore from backup initiated'
            });
        });

        test('should execute retry_backup tool call', async () => {
            const result = await aiBridge.callTool('retry_backup', {
                backupType: 'Daily Backup'
            });

            expect(result).toEqual({
                success: true,
                backupType: 'Daily Backup',
                message: 'Backup retry initiated'
            });
        });

        test('should execute show_backup_error tool call', async () => {
            const result = await aiBridge.callTool('show_backup_error', {
                backupType: 'Daily Backup',
                details: 'Insufficient space'
            });

            expect(result).toEqual({
                success: true,
                backupType: 'Daily Backup',
                details: 'Insufficient space',
                errorDetails: 'Backup error details would be shown here'
            });
        });

        test('should handle unknown tool call', async () => {
            await expect(aiBridge.callTool('unknown_tool', {}))
                .rejects.toThrow('Unknown tool: unknown_tool');
        });

        test('should handle tool call when disconnected', async () => {
            aiBridge.disconnect();

            await expect(aiBridge.callTool('open_note', {}))
                .rejects.toThrow('AI Core Service not connected');
        });
    });

    describe('Error Handling', () => {
        test('should handle tool call timeout', async () => {
            // Mock a slow tool call
            const originalSimulateToolExecution = (aiBridge as any).simulateToolExecution;
            (aiBridge as any).simulateToolExecution = jest.fn().mockImplementation(
                () => new Promise(resolve => setTimeout(resolve, 35000)) // Longer than timeout
            );

            await expect(aiBridge.callTool('slow_tool', {}))
                .rejects.toThrow('Tool call timeout: slow_tool');

            // Restore original method
            (aiBridge as any).simulateToolExecution = originalSimulateToolExecution;
        });

        test('should handle tool execution errors', async () => {
            // Mock a tool that throws an error
            const originalSimulateToolExecution = (aiBridge as any).simulateToolExecution;
            (aiBridge as any).simulateToolExecution = jest.fn().mockRejectedValue(
                new Error('Tool execution failed')
            );

            await expect(aiBridge.callTool('failing_tool', {}))
                .rejects.toThrow('Tool execution failed');

            // Restore original method
            (aiBridge as any).simulateToolExecution = originalSimulateToolExecution;
        });
    });

    describe('Concurrent Tool Calls', () => {
        test('should handle multiple concurrent tool calls', async () => {
            const promises = [
                aiBridge.callTool('open_note', { noteId: 'note1' }),
                aiBridge.callTool('open_note', { noteId: 'note2' }),
                aiBridge.callTool('open_note', { noteId: 'note3' })
            ];

            const results = await Promise.all(promises);

            expect(results).toHaveLength(3);
            results.forEach((result, index) => {
                expect(result.noteId).toBe(`note${index + 1}`);
            });
        });

        test('should handle mixed success and failure tool calls', async () => {
            const promises = [
                aiBridge.callTool('open_note', { noteId: 'note1' }),
                aiBridge.callTool('unknown_tool', {}),
                aiBridge.callTool('open_note', { noteId: 'note2' })
            ];

            const results = await Promise.allSettled(promises);

            expect(results[0].status).toBe('fulfilled');
            expect(results[1].status).toBe('rejected');
            expect(results[2].status).toBe('fulfilled');
        });
    });

    describe('Event System', () => {
        test('should emit tool call success events', async () => {
            const successSpy = jest.fn();
            aiBridge.on('tool:call-success', successSpy);

            await aiBridge.callTool('open_note', { noteId: 'test' });

            expect(successSpy).toHaveBeenCalledWith(
                'open_note',
                { noteId: 'test' },
                expect.any(Object)
            );
        });

        test('should emit tool call error events', async () => {
            const errorSpy = jest.fn();
            aiBridge.on('tool:call-error', errorSpy);

            try {
                await aiBridge.callTool('unknown_tool', {});
            } catch (error) {
                // Expected error
            }

            expect(errorSpy).toHaveBeenCalledWith(
                'unknown_tool',
                {},
                expect.any(Error)
            );
        });
    });

    describe('Cleanup', () => {
        test('should cleanup resources', () => {
            const disconnectSpy = jest.spyOn(aiBridge, 'disconnect');
            const removeAllListenersSpy = jest.spyOn(aiBridge, 'removeAllListeners');

            aiBridge.cleanup();

            expect(disconnectSpy).toHaveBeenCalled();
            expect(removeAllListenersSpy).toHaveBeenCalled();
        });
    });
});
