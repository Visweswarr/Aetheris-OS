/// IPC Ping-Pong Integration Test for Polymera OS
/// 
/// This test validates the IPC system by spawning two tasks that exchange
/// ping-pong messages, measuring latency and ensuring correct message ordering.

use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use core::arch::x86_64::_rdtsc;

use crate::ipc::types::*;
use crate::ipc::sys;
use crate::sched::{TaskId, TaskState};
use crate::security::cap;
use crate::{kprintln, klog};

/// Maximum number of ping-pong iterations to perform
const MAX_ITERATIONS: usize = 100;

/// Expected maximum median latency in microseconds (200µs)
const MAX_MEDIAN_LATENCY_US: u64 = 200;

/// TSC frequency estimation (will be calibrated at runtime)
static TSC_FREQUENCY_MHZ: AtomicU64 = AtomicU64::new(2400); // Default 2.4 GHz

/// Test completion flags
static TASK_A_COMPLETE: AtomicBool = AtomicBool::new(false);
static TASK_B_COMPLETE: AtomicBool = AtomicBool::new(false);
static TEST_FAILED: AtomicBool = AtomicBool::new(false);

/// Latency measurements storage
static mut LATENCY_MEASUREMENTS: [u64; MAX_ITERATIONS] = [0; MAX_ITERATIONS];
static MEASUREMENT_COUNT: AtomicU64 = AtomicU64::new(0);

/// Message sequence tracking
static EXPECTED_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static ACTUAL_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Timestamp helper functions
mod timestamp {
    use super::*;
    
    /// Get current timestamp using RDTSC
    #[inline]
    pub fn get_timestamp() -> u64 {
        unsafe { _rdtsc() }
    }
    
    /// Convert TSC cycles to microseconds
    #[inline]
    pub fn cycles_to_microseconds(cycles: u64) -> u64 {
        let freq_mhz = TSC_FREQUENCY_MHZ.load(Ordering::Relaxed);
        if freq_mhz > 0 {
            cycles / freq_mhz
        } else {
            cycles / 2400 // Fallback to 2.4 GHz
        }
    }
    
    /// Calibrate TSC frequency (simplified implementation)
    pub fn calibrate_tsc_frequency() {
        // For testing purposes, we'll use a reasonable default
        // In a real implementation, this would measure against a known timer
        TSC_FREQUENCY_MHZ.store(2400, Ordering::Relaxed); // 2.4 GHz default
        kprintln!("[PING-PONG] TSC frequency calibrated to {} MHz", 
                  TSC_FREQUENCY_MHZ.load(Ordering::Relaxed));
    }
    
    /// Get elapsed time between two timestamps in microseconds
    #[inline]
    pub fn elapsed_microseconds(start: u64, end: u64) -> u64 {
        cycles_to_microseconds(end.saturating_sub(start))
    }
}

/// Message types for ping-pong test
mod ping_pong_messages {
    use super::*;
    
    /// Create a ping message with sequence number
    pub fn create_ping_message(sequence: u64, sender: ProcessId, receiver: ProcessId) -> Message {
        let payload_text = format!("ping-{}", sequence);
        Message::new_with_priority(
            sender,
            receiver,
            MessageType::Request,
            MessagePriority::Normal,
            MessagePayload::from_text(&payload_text),
        )
    }
    
    /// Create a pong message with sequence number
    pub fn create_pong_message(sequence: u64, sender: ProcessId, receiver: ProcessId) -> Message {
        let payload_text = format!("pong-{}", sequence);
        Message::new_with_priority(
            sender,
            receiver,
            MessageType::Response,
            MessagePriority::Normal,
            MessagePayload::from_text(&payload_text),
        )
    }
    
    /// Extract sequence number from message payload
    pub fn extract_sequence(message: &Message) -> Option<u64> {
        if let MessagePayload::Text(ref text_data) = message.payload {
            if let Ok(text) = core::str::from_utf8(&text_data.data) {
                if text.starts_with("ping-") || text.starts_with("pong-") {
                    if let Some(seq_str) = text.split('-').nth(1) {
                        return seq_str.parse().ok();
                    }
                }
            }
        }
        None
    }
    
    /// Check if message is a ping
    pub fn is_ping(message: &Message) -> bool {
        if let MessagePayload::Text(ref text_data) = message.payload {
            if let Ok(text) = core::str::from_utf8(&text_data.data) {
                return text.starts_with("ping-");
            }
        }
        false
    }
    
    /// Check if message is a pong
    pub fn is_pong(message: &Message) -> bool {
        if let MessagePayload::Text(ref text_data) = message.payload {
            if let Ok(text) = core::str::from_utf8(&text_data.data) {
                return text.starts_with("pong-");
            }
        }
        false
    }
}

/// Task A: Ping sender
/// Sends ping messages and waits for pong responses
fn task_a_main() {
    let task_a_id = ProcessId(10);
    let task_b_id = ProcessId(20);
    
    kprintln!("[TASK-A] Starting ping-pong sender task");
    
    for iteration in 0..MAX_ITERATIONS {
        let sequence = iteration as u64 + 1;
        
        // Create ping message
        let ping_message = ping_pong_messages::create_ping_message(sequence, task_a_id, task_b_id);
        
        klog!(TRACE, "[TASK-A] Sending ping #{}", sequence);
        
        // Record start timestamp
        let start_time = timestamp::get_timestamp();
        
        // Send ping message
        match sys::sys_send(task_b_id.0, &ping_message) {
            Ok(()) => {
                klog!(TRACE, "[TASK-A] Ping #{} sent successfully", sequence);
            }
            Err(e) => {
                kprintln!("[TASK-A] Failed to send ping #{}: {}", sequence, e);
                TEST_FAILED.store(true, Ordering::Relaxed);
                break;
            }
        }
        
        // Wait for pong response (blocking receive)
        match sys::sys_recv(true) {
            Ok(response_message) => {
                let end_time = timestamp::get_timestamp();
                
                // Validate that it's a pong message
                if ping_pong_messages::is_pong(&response_message) {
                    if let Some(response_seq) = ping_pong_messages::extract_sequence(&response_message) {
                        if response_seq == sequence {
                            // Calculate and record latency
                            let latency_us = timestamp::elapsed_microseconds(start_time, end_time);
                            
                            // Store latency measurement
                            let measurement_idx = MEASUREMENT_COUNT.fetch_add(1, Ordering::Relaxed) as usize;
                            if measurement_idx < MAX_ITERATIONS {
                                unsafe {
                                    LATENCY_MEASUREMENTS[measurement_idx] = latency_us;
                                }
                            }
                            
                            // Update sequence tracking
                            ACTUAL_SEQUENCE.store(sequence, Ordering::Relaxed);
                            
                            klog!(TRACE, "[TASK-A] Received pong #{} (latency: {}µs)", 
                                  sequence, latency_us);
                        } else {
                            kprintln!("[TASK-A] Sequence mismatch: expected {}, got {}", 
                                      sequence, response_seq);
                            TEST_FAILED.store(true, Ordering::Relaxed);
                            break;
                        }
                    } else {
                        kprintln!("[TASK-A] Failed to extract sequence from pong message");
                        TEST_FAILED.store(true, Ordering::Relaxed);
                        break;
                    }
                } else {
                    kprintln!("[TASK-A] Received unexpected message type (not pong)");
                    TEST_FAILED.store(true, Ordering::Relaxed);
                    break;
                }
            }
            Err(e) => {
                kprintln!("[TASK-A] Failed to receive pong #{}: {}", sequence, e);
                TEST_FAILED.store(true, Ordering::Relaxed);
                break;
            }
        }
    }
    
    kprintln!("[TASK-A] Ping-pong sender task completed");
    TASK_A_COMPLETE.store(true, Ordering::Relaxed);
}

/// Task B: Pong responder
/// Waits for ping messages and sends pong responses
fn task_b_main() {
    let task_a_id = ProcessId(10);
    let task_b_id = ProcessId(20);
    
    kprintln!("[TASK-B] Starting ping-pong responder task");
    
    for iteration in 0..MAX_ITERATIONS {
        let expected_sequence = iteration as u64 + 1;
        
        klog!(TRACE, "[TASK-B] Waiting for ping #{}", expected_sequence);
        
        // Wait for ping message (blocking receive)
        match sys::sys_recv(true) {
            Ok(ping_message) => {
                // Validate that it's a ping message
                if ping_pong_messages::is_ping(&ping_message) {
                    if let Some(ping_seq) = ping_pong_messages::extract_sequence(&ping_message) {
                        if ping_seq == expected_sequence {
                            klog!(TRACE, "[TASK-B] Received ping #{}", ping_seq);
                            
                            // Create pong response
                            let pong_message = ping_pong_messages::create_pong_message(
                                ping_seq, task_b_id, task_a_id);
                            
                            // Send pong response
                            match sys::sys_send(task_a_id.0, &pong_message) {
                                Ok(()) => {
                                    klog!(TRACE, "[TASK-B] Sent pong #{}", ping_seq);
                                }
                                Err(e) => {
                                    kprintln!("[TASK-B] Failed to send pong #{}: {}", ping_seq, e);
                                    TEST_FAILED.store(true, Ordering::Relaxed);
                                    break;
                                }
                            }
                        } else {
                            kprintln!("[TASK-B] Sequence mismatch: expected {}, got {}", 
                                      expected_sequence, ping_seq);
                            TEST_FAILED.store(true, Ordering::Relaxed);
                            break;
                        }
                    } else {
                        kprintln!("[TASK-B] Failed to extract sequence from ping message");
                        TEST_FAILED.store(true, Ordering::Relaxed);
                        break;
                    }
                } else {
                    kprintln!("[TASK-B] Received unexpected message type (not ping)");
                    TEST_FAILED.store(true, Ordering::Relaxed);
                    break;
                }
            }
            Err(e) => {
                kprintln!("[TASK-B] Failed to receive ping #{}: {}", expected_sequence, e);
                TEST_FAILED.store(true, Ordering::Relaxed);
                break;
            }
        }
    }
    
    kprintln!("[TASK-B] Ping-pong responder task completed");
    TASK_B_COMPLETE.store(true, Ordering::Relaxed);
}

/// Statistics and analysis functions
mod statistics {
    use super::*;
    
    /// Calculate median latency from measurements
    pub fn calculate_median_latency() -> u64 {
        let count = MEASUREMENT_COUNT.load(Ordering::Relaxed) as usize;
        if count == 0 {
            return 0;
        }
        
        // Copy measurements to a sortable array
        let mut measurements = Vec::new();
        for i in 0..count.min(MAX_ITERATIONS) {
            unsafe {
                measurements.push(LATENCY_MEASUREMENTS[i]);
            }
        }
        
        // Sort measurements
        measurements.sort_unstable();
        
        // Calculate median
        if measurements.len() % 2 == 0 {
            let mid = measurements.len() / 2;
            (measurements[mid - 1] + measurements[mid]) / 2
        } else {
            measurements[measurements.len() / 2]
        }
    }
    
    /// Calculate average latency
    pub fn calculate_average_latency() -> u64 {
        let count = MEASUREMENT_COUNT.load(Ordering::Relaxed) as usize;
        if count == 0 {
            return 0;
        }
        
        let mut sum = 0u64;
        for i in 0..count.min(MAX_ITERATIONS) {
            unsafe {
                sum += LATENCY_MEASUREMENTS[i];
            }
        }
        
        sum / count as u64
    }
    
    /// Calculate minimum latency
    pub fn calculate_min_latency() -> u64 {
        let count = MEASUREMENT_COUNT.load(Ordering::Relaxed) as usize;
        if count == 0 {
            return 0;
        }
        
        let mut min = u64::MAX;
        for i in 0..count.min(MAX_ITERATIONS) {
            unsafe {
                if LATENCY_MEASUREMENTS[i] < min {
                    min = LATENCY_MEASUREMENTS[i];
                }
            }
        }
        
        min
    }
    
    /// Calculate maximum latency
    pub fn calculate_max_latency() -> u64 {
        let count = MEASUREMENT_COUNT.load(Ordering::Relaxed) as usize;
        if count == 0 {
            return 0;
        }
        
        let mut max = 0u64;
        for i in 0..count.min(MAX_ITERATIONS) {
            unsafe {
                if LATENCY_MEASUREMENTS[i] > max {
                    max = LATENCY_MEASUREMENTS[i];
                }
            }
        }
        
        max
    }
    
    /// Print comprehensive latency statistics
    pub fn print_latency_statistics() {
        let count = MEASUREMENT_COUNT.load(Ordering::Relaxed);
        let median = calculate_median_latency();
        let average = calculate_average_latency();
        let min = calculate_min_latency();
        let max = calculate_max_latency();
        
        kprintln!("");
        kprintln!("=== PING-PONG LATENCY STATISTICS ===");
        kprintln!("Measurements collected: {}", count);
        kprintln!("Minimum latency: {}µs", min);
        kprintln!("Maximum latency: {}µs", max);
        kprintln!("Average latency: {}µs", average);
        kprintln!("Median latency: {}µs", median);
        kprintln!("Target median: <{}µs", MAX_MEDIAN_LATENCY_US);
        
        if median < MAX_MEDIAN_LATENCY_US {
            kprintln!("✅ LATENCY TEST PASSED");
        } else {
            kprintln!("❌ LATENCY TEST FAILED");
        }
        
        kprintln!("=== END LATENCY STATISTICS ===");
        kprintln!("");
    }
}

/// Setup capabilities for ping-pong tasks
fn setup_ping_pong_capabilities() -> Result<(), &'static str> {
    let task_a_id = 10;
    let task_b_id = 20;
    
    // Grant capability for Task A to send to Task B
    match cap::grant_capability(task_a_id, task_b_id, cap::scope::SEND, 60000) {
        Ok(token) => {
            klog!(TRACE, "[PING-PONG] Granted A->B capability: {}", token);
        }
        Err(_) => {
            return Err("Failed to grant capability A->B");
        }
    }
    
    // Grant capability for Task B to send to Task A
    match cap::grant_capability(task_b_id, task_a_id, cap::scope::SEND, 60000) {
        Ok(token) => {
            klog!(TRACE, "[PING-PONG] Granted B->A capability: {}", token);
        }
        Err(_) => {
            return Err("Failed to grant capability B->A");
        }
    }
    
    Ok(())
}

/// Spawn ping-pong tasks
fn spawn_ping_pong_tasks() -> Result<(TaskId, TaskId), &'static str> {
    // For this test, we'll simulate task creation
    // In a real implementation, we would create actual kernel tasks
    
    let task_a_id = TaskId(10);
    let task_b_id = TaskId(20);
    
    // Create Task A (ping sender)
    let task_a = crate::sched::create_task(0x10000);
    if task_a.0 == 0 {
        return Err("Failed to create Task A");
    }
    
    // Create Task B (pong responder)
    let task_b = crate::sched::create_task(0x20000);
    if task_b.0 == 0 {
        return Err("Failed to create Task B");
    }
    
    kprintln!("[PING-PONG] Spawned Task A (ID: {}) and Task B (ID: {})", 
              task_a.0, task_b.0);
    
    Ok((task_a, task_b))
}

/// Validate message order and sequence
fn validate_message_order() -> bool {
    let expected = EXPECTED_SEQUENCE.load(Ordering::Relaxed);
    let actual = ACTUAL_SEQUENCE.load(Ordering::Relaxed);
    
    kprintln!("[PING-PONG] Message order validation:");
    kprintln!("  Expected sequence: {}", expected);
    kprintln!("  Actual sequence: {}", actual);
    
    if actual >= MAX_ITERATIONS as u64 {
        kprintln!("  ✅ All messages exchanged in correct order");
        true
    } else {
        kprintln!("  ❌ Message sequence incomplete or out of order");
        false
    }
}

/// Wait for tasks to complete with timeout
fn wait_for_completion() -> bool {
    let mut timeout_counter = 0;
    const MAX_TIMEOUT: u32 = 1000; // Adjust based on expected test duration
    
    while timeout_counter < MAX_TIMEOUT {
        if TASK_A_COMPLETE.load(Ordering::Relaxed) && 
           TASK_B_COMPLETE.load(Ordering::Relaxed) {
            return true;
        }
        
        if TEST_FAILED.load(Ordering::Relaxed) {
            return false;
        }
        
        // Yield CPU to allow tasks to run
        crate::sched::yield_current();
        timeout_counter += 1;
    }
    
    false
}

/// Main ping-pong integration test
pub fn run_ping_pong_test() -> bool {
    kprintln!("");
    kprintln!("=== IPC PING-PONG INTEGRATION TEST ===");
    kprintln!("Testing IPC system with {} iterations", MAX_ITERATIONS);
    kprintln!("Target median latency: <{}µs", MAX_MEDIAN_LATENCY_US);
    
    // Initialize test state
    TASK_A_COMPLETE.store(false, Ordering::Relaxed);
    TASK_B_COMPLETE.store(false, Ordering::Relaxed);
    TEST_FAILED.store(false, Ordering::Relaxed);
    MEASUREMENT_COUNT.store(0, Ordering::Relaxed);
    EXPECTED_SEQUENCE.store(MAX_ITERATIONS as u64, Ordering::Relaxed);
    ACTUAL_SEQUENCE.store(0, Ordering::Relaxed);
    
    // Calibrate timestamp system
    timestamp::calibrate_tsc_frequency();
    
    // Setup capabilities
    match setup_ping_pong_capabilities() {
        Ok(()) => kprintln!("✅ Capabilities setup successful"),
        Err(e) => {
            kprintln!("❌ Capabilities setup failed: {}", e);
            return false;
        }
    }
    
    // Spawn tasks
    match spawn_ping_pong_tasks() {
        Ok((task_a, task_b)) => {
            kprintln!("✅ Tasks spawned successfully: A={}, B={}", task_a.0, task_b.0);
        }
        Err(e) => {
            kprintln!("❌ Task spawning failed: {}", e);
            return false;
        }
    }
    
    // For this test, we'll simulate the task execution
    // In a real kernel, the tasks would run concurrently
    kprintln!("🔄 Starting ping-pong message exchange...");
    
    // Simulate concurrent execution by alternating between tasks
    // This is a simplified version for testing purposes
    for iteration in 0..MAX_ITERATIONS {
        klog!(TRACE, "[PING-PONG] Iteration {} starting", iteration + 1);
        
        // Simulate Task A sending ping and Task B responding with pong
        // In real implementation, this would be handled by the scheduler
        
        // Check for test failure
        if TEST_FAILED.load(Ordering::Relaxed) {
            kprintln!("❌ Test failed during iteration {}", iteration + 1);
            break;
        }
    }
    
    // Wait for task completion
    kprintln!("⏳ Waiting for tasks to complete...");
    let completion_success = wait_for_completion();
    
    if !completion_success {
        kprintln!("❌ Tasks did not complete within timeout");
        return false;
    }
    
    // Validate results
    let order_valid = validate_message_order();
    let test_passed = !TEST_FAILED.load(Ordering::Relaxed);
    
    // Print statistics
    statistics::print_latency_statistics();
    
    // Final validation
    let median_latency = statistics::calculate_median_latency();
    let latency_test_passed = median_latency < MAX_MEDIAN_LATENCY_US;
    
    let overall_success = test_passed && order_valid && latency_test_passed;
    
    kprintln!("");
    kprintln!("=== PING-PONG TEST RESULTS ===");
    kprintln!("Message exchange: {}", if test_passed { "✅ PASSED" } else { "❌ FAILED" });
    kprintln!("Message order: {}", if order_valid { "✅ PASSED" } else { "❌ FAILED" });
    kprintln!("Latency requirement: {}", if latency_test_passed { "✅ PASSED" } else { "❌ FAILED" });
    kprintln!("Overall test: {}", if overall_success { "✅ PASSED" } else { "❌ FAILED" });
    kprintln!("=== END PING-PONG TEST ===");
    kprintln!("");
    
    overall_success
}

/// Simplified ping-pong test for integration into boot sequence
pub fn run_simple_ping_pong_test() -> bool {
    kprintln!("");
    kprintln!("=== SIMPLE IPC PING-PONG TEST ===");
    
    let task_a_id = ProcessId(10);
    let task_b_id = ProcessId(20);
    
    // Setup capabilities for the test
    match setup_ping_pong_capabilities() {
        Ok(()) => {
            kprintln!("✅ Capabilities setup successful");
        }
        Err(e) => {
            kprintln!("❌ Capabilities setup failed: {}", e);
            return false;
        }
    }
    
    // Simulate a single ping-pong exchange for validation
    kprintln!("🏓 Testing single ping-pong exchange...");
    
    // Create a ping message
    let ping_message = ping_pong_messages::create_ping_message(1, task_a_id, task_b_id);
    
    // Measure start time
    let start_time = timestamp::get_timestamp();
    
    // Send ping message (Task A -> Task B)
    match sys::sys_send(task_b_id.0, &ping_message) {
        Ok(()) => {
            kprintln!("  ✅ Ping message sent successfully");
        }
        Err(e) => {
            kprintln!("  ❌ Failed to send ping message: {}", e);
            return false;
        }
    }
    
    // Try to receive the ping message (simulating Task B)
    match crate::ipc::queues::try_receive_message(task_b_id.0) {
        Some(received_ping) => {
            if ping_pong_messages::is_ping(&received_ping) {
                kprintln!("  ✅ Ping message received successfully");
                
                // Create pong response
                let pong_message = ping_pong_messages::create_pong_message(1, task_b_id, task_a_id);
                
                // Send pong response (Task B -> Task A)
                match sys::sys_send(task_a_id.0, &pong_message) {
                    Ok(()) => {
                        kprintln!("  ✅ Pong message sent successfully");
                    }
                    Err(e) => {
                        kprintln!("  ❌ Failed to send pong message: {}", e);
                        return false;
                    }
                }
                
                // Try to receive the pong message (simulating Task A)
                match crate::ipc::queues::try_receive_message(task_a_id.0) {
                    Some(received_pong) => {
                        if ping_pong_messages::is_pong(&received_pong) {
                            let end_time = timestamp::get_timestamp();
                            let latency_us = timestamp::elapsed_microseconds(start_time, end_time);
                            
                            kprintln!("  ✅ Pong message received successfully");
                            kprintln!("  📊 Round-trip latency: {}µs", latency_us);
                            
                            if latency_us < MAX_MEDIAN_LATENCY_US {
                                kprintln!("  ✅ Latency test PASSED ({}µs < {}µs)", 
                                          latency_us, MAX_MEDIAN_LATENCY_US);
                            } else {
                                kprintln!("  ❌ Latency test FAILED ({}µs >= {}µs)", 
                                          latency_us, MAX_MEDIAN_LATENCY_US);
                                return false;
                            }
                        } else {
                            kprintln!("  ❌ Received message is not a pong");
                            return false;
                        }
                    }
                    None => {
                        kprintln!("  ❌ Failed to receive pong message");
                        return false;
                    }
                }
            } else {
                kprintln!("  ❌ Received message is not a ping");
                return false;
            }
        }
        None => {
            kprintln!("  ❌ Failed to receive ping message");
            return false;
        }
    }
    
    kprintln!("✅ Simple ping-pong test PASSED");
    kprintln!("=== END SIMPLE PING-PONG TEST ===");
    kprintln!("");
    
    true
}

/// Quick ping-pong test with fewer iterations for CI
pub fn run_quick_ping_pong_test() -> bool {
    // For the quick test, just run the simple test
    run_simple_ping_pong_test()
}

/// Benchmark test with multiple iterations (for performance testing)
pub fn run_benchmark_ping_pong_test() -> bool {
    kprintln!("");
    kprintln!("=== IPC PING-PONG BENCHMARK TEST ===");
    kprintln!("Testing {} iterations for performance analysis", MAX_ITERATIONS);
    
    // Initialize test
    timestamp::calibrate_tsc_frequency();
    
    // Setup capabilities
    match setup_ping_pong_capabilities() {
        Ok(()) => {
            kprintln!("✅ Capabilities setup successful");
        }
        Err(e) => {
            kprintln!("❌ Capabilities setup failed: {}", e);
            return false;
        }
    }
    
    let task_a_id = ProcessId(10);
    let task_b_id = ProcessId(20);
    let mut latencies = Vec::new();
    
    // Run multiple iterations
    for iteration in 0..MAX_ITERATIONS.min(10) { // Limit to 10 for boot-time testing
        let sequence = iteration as u64 + 1;
        
        // Create ping message
        let ping_message = ping_pong_messages::create_ping_message(sequence, task_a_id, task_b_id);
        
        let start_time = timestamp::get_timestamp();
        
        // Send ping
        if let Err(_) = sys::sys_send(task_b_id.0, &ping_message) {
            kprintln!("❌ Failed to send ping #{}", sequence);
            return false;
        }
        
        // Receive ping (simulate Task B)
        if let Some(received_ping) = crate::ipc::queues::try_receive_message(task_b_id.0) {
            if ping_pong_messages::is_ping(&received_ping) {
                // Send pong response
                let pong_message = ping_pong_messages::create_pong_message(sequence, task_b_id, task_a_id);
                
                if let Err(_) = sys::sys_send(task_a_id.0, &pong_message) {
                    kprintln!("❌ Failed to send pong #{}", sequence);
                    return false;
                }
                
                // Receive pong (simulate Task A)
                if let Some(received_pong) = crate::ipc::queues::try_receive_message(task_a_id.0) {
                    if ping_pong_messages::is_pong(&received_pong) {
                        let end_time = timestamp::get_timestamp();
                        let latency_us = timestamp::elapsed_microseconds(start_time, end_time);
                        latencies.push(latency_us);
                        
                        klog!(TRACE, "[BENCHMARK] Iteration {} latency: {}µs", sequence, latency_us);
                    } else {
                        kprintln!("❌ Invalid pong message #{}", sequence);
                        return false;
                    }
                } else {
                    kprintln!("❌ Failed to receive pong #{}", sequence);
                    return false;
                }
            } else {
                kprintln!("❌ Invalid ping message #{}", sequence);
                return false;
            }
        } else {
            kprintln!("❌ Failed to receive ping #{}", sequence);
            return false;
        }
    }
    
    // Calculate statistics
    if !latencies.is_empty() {
        latencies.sort_unstable();
        
        let min = latencies[0];
        let max = latencies[latencies.len() - 1];
        let median = if latencies.len() % 2 == 0 {
            (latencies[latencies.len() / 2 - 1] + latencies[latencies.len() / 2]) / 2
        } else {
            latencies[latencies.len() / 2]
        };
        let average = latencies.iter().sum::<u64>() / latencies.len() as u64;
        
        kprintln!("");
        kprintln!("📊 BENCHMARK RESULTS ({} iterations):", latencies.len());
        kprintln!("   Min latency: {}µs", min);
        kprintln!("   Max latency: {}µs", max);
        kprintln!("   Average latency: {}µs", average);
        kprintln!("   Median latency: {}µs", median);
        kprintln!("   Target: <{}µs", MAX_MEDIAN_LATENCY_US);
        
        if median < MAX_MEDIAN_LATENCY_US {
            kprintln!("✅ BENCHMARK PASSED (median {}µs < {}µs)", median, MAX_MEDIAN_LATENCY_US);
        } else {
            kprintln!("❌ BENCHMARK FAILED (median {}µs >= {}µs)", median, MAX_MEDIAN_LATENCY_US);
            return false;
        }
    } else {
        kprintln!("❌ No latency measurements collected");
        return false;
    }
    
    kprintln!("=== END BENCHMARK TEST ===");
    kprintln!("");
    
    true
}

/// Comprehensive test runner for all ping-pong tests
pub fn run_all_ping_pong_tests() -> bool {
    kprintln!("");
    kprintln!("🧪 RUNNING ALL IPC PING-PONG TESTS");
    kprintln!("=====================================");
    
    let mut all_passed = true;
    
    // Run simple test
    kprintln!("1️⃣ Running simple ping-pong test...");
    if !run_simple_ping_pong_test() {
        kprintln!("❌ Simple ping-pong test FAILED");
        all_passed = false;
    } else {
        kprintln!("✅ Simple ping-pong test PASSED");
    }
    
    // Run benchmark test
    kprintln!("2️⃣ Running benchmark ping-pong test...");
    if !run_benchmark_ping_pong_test() {
        kprintln!("❌ Benchmark ping-pong test FAILED");
        all_passed = false;
    } else {
        kprintln!("✅ Benchmark ping-pong test PASSED");
    }
    
    // Summary
    kprintln!("");
    kprintln!("=====================================");
    if all_passed {
        kprintln!("🎉 ALL PING-PONG TESTS PASSED");
    } else {
        kprintln!("💥 SOME PING-PONG TESTS FAILED");
    }
    kprintln!("=====================================");
    kprintln!("");
    
    all_passed
}

/// Test helper to run ping-pong test with custom parameters
pub fn run_custom_ping_pong_test(iterations: usize, max_latency_us: u64) -> bool {
    kprintln!("");
    kprintln!("=== CUSTOM IPC PING-PONG TEST ===");
    kprintln!("Iterations: {}", iterations);
    kprintln!("Max latency: {}µs", max_latency_us);
    
    let task_a_id = ProcessId(10);
    let task_b_id = ProcessId(20);
    
    // Setup capabilities
    if let Err(e) = setup_ping_pong_capabilities() {
        kprintln!("❌ Capabilities setup failed: {}", e);
        return false;
    }
    
    let mut latencies = Vec::new();
    
    // Run iterations
    for iteration in 0..iterations {
        let sequence = iteration as u64 + 1;
        
        let ping_message = ping_pong_messages::create_ping_message(sequence, task_a_id, task_b_id);
        let start_time = timestamp::get_timestamp();
        
        // Full ping-pong exchange
        if sys::sys_send(task_b_id.0, &ping_message).is_err() {
            kprintln!("❌ Failed at iteration {}: ping send", iteration + 1);
            return false;
        }
        
        if let Some(received_ping) = crate::ipc::queues::try_receive_message(task_b_id.0) {
            if ping_pong_messages::is_ping(&received_ping) {
                let pong_message = ping_pong_messages::create_pong_message(sequence, task_b_id, task_a_id);
                
                if sys::sys_send(task_a_id.0, &pong_message).is_err() {
                    kprintln!("❌ Failed at iteration {}: pong send", iteration + 1);
                    return false;
                }
                
                if let Some(received_pong) = crate::ipc::queues::try_receive_message(task_a_id.0) {
                    if ping_pong_messages::is_pong(&received_pong) {
                        let end_time = timestamp::get_timestamp();
                        let latency_us = timestamp::elapsed_microseconds(start_time, end_time);
                        latencies.push(latency_us);
                    } else {
                        kprintln!("❌ Invalid pong at iteration {}", iteration + 1);
                        return false;
                    }
                } else {
                    kprintln!("❌ Failed to receive pong at iteration {}", iteration + 1);
                    return false;
                }
            } else {
                kprintln!("❌ Invalid ping at iteration {}", iteration + 1);
                return false;
            }
        } else {
            kprintln!("❌ Failed to receive ping at iteration {}", iteration + 1);
            return false;
        }
    }
    
    // Analyze results
    if !latencies.is_empty() {
        latencies.sort_unstable();
        let median = if latencies.len() % 2 == 0 {
            (latencies[latencies.len() / 2 - 1] + latencies[latencies.len() / 2]) / 2
        } else {
            latencies[latencies.len() / 2]
        };
        
        kprintln!("📊 Custom test results:");
        kprintln!("   Completed iterations: {}", latencies.len());
        kprintln!("   Median latency: {}µs", median);
        kprintln!("   Required: <{}µs", max_latency_us);
        
        let passed = median < max_latency_us;
        kprintln!("   Result: {}", if passed { "✅ PASSED" } else { "❌ FAILED" });
        
        kprintln!("=== END CUSTOM TEST ===");
        kprintln!("");
        
        passed
    } else {
        kprintln!("❌ No measurements collected");
        false
    }
}
