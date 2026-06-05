/**
 * @file summary-notification.ts
 * @brief Example: Summary Ready Notification with Action Buttons
 * 
 * This example demonstrates the expected behavior:
 * When summary ready → toast with "Open note" action
 */

import { NotificationManager, initializeNotificationManager } from '../notify';
import { AICoreBridge } from '../bridge/ai-core-bridge';

/**
 * Example: Create a summary ready notification
 */
export async function createSummaryReadyExample(): Promise<void> {
    // Initialize the notification system
    const aiBridge = new AICoreBridge();
    const notificationManager = initializeNotificationManager(aiBridge);

    // Simulate a summary being generated
    const summaryId = 'summary_2024_01_15_meeting_notes';
    const summaryTitle = 'Weekly Team Meeting - January 15, 2024';

    console.log('📝 Generating summary...');
    
    // Simulate summary generation delay
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    console.log('✅ Summary generated successfully!');
    
    // Create the notification with actionable buttons
    const notificationId = notificationManager.createSummaryReadyNotification(
        summaryId,
        summaryTitle
    );

    console.log(`🔔 Notification created: ${notificationId}`);
    console.log('📱 Check your notification center for the "Summary Ready" toast');
    console.log('🎯 The notification includes these action buttons:');
    console.log('   • 📝 Open Note - Opens the summary in your note app');
    console.log('   • 📋 Copy Summary - Copies the summary to clipboard');
    console.log('   • ✕ Dismiss - Dismisses the notification');

    // Set up event listeners to demonstrate the action flow
    notificationManager.on('notification:actioned', (notification, action) => {
        console.log(`🎯 Action executed: ${action.label} for notification ${notification.id}`);
        
        if (action.toolCall) {
            console.log(`🔧 Tool call: ${action.toolCall.toolName} with parameters:`, action.toolCall.parameters);
        }
    });

    notificationManager.on('tool:call-success', (toolName, parameters, result) => {
        console.log(`✅ Tool call successful: ${toolName}`);
        console.log(`📊 Result:`, result);
    });

    notificationManager.on('tool:call-error', (toolName, parameters, error) => {
        console.log(`❌ Tool call failed: ${toolName}`);
        console.log(`🚨 Error:`, error);
    });

    return notificationId;
}

/**
 * Example: Simulate user clicking "Open Note" action
 */
export async function simulateOpenNoteAction(notificationId: string): Promise<void> {
    const aiBridge = new AICoreBridge();
    const notificationManager = initializeNotificationManager(aiBridge);

    // Get the notification
    const notification = notificationManager.getNotification(notificationId);
    if (!notification) {
        console.log('❌ Notification not found');
        return;
    }

    // Find the "Open Note" action
    const openNoteAction = notification.actions?.find(action => action.id === 'open_note');
    if (!openNoteAction) {
        console.log('❌ Open Note action not found');
        return;
    }

    console.log('🎯 Simulating user click on "Open Note" action...');
    
    // Trigger the action
    notificationManager.emit('notification:action', notification, openNoteAction);
}

/**
 * Example: Create multiple notification types
 */
export async function createMultipleNotificationExamples(): Promise<void> {
    const aiBridge = new AICoreBridge();
    const notificationManager = initializeNotificationManager(aiBridge);

    console.log('🔔 Creating multiple notification examples...\n');

    // 1. Summary Ready Notification
    console.log('1️⃣ Summary Ready Notification:');
    const summaryId = notificationManager.createSummaryReadyNotification(
        'summary_project_planning',
        'Q1 2024 Project Planning Session'
    );
    console.log(`   Created: ${summaryId}\n`);

    // 2. File Operation Notification
    console.log('2️⃣ File Operation Notification:');
    const fileId = notificationManager.createFileOperationNotification(
        'save',
        'project_proposal.docx',
        true
    );
    console.log(`   Created: ${fileId}\n`);

    // 3. System Update Notification
    console.log('3️⃣ System Update Notification:');
    const updateId = notificationManager.createSystemUpdateNotification(
        'Security Update',
        'Critical security patches available for your system'
    );
    console.log(`   Created: ${updateId}\n`);

    // 4. Backup Notification
    console.log('4️⃣ Backup Notification:');
    const backupId = notificationManager.createBackupNotification(
        'Daily Backup',
        true,
        'Backup completed successfully. 2.3GB of data backed up.'
    );
    console.log(`   Created: ${backupId}\n`);

    // 5. Error Notification
    console.log('5️⃣ Error Notification:');
    const errorId = notificationManager.createFileOperationNotification(
        'upload',
        'large_video_file.mp4',
        false
    );
    console.log(`   Created: ${errorId}\n`);

    console.log('✅ All notification examples created!');
    console.log('📱 Check your notification center to see all the different types');
}

/**
 * Example: Demonstrate notification lifecycle
 */
export async function demonstrateNotificationLifecycle(): Promise<void> {
    const aiBridge = new AICoreBridge();
    const notificationManager = initializeNotificationManager(aiBridge);

    console.log('🔄 Demonstrating notification lifecycle...\n');

    // Create a notification
    const notificationId = notificationManager.createSummaryReadyNotification(
        'demo_summary',
        'Demo Summary for Lifecycle Test'
    );

    console.log(`1️⃣ Created notification: ${notificationId}`);

    // Wait a bit
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Check active notifications
    const activeNotifications = notificationManager.getNotifications();
    console.log(`2️⃣ Active notifications: ${activeNotifications.length}`);

    // Simulate action
    const notification = notificationManager.getNotification(notificationId);
    if (notification) {
        const action = notification.actions?.[0];
        if (action) {
            console.log(`3️⃣ Executing action: ${action.label}`);
            notificationManager.emit('notification:action', notification, action);
        }
    }

    // Wait for action to complete
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Check notifications again (should be dismissed after action)
    const remainingNotifications = notificationManager.getNotifications();
    console.log(`4️⃣ Remaining notifications: ${remainingNotifications.length}`);

    console.log('✅ Notification lifecycle demonstration complete!');
}

// Export the main example function
export async function runSummaryNotificationExample(): Promise<void> {
    console.log('🚀 Starting Summary Notification Example\n');
    
    try {
        // Create the main example
        const notificationId = await createSummaryReadyExample();
        
        // Wait a bit
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        // Simulate user action
        await simulateOpenNoteAction(notificationId);
        
        // Wait a bit more
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Show multiple examples
        await createMultipleNotificationExamples();
        
        // Wait a bit
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Demonstrate lifecycle
        await demonstrateNotificationLifecycle();
        
        console.log('\n🎉 Summary Notification Example completed successfully!');
        
    } catch (error) {
        console.error('❌ Example failed:', error);
    }
}

// Run the example if this file is executed directly
if (require.main === module) {
    runSummaryNotificationExample().catch(console.error);
}
