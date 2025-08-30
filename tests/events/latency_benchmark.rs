use kernel::event::fabric::{EventFabric, Pattern};
use kernel::event::queue::Lane;
use kernel::secman::cap_flags::*;
use std::time::{Instant, Duration};

/// Measure event fabric latency
#[test]
fn test_event_latency_benchmark() {
    let fabric = EventFabric::new();
    let task_id = 500;
    
    // Subscribe to intent events on HI lane for best performance
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_SUBMIT).unwrap();
    
    // Benchmark parameters
    const NUM_EVENTS: usize = 50_000;
    const WARMUP_EVENTS: usize = 1000;
    
    // Warmup phase
    for i in 0..WARMUP_EVENTS {
        let payload = format!("warmup_{}", i).into_bytes();
        fabric.publish("intent.created", 0, payload).unwrap();
    }
    
    // Clear warmup events
    fabric.poll(task_id, WARMUP_EVENTS).unwrap();
    
    // Benchmark phase
    let mut latencies = Vec::with_capacity(NUM_EVENTS);
    
    for i in 0..NUM_EVENTS {
        let payload = format!("benchmark_{}", i).into_bytes();
        
        // Measure publish time
        let publish_start = Instant::now();
        fabric.publish("intent.created", 0, payload).unwrap();
        let publish_end = Instant::now();
        
        // Measure poll time
        let poll_start = Instant::now();
        let events = fabric.poll(task_id, 1).unwrap();
        let poll_end = Instant::now();
        
        // Calculate total latency (publish + poll)
        let total_latency = publish_start.elapsed() + poll_start.elapsed();
        latencies.push(total_latency);
        
        // Verify event received
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].payload, format!("benchmark_{}", i).into_bytes());
    }
    
    // Calculate statistics
    latencies.sort();
    
    let p50_idx = latencies.len() / 2;
    let p95_idx = (latencies.len() * 95) / 100;
    let p99_idx = (latencies.len() * 99) / 100;
    
    let p50_us = latencies[p50_idx].as_micros();
    let p95_us = latencies[p95_idx].as_micros();
    let p99_us = latencies[p99_idx].as_micros();
    
    let min_us = latencies[0].as_micros();
    let max_us = latencies[latencies.len() - 1].as_micros();
    let avg_us = latencies.iter().map(|d| d.as_micros()).sum::<u128>() / latencies.len() as u128;
    
    // Print JSON metrics line
    println!("{{\"test\":\"event_latency_benchmark\",\"events\":{},\"p50_us\":{},\"p95_us\":{},\"p99_us\":{},\"min_us\":{},\"max_us\":{},\"avg_us\":{}}}",
        NUM_EVENTS, p50_us, p95_us, p99_us, min_us, max_us, avg_us);
    
    // Verify performance targets
    assert!(p95_us <= 400, "p95 latency {}µs exceeds 400µs target", p95_us);
    assert!(avg_us <= 200, "average latency {}µs exceeds 200µs target", avg_us);
    
    // Verify fabric statistics
    let stats = fabric.get_stats();
    assert_eq!(stats.total_events_published, NUM_EVENTS as u64);
    assert_eq!(stats.total_events_delivered, NUM_EVENTS as u64);
    assert_eq!(stats.total_drops_hi, 0); // HI lane should be lossless
}

/// Measure different priority lane performance
#[test]
fn test_lane_performance_comparison() {
    let fabric = EventFabric::new();
    let task_id = 501;
    
    // Subscribe to all lanes
    let pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, pattern.clone(), Lane::HI, CAP_INTENT_SUBMIT).unwrap();
    fabric.subscribe(task_id, pattern.clone(), Lane::MED, CAP_INTENT_SUBMIT).unwrap();
    fabric.subscribe(task_id, pattern, Lane::LO, CAP_INTENT_SUBMIT).unwrap();
    
    const EVENTS_PER_LANE: usize = 10_000;
    
    // Benchmark HI lane
    let hi_start = Instant::now();
    for i in 0..EVENTS_PER_LANE {
        let payload = format!("hi_{}", i).into_bytes();
        fabric.publish("intent.created", 0, payload).unwrap();
    }
    let hi_events = fabric.poll(task_id, EVENTS_PER_LANE).unwrap();
    let hi_duration = hi_start.elapsed();
    
    // Benchmark MED lane
    let med_start = Instant::now();
    for i in 0..EVENTS_PER_LANE {
        let payload = format!("med_{}", i).into_bytes();
        fabric.publish("intent.previewed", 1, payload).unwrap();
    }
    let med_events = fabric.poll(task_id, EVENTS_PER_LANE).unwrap();
    let med_duration = med_start.elapsed();
    
    // Benchmark LO lane
    let lo_start = Instant::now();
    for i in 0..EVENTS_PER_LANE {
        let payload = format!("lo_{}", i).into_bytes();
        fabric.publish("intent.completed", 2, payload).unwrap();
    }
    let lo_events = fabric.poll(task_id, EVENTS_PER_LANE).unwrap();
    let lo_duration = lo_start.elapsed();
    
    // Verify all events received
    assert_eq!(hi_events.len(), EVENTS_PER_LANE);
    assert_eq!(med_events.len(), EVENTS_PER_LANE);
    assert_eq!(lo_events.len(), EVENTS_PER_LANE);
    
    // Calculate throughput
    let hi_throughput = EVENTS_PER_LANE as f64 / hi_duration.as_secs_f64();
    let med_throughput = EVENTS_PER_LANE as f64 / med_duration.as_secs_f64();
    let lo_throughput = EVENTS_PER_LANE as f64 / lo_duration.as_secs_f64();
    
    // Print lane comparison metrics
    println!("{{\"test\":\"lane_performance_comparison\",\"hi_events_per_sec\":{:.0},\"med_events_per_sec\":{:.0},\"lo_events_per_sec\":{:.0}}}",
        hi_throughput, med_throughput, lo_throughput);
    
    // Verify performance characteristics
    // HI lane should be fastest (no drops, simple processing)
    assert!(hi_throughput >= 100_000.0, "HI lane throughput {} events/sec below 100k target", hi_throughput);
    
    // All lanes should be reasonably fast
    assert!(med_throughput >= 50_000.0, "MED lane throughput {} events/sec below 50k target", med_throughput);
    assert!(lo_throughput >= 25_000.0, "LO lane throughput {} events/sec below 25k target", lo_throughput);
}

/// Measure subscription pattern performance
#[test]
fn test_pattern_matching_performance() {
    let fabric = EventFabric::new();
    let task_id = 502;
    
    // Subscribe to exact pattern
    let exact_pattern = Pattern::new("intent.created".to_string()).unwrap();
    fabric.subscribe(task_id, exact_pattern, Lane::HI, CAP_INTENT_SUBMIT).unwrap();
    
    // Subscribe to prefix pattern
    let prefix_pattern = Pattern::new("intent.*".to_string()).unwrap();
    fabric.subscribe(task_id, prefix_pattern, Lane::MED, CAP_INTENT_SUBMIT).unwrap();
    
    const NUM_EVENTS: usize = 20_000;
    
    // Benchmark exact pattern matching
    let exact_start = Instant::now();
    for i in 0..NUM_EVENTS {
        let payload = format!("exact_{}", i).into_bytes();
        fabric.publish("intent.created", 0, payload).unwrap();
    }
    let exact_events = fabric.poll(task_id, NUM_EVENTS).unwrap();
    let exact_duration = exact_start.elapsed();
    
    // Benchmark prefix pattern matching
    let prefix_start = Instant::now();
    for i in 0..NUM_EVENTS {
        let payload = format!("prefix_{}", i).into_bytes();
        fabric.publish("intent.previewed", 1, payload).unwrap();
    }
    let prefix_events = fabric.poll(task_id, NUM_EVENTS).unwrap();
    let prefix_duration = prefix_start.elapsed();
    
    // Verify events received
    assert_eq!(exact_events.len(), NUM_EVENTS);
    assert_eq!(prefix_events.len(), NUM_EVENTS);
    
    // Calculate throughput
    let exact_throughput = NUM_EVENTS as f64 / exact_duration.as_secs_f64();
    let prefix_throughput = NUM_EVENTS as f64 / prefix_duration.as_secs_f64();
    
    // Print pattern matching metrics
    println!("{{\"test\":\"pattern_matching_performance\",\"exact_events_per_sec\":{:.0},\"prefix_events_per_sec\":{:.0}}}",
        exact_throughput, prefix_throughput);
    
    // Verify performance
    assert!(exact_throughput >= 100_000.0, "Exact pattern throughput {} events/sec below 100k target", exact_throughput);
    assert!(prefix_throughput >= 80_000.0, "Prefix pattern throughput {} events/sec below 80k target", prefix_throughput);
}

/// Measure concurrent task performance
#[test]
fn test_concurrent_task_performance() {
    let fabric = EventFabric::new();
    const NUM_TASKS: usize = 10;
    const EVENTS_PER_TASK: usize = 5_000;
    
    // Create multiple tasks
    let task_ids: Vec<u32> = (600..600 + NUM_TASKS as u32).collect();
    
    for &task_id in &task_ids {
        let pattern = Pattern::new("intent.*".to_string()).unwrap();
        fabric.subscribe(task_id, pattern, Lane::HI, CAP_INTENT_SUBMIT).unwrap();
    }
    
    // Publish events to all tasks concurrently
    let start = Instant::now();
    
    for i in 0..EVENTS_PER_TASK {
        let payload = format!("concurrent_{}", i).into_bytes();
        fabric.publish("intent.created", 0, payload).unwrap();
    }
    
    let publish_duration = start.elapsed();
    
    // Poll events from all tasks
    let poll_start = Instant::now();
    let mut total_events = 0;
    
    for &task_id in &task_ids {
        let events = fabric.poll(task_id, EVENTS_PER_TASK).unwrap();
        total_events += events.len();
    }
    
    let poll_duration = poll_start.elapsed();
    let total_duration = start.elapsed();
    
    // Verify all events received
    assert_eq!(total_events, NUM_TASKS * EVENTS_PER_TASK);
    
    // Calculate throughput
    let total_events_processed = NUM_TASKS * EVENTS_PER_TASK;
    let overall_throughput = total_events_processed as f64 / total_duration.as_secs_f64();
    
    // Print concurrent performance metrics
    println!("{{\"test\":\"concurrent_task_performance\",\"tasks\":{},\"events_per_task\":{},\"total_events\":{},\"overall_events_per_sec\":{:.0}}}",
        NUM_TASKS, EVENTS_PER_TASK, total_events_processed, overall_throughput);
    
    // Verify performance
    assert!(overall_throughput >= 200_000.0, "Concurrent throughput {} events/sec below 200k target", overall_throughput);
    
    // Verify fabric statistics
    let stats = fabric.get_stats();
    assert_eq!(stats.total_events_published, total_events_processed as u64);
    assert_eq!(stats.total_events_delivered, total_events_processed as u64);
}
