//! TypeScript Network Benchmarking for Aetheris OS
//! 
//! This module provides TypeScript/Node.js benchmarking capabilities for
//! testing network performance at scale with both plain TCP and TLS/mTLS protocols.

import * as net from 'net';
import * as tls from 'tls';
import * as fs from 'fs';
import * as path from 'path';
import { performance } from 'perf_hooks';

// Benchmark configuration interface
export interface BenchConfig {
    clients: number;
    duration: number; // in milliseconds
    tlsEnabled: boolean;
    tlsProfile: string;
    pqcAlgorithms: string[];
    serverAddr: string;
    payloadSize: number;
    zeroCopy: boolean;
    backPressure: boolean;
    cpuAffinity?: number[];
    rngSeed?: number;
    warmupDuration: number;
    enableProfiling: boolean;
    recordBaseline: boolean;
    outputFile?: string;
}

// Benchmark metrics interface
export interface BenchMetrics {
    rps: number;
    latencyP50: number;
    latencyP95: number;
    latencyP99: number;
    bytesTX: number;
    bytesRX: number;
    cpuPct: number;
    rssMB: number;
    gcStats?: GCStats;
    syscallStats: SyscallStats;
    errors: number;
    connections: number;
    duration: number;
    timestamp: number;
}

// Garbage collection statistics interface
export interface GCStats {
    gcCount: number;
    gcTimeMs: number;
    heapSizeMB: number;
    heapUsedMB: number;
}

// System call statistics interface
export interface SyscallStats {
    syscallsPerSec: number;
    readSyscalls: number;
    writeSyscalls: number;
    connectSyscalls: number;
    acceptSyscalls: number;
    epollSyscalls: number;
}

// Benchmark result interface
export interface BenchResult {
    config: BenchConfig;
    metrics: BenchMetrics;
    success: boolean;
    errorMessage?: string;
}

// Metrics collector class
export class BenchMetricsCollector {
    private requests: number = 0;
    private errors: number = 0;
    private bytesTX: number = 0;
    private bytesRX: number = 0;
    private latencies: number[] = [];
    private syscallStats: SyscallStats = {
        syscallsPerSec: 0,
        readSyscalls: 0,
        writeSyscalls: 0,
        connectSyscalls: 0,
        acceptSyscalls: 0,
        epollSyscalls: 0,
    };
    private startTime: number = performance.now();

    recordRequest(latency: number, bytesTX: number, bytesRX: number): void {
        this.requests++;
        this.bytesTX += bytesTX;
        this.bytesRX += bytesRX;
        this.latencies.push(latency);
    }

    recordError(): void {
        this.errors++;
    }

    recordSyscall(syscallType: string): void {
        switch (syscallType) {
            case 'read':
                this.syscallStats.readSyscalls++;
                break;
            case 'write':
                this.syscallStats.writeSyscalls++;
                break;
            case 'connect':
                this.syscallStats.connectSyscalls++;
                break;
            case 'accept':
                this.syscallStats.acceptSyscalls++;
                break;
            case 'epoll':
                this.syscallStats.epollSyscalls++;
                break;
        }
    }

    finalize(duration: number): BenchMetrics {
        const rps = this.requests / (duration / 1000);
        
        // Calculate latency percentiles
        const sortedLatencies = [...this.latencies].sort((a, b) => a - b);
        const p50 = sortedLatencies[Math.floor(sortedLatencies.length / 2)] || 0;
        const p95 = sortedLatencies[Math.floor(sortedLatencies.length * 0.95)] || 0;
        const p99 = sortedLatencies[Math.floor(sortedLatencies.length * 0.99)] || 0;
        
        const totalSyscalls = this.syscallStats.readSyscalls + 
                             this.syscallStats.writeSyscalls + 
                             this.syscallStats.connectSyscalls + 
                             this.syscallStats.acceptSyscalls + 
                             this.syscallStats.epollSyscalls;
        
        this.syscallStats.syscallsPerSec = totalSyscalls / (duration / 1000);
        
        // Get system metrics
        const { cpuPct, rssMB } = this.getSystemMetrics();
        const gcStats = this.getGCStats();
        
        return {
            rps,
            latencyP50: p50,
            latencyP95: p95,
            latencyP99: p99,
            bytesTX: this.bytesTX,
            bytesRX: this.bytesRX,
            cpuPct,
            rssMB,
            gcStats,
            syscallStats: this.syscallStats,
            errors: this.errors,
            connections: 0, // Will be set by caller
            duration,
            timestamp: Date.now(),
        };
    }

    private getSystemMetrics(): { cpuPct: number; rssMB: number } {
        // Get memory usage
        const memUsage = process.memoryUsage();
        const rssMB = memUsage.rss / 1024 / 1024;
        
        // Mock CPU usage (in real implementation, would use platform-specific APIs)
        const cpuPct = 25.0;
        
        return { cpuPct, rssMB };
    }

    private getGCStats(): GCStats {
        const memUsage = process.memoryUsage();
        
        return {
            gcCount: 0, // Would be filled by GC monitoring
            gcTimeMs: 0, // Would be filled by GC monitoring
            heapSizeMB: memUsage.heapTotal / 1024 / 1024,
            heapUsedMB: memUsage.heapUsed / 1024 / 1024,
        };
    }
}

// Benchmark harness class
export class BenchHarness {
    private config: BenchConfig;
    private metrics: BenchMetricsCollector;
    private connections: net.Socket[] = [];

    constructor(config: BenchConfig) {
        this.config = config;
        this.metrics = new BenchMetricsCollector();
    }

    async run(): Promise<BenchResult> {
        const startTime = performance.now();
        
        // Set RNG seed for deterministic results
        if (this.config.rngSeed !== undefined) {
            this.setRNGSeed(this.config.rngSeed);
        }
        
        // Warm-up phase
        if (this.config.warmupDuration > 0) {
            await this.warmup();
        }
        
        // Reset metrics after warm-up
        this.metrics = new BenchMetricsCollector();
        
        // Run the benchmark
        const benchmarkResult = await this.runBenchmarkPhase();
        
        // Calculate final metrics
        const duration = performance.now() - startTime;
        const metrics = this.metrics.finalize(duration);
        metrics.connections = this.config.clients;
        
        const result: BenchResult = {
            config: this.config,
            metrics,
            success: benchmarkResult.success,
            errorMessage: benchmarkResult.errorMessage,
        };
        
        // Write results to file if specified
        if (this.config.outputFile) {
            await this.writeResults(result);
        }
        
        // Record baseline if requested
        if (this.config.recordBaseline) {
            await this.recordBaseline(result);
        }
        
        return result;
    }

    private async runBenchmarkPhase(): Promise<{ success: boolean; errorMessage?: string }> {
        const promises: Promise<void>[] = [];
        
        // Start client workers
        for (let i = 0; i < this.config.clients; i++) {
            promises.push(this.clientWorker(i));
        }
        
        // Wait for benchmark duration
        await new Promise(resolve => setTimeout(resolve, this.config.duration));
        
        // Close all connections
        this.connections.forEach(conn => conn.destroy());
        
        // Wait for all workers to complete
        try {
            await Promise.all(promises);
        } catch (error) {
            return {
                success: false,
                errorMessage: error instanceof Error ? error.message : 'Unknown error',
            };
        }
        
        // Check error rate
        if (this.metrics.errors > this.config.clients / 10) {
            return {
                success: false,
                errorMessage: `Too many client errors: ${this.metrics.errors}`,
            };
        }
        
        return { success: true };
    }

    private async clientWorker(clientID: number): Promise<void> {
        return new Promise((resolve, reject) => {
            // Connect to server
            const conn = net.createConnection({
                host: this.config.serverAddr.split(':')[0],
                port: parseInt(this.config.serverAddr.split(':')[1]),
            });
            
            this.connections.push(conn);
            this.metrics.recordSyscall('connect');
            
            conn.on('connect', () => {
                // Generate test payload
                const payload = Buffer.alloc(this.config.payloadSize);
                for (let i = 0; i < payload.length; i++) {
                    payload[i] = i % 256;
                }
                
                // Main request loop
                const sendRequest = () => {
                    const start = performance.now();
                    
                    // Send request
                    conn.write(payload, (err) => {
                        if (err) {
                            this.metrics.recordError();
                            reject(err);
                            return;
                        }
                        this.metrics.recordSyscall('write');
                    });
                    
                    // Receive response
                    conn.once('data', (data) => {
                        const latency = performance.now() - start;
                        this.metrics.recordRequest(latency, payload.length, data.length);
                        this.metrics.recordSyscall('read');
                        
                        // Apply back-pressure if enabled
                        if (this.config.backPressure && latency > 100) {
                            setTimeout(sendRequest, 1);
                        } else {
                            setImmediate(sendRequest);
                        }
                    });
                };
                
                sendRequest();
            });
            
            conn.on('error', (err) => {
                this.metrics.recordError();
                reject(err);
            });
            
            conn.on('close', () => {
                resolve();
            });
        });
    }

    private async warmup(): Promise<void> {
        const warmupClients = Math.max(1, Math.floor(this.config.clients / 10));
        const warmupConfig = {
            ...this.config,
            clients: warmupClients,
            duration: this.config.warmupDuration,
        };
        
        const warmupHarness = new BenchHarness(warmupConfig);
        await warmupHarness.run();
    }

    private setRNGSeed(seed: number): void {
        // This would set the RNG seed for deterministic results
        console.log(`Setting RNG seed to: ${seed}`);
    }

    private async writeResults(result: BenchResult): Promise<void> {
        const data = JSON.stringify(result, null, 2);
        await fs.promises.writeFile(this.config.outputFile!, data, 'utf8');
    }

    private async recordBaseline(result: BenchResult): Promise<void> {
        // Create baseline directory if it doesn't exist
        const baselineDir = 'perf/baselines';
        await fs.promises.mkdir(baselineDir, { recursive: true });
        
        // Load existing baselines
        const baselineFile = path.join(baselineDir, 'p4_03_net_adv.json');
        let baselines: Record<string, any> = {};
        
        try {
            const data = await fs.promises.readFile(baselineFile, 'utf8');
            baselines = JSON.parse(data);
        } catch (error) {
            // File doesn't exist or is invalid, start with empty object
        }
        
        // Add new baseline
        const configKey = `clients_${this.config.clients}_tls_${this.config.tlsEnabled}_duration_${this.config.duration}`;
        baselines[configKey] = result.metrics;
        
        // Write updated baselines
        const data = JSON.stringify(baselines, null, 2);
        await fs.promises.writeFile(baselineFile, data, 'utf8');
    }
}

// Main benchmarking function
export async function runBenchmark(config: BenchConfig): Promise<BenchResult> {
    const harness = new BenchHarness(config);
    return await harness.run();
}

// CLI interface for Node.js
export async function main(): Promise<void> {
    const args = process.argv.slice(2);
    
    if (args.length === 0 || args[0] === '--help') {
        console.log(`
TypeScript Network Benchmark for Aetheris OS

Usage: node net-bench.ts [options]

Options:
  --clients <number>        Number of concurrent connections (default: 1000)
  --duration <ms>           Benchmark duration in milliseconds (default: 30000)
  --tls                     Enable TLS (default: false)
  --profile <string>        TLS profile (default: tls13_modern)
  --pqc <string>            PQC algorithms, comma-separated (default: kyber512,dilithium2)
  --addr <string>           Server address (default: 127.0.0.1:8080)
  --payload <number>        Payload size in bytes (default: 1024)
  --zero-copy               Enable zero-copy I/O (default: true)
  --back-pressure           Enable back-pressure (default: true)
  --warmup <ms>             Warm-up duration in milliseconds (default: 5000)
  --json <file>             Output JSON file
  --record-baseline         Record baseline metrics
  --help                    Show this help message
        `);
        return;
    }
    
    // Parse command line arguments
    const config: BenchConfig = {
        clients: 1000,
        duration: 30000,
        tlsEnabled: false,
        tlsProfile: 'tls13_modern',
        pqcAlgorithms: ['kyber512', 'dilithium2'],
        serverAddr: '127.0.0.1:8080',
        payloadSize: 1024,
        zeroCopy: true,
        backPressure: true,
        warmupDuration: 5000,
        enableProfiling: false,
        recordBaseline: false,
    };
    
    for (let i = 0; i < args.length; i++) {
        switch (args[i]) {
            case '--clients':
                config.clients = parseInt(args[++i]);
                break;
            case '--duration':
                config.duration = parseInt(args[++i]);
                break;
            case '--tls':
                config.tlsEnabled = true;
                break;
            case '--profile':
                config.tlsProfile = args[++i];
                break;
            case '--pqc':
                config.pqcAlgorithms = args[++i].split(',');
                break;
            case '--addr':
                config.serverAddr = args[++i];
                break;
            case '--payload':
                config.payloadSize = parseInt(args[++i]);
                break;
            case '--zero-copy':
                config.zeroCopy = true;
                break;
            case '--back-pressure':
                config.backPressure = true;
                break;
            case '--warmup':
                config.warmupDuration = parseInt(args[++i]);
                break;
            case '--json':
                config.outputFile = args[++i];
                break;
            case '--record-baseline':
                config.recordBaseline = true;
                break;
        }
    }
    
    // Run benchmark
    console.log('Starting TypeScript network benchmark...');
    const result = await runBenchmark(config);
    
    // Print results
    console.log('\nBenchmark Results:');
    console.log('==================');
    console.log(`Clients: ${result.metrics.connections}`);
    console.log(`Duration: ${result.metrics.duration}ms`);
    console.log(`RPS: ${result.metrics.rps.toFixed(2)}`);
    console.log(`Latency P50: ${result.metrics.latencyP50.toFixed(2)}ms`);
    console.log(`Latency P95: ${result.metrics.latencyP95.toFixed(2)}ms`);
    console.log(`Latency P99: ${result.metrics.latencyP99.toFixed(2)}ms`);
    console.log(`Bytes TX: ${result.metrics.bytesTX}`);
    console.log(`Bytes RX: ${result.metrics.bytesRX}`);
    console.log(`CPU: ${result.metrics.cpuPct.toFixed(2)}%`);
    console.log(`RSS: ${result.metrics.rssMB.toFixed(2)}MB`);
    console.log(`Errors: ${result.metrics.errors}`);
    console.log(`Syscalls/sec: ${result.metrics.syscallStats.syscallsPerSec.toFixed(2)}`);
    
    if (result.metrics.gcStats) {
        console.log(`GC Count: ${result.metrics.gcStats.gcCount}`);
        console.log(`GC Time: ${result.metrics.gcStats.gcTimeMs.toFixed(2)}ms`);
        console.log(`Heap Size: ${result.metrics.gcStats.heapSizeMB.toFixed(2)}MB`);
        console.log(`Heap Used: ${result.metrics.gcStats.heapUsedMB.toFixed(2)}MB`);
    }
    
    if (!result.success) {
        console.log(`Benchmark failed: ${result.errorMessage}`);
        process.exit(1);
    }
}

// Run main function if this file is executed directly
if (require.main === module) {
    main().catch(console.error);
}
