//! Phase 2 Soak Test Main Entry Point
//! 
//! This binary runs comprehensive soak testing for Polymera OS Phase 2,
//! including IPC stress, timer jitter, key rotation, and chaos engineering.

use std::env;
use std::process;
use std::time::Duration;

mod soak_p2;

use soak_p2::{SoakTestOrchestrator, default_soak_config, mini_soak_config};

fn main() {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    
    // Parse command line arguments
    let config = if args.len() > 1 && args[1] == "mini" {
        log::info!("Running mini soak test (10 minutes)");
        mini_soak_config()
    } else {
        log::info!("Running full soak test (2 hours)");
        default_soak_config()
    };
    
    // Create soak test orchestrator
    let mut orchestrator = SoakTestOrchestrator::new(config);
    
    // Start the test
    if let Err(e) = orchestrator.start() {
        log::error!("Failed to start soak test: {}", e);
        process::exit(1);
    }
    
    // Wait for completion
    if let Err(e) = orchestrator.wait_for_completion() {
        log::error!("Test completion error: {}", e);
        process::exit(1);
    }
    
    // Stop the test
    if let Err(e) = orchestrator.stop() {
        log::error!("Failed to stop soak test: {}", e);
        process::exit(1);
    }
    
    // Generate and print report
    match orchestrator.generate_report() {
        Ok(report) => {
            println!("{}", report);
            
            // Check if test passed
            let stats = orchestrator.get_stats();
            if stats.test_results.passed {
                log::info!("Soak test PASSED");
                process::exit(0);
            } else {
                log::error!("Soak test FAILED");
                for reason in &stats.test_results.failure_reasons {
                    log::error!("  - {}", reason);
                }
                process::exit(1);
            }
        }
        Err(e) => {
            log::error!("Failed to generate report: {}", e);
            process::exit(1);
        }
    }
}

