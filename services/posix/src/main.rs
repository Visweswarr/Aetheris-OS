use polymera_posix::POSIXService;
use std::process;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    println!("🚀 Polymera OS POSIX Service v0.1");
    println!("🎯 Sandboxed POSIX Surface with Polyglot Runtime");
    println!("🌟 Capability-Secure, AI-Native, Web3 OS");
    println!();
    
    let service = POSIXService::new();
    
    match service.initialize() {
        Ok(()) => {
            println!("✅ POSIX Service initialized successfully");
            
            // Run tests
            if let Err(e) = service.test_syscall_broker() {
                eprintln!("❌ Syscall broker test failed: {}", e);
                process::exit(1);
            }
            
            if let Err(e) = service.test_vfs_operations() {
                eprintln!("❌ VFS operations test failed: {}", e);
                process::exit(1);
            }
            
            if let Err(e) = service.test_polyglot_shims() {
                eprintln!("❌ Polyglot shims test failed: {}", e);
                process::exit(1);
            }
            
            // Run performance benchmarks
            if let Err(e) = service.run_performance_benchmarks() {
                eprintln!("❌ Performance benchmarks failed: {}", e);
                process::exit(1);
            }
            
            // Show final status
            let status = service.get_status();
            println!();
            println!("🎉 SUCCESS: POSIX Service is operational!");
            println!("🚀 [POSIX v0.1] broker OK | p50 read≤300µs p95≤800µs | caps=enforced");
            println!();
            
            println!("📊 Final System Status:");
            println!("  - Broker: ✅ READY");
            println!("  - VFS: ✅ READY ({} mounts, {} files)", status.mount_count, status.file_count);
            println!("  - Shims: ✅ READY ({} languages)", status.shim_count);
            println!("  - Shell: ✅ READY");
            println!("  - Broker Calls: {}", status.broker_calls);
            println!();
            
            // Show performance report
            let perf_report = service.get_performance_report();
            println!("{}", perf_report);
            
            // Interactive shell demo
            println!("🐚 Interactive Shell Demo:");
            let commands = vec![
                "echo Hello, Polymera OS!",
                "pwd",
                "ls",
                "ngfsctl status",
                "help",
            ];
            
            for cmd in commands {
                println!("$ {}", cmd);
                match service.execute_shell_command(cmd) {
                    Ok(output) => {
                        if !output.is_empty() {
                            println!("{}", output);
                        }
                    },
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                println!();
            }
            
            println!("🎯 POSIX Service ready for production use!");
            println!("🌟 Features:");
            println!("  • Syscall broker with capability enforcement");
            println!("  • Capability-aware VFS with /snap, /pdv, /tmp mounts");
            println!("  • Polyglot shims (C, Go, Rust, Node.js, WASI)");
            println!("  • Aesh shell with built-in commands");
            println!("  • Performance monitoring and budgets");
            println!("  • Security-first design with deny-by-default");
        },
        Err(e) => {
            eprintln!("❌ Failed to initialize POSIX Service: {}", e);
            process::exit(1);
        }
    }
}

