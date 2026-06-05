//! Chaos Engineering and Metrics Client for Aetheris OS Networking
//! 
//! This module provides TypeScript/Node.js client for chaos injection and
//! metrics collection in the Aetheris OS networking subsystem.

import * as net from 'net';
import * as http from 'http';
import * as fs from 'fs';
import * as path from 'path';
import { performance } from 'perf_hooks';

// Chaos configuration interface
export interface ChaosConfig {
    enabled: boolean;
    seed?: number;
    duration: number; // in milliseconds
    packetLossRate: number; // 0.0 to 1.0
    latencyMs: number;
    latencyJitterMs: number;
    connectionChurnRate: number; // connections per second
    connectionChurnDuration: number; // in milliseconds
    bandwidthLimitMbps?: number;
    cpuStressPercent?: number;
    memoryStressMB?: number;
}

// Chaos type enumeration
export enum ChaosType {
    PacketLoss = 'packet_loss',
    Latency = 'latency',
    Jitter = 'jitter',
    ConnectionChurn = 'connection_churn',
    BandwidthLimit = 'bandwidth_limit',
    CpuStress = 'cpu_stress',
    MemoryStress = 'memory_stress',
}

// Chaos result interface
export interface ChaosResult {
    chaosType: ChaosType;
    applied: boolean;
    duration: number; // in milliseconds
    packetsDropped: number;
    packetsDelayed: number;
    connectionsChurned: number;
    errorCount: number;
}

// Metrics configuration interface
export interface MetricsConfig {
    enabled: boolean;
    exportInterval: number; // in milliseconds
    prometheusPort?: number;
    jsonOutput?: string;
    serviceName: string;
    serviceVersion: string;
}

// Metric value interface
export interface MetricValue {
    type: 'counter' | 'gauge' | 'histogram' | 'summary';
    value: any;
}

// Metric interface
export interface Metric {
    name: string;
    help: string;
    type: 'counter' | 'gauge' | 'histogram' | 'summary';
    labels: Record<string, string>;
    value: MetricValue;
    timestamp: number;
}

// Chaos manager class
export class ChaosManager {
    private config: ChaosConfig;
    private activeInjections: Map<string, ChaosInjection> = new Map();
    private stats: ChaosStats = new ChaosStats();
    private rng: ChaosRng;

    constructor(config: ChaosConfig) {
        this.config = config;
        this.rng = new ChaosRng(config.seed || 42);
    }

    // Start packet loss injection
    async startPacketLoss(rate: number): Promise<string> {
        const injectionId = `packet_loss_${Date.now()}`;
        
        const injection = new ChaosInjection({
            id: injectionId,
            chaosType: ChaosType.PacketLoss,
            config: { ...this.config, packetLossRate: rate },
            startTime: performance.now(),
            stats: this.stats,
            rng: this.rng,
        });

        this.activeInjections.set(injectionId, injection);
        
        // Start packet loss injection
        setInterval(() => {
            if (this.rng.nextBool(rate)) {
                this.stats.packetsDropped++;
            }
        }, 1);

        return injectionId;
    }

    // Start latency injection
    async startLatency(latencyMs: number, jitterMs: number): Promise<string> {
        const injectionId = `latency_${Date.now()}`;
        
        const injection = new ChaosInjection({
            id: injectionId,
            chaosType: ChaosType.Latency,
            config: { ...this.config, latencyMs, latencyJitterMs: jitterMs },
            startTime: performance.now(),
            stats: this.stats,
            rng: this.rng,
        });

        this.activeInjections.set(injectionId, injection);
        
        // Start latency injection
        setInterval(() => {
            const jitter = (this.rng.nextFloat() - 0.5) * 2.0 * jitterMs;
            const totalLatency = latencyMs + jitter;
            if (totalLatency > 0) {
                setTimeout(() => {
                    this.stats.packetsDelayed++;
                }, totalLatency);
            }
        }, 1);

        return injectionId;
    }

    // Start connection churn injection
    async startConnectionChurn(rate: number): Promise<string> {
        const injectionId = `connection_churn_${Date.now()}`;
        
        const injection = new ChaosInjection({
            id: injectionId,
            chaosType: ChaosType.ConnectionChurn,
            config: { ...this.config, connectionChurnRate: rate },
            startTime: performance.now(),
            stats: this.stats,
            rng: this.rng,
        });

        this.activeInjections.set(injectionId, injection);
        
        // Start connection churn injection
        setInterval(() => {
            if (this.rng.nextBool(rate / 1000.0)) { // Convert to per-millisecond probability
                this.stats.connectionsChurned++;
                // Simulate connection churn duration
                setTimeout(() => {}, this.config.connectionChurnDuration);
            }
        }, 1);

        return injectionId;
    }

    // Stop a chaos injection
    async stopInjection(injectionId: string): Promise<ChaosResult> {
        const injection = this.activeInjections.get(injectionId);
        if (!injection) {
            throw new Error(`Injection not found: ${injectionId}`);
        }

        this.activeInjections.delete(injectionId);
        
        const duration = performance.now() - injection.startTime;
        
        return {
            chaosType: injection.chaosType,
            applied: true,
            duration,
            packetsDropped: this.stats.packetsDropped,
            packetsDelayed: this.stats.packetsDelayed,
            connectionsChurned: this.stats.connectionsChurned,
            errorCount: this.stats.errorCount,
        };
    }

    // Stop all chaos injections
    async stopAllInjections(): Promise<ChaosResult[]> {
        const results: ChaosResult[] = [];
        const injectionIds = Array.from(this.activeInjections.keys());
        
        for (const id of injectionIds) {
            try {
                const result = await this.stopInjection(id);
                results.push(result);
            } catch (error) {
                console.error(`Error stopping injection ${id}:`, error);
            }
        }
        
        return results;
    }

    // Check if a packet should be dropped
    shouldDropPacket(): boolean {
        if (!this.config.enabled) {
            return false;
        }
        return this.rng.nextBool(this.config.packetLossRate);
    }

    // Get latency to inject
    getLatencyToInject(): number {
        if (!this.config.enabled || this.config.latencyMs <= 0) {
            return 0;
        }
        
        const jitter = (this.rng.nextFloat() - 0.5) * 2.0 * this.config.latencyJitterMs;
        const totalLatency = this.config.latencyMs + jitter;
        return Math.max(0, totalLatency);
    }

    // Check if a connection should be churned
    shouldChurnConnection(): boolean {
        if (!this.config.enabled) {
            return false;
        }
        return this.rng.nextBool(this.config.connectionChurnRate / 1000.0);
    }
}

// Chaos injection class
class ChaosInjection {
    id: string;
    chaosType: ChaosType;
    config: ChaosConfig;
    startTime: number;
    stats: ChaosStats;
    rng: ChaosRng;

    constructor(params: {
        id: string;
        chaosType: ChaosType;
        config: ChaosConfig;
        startTime: number;
        stats: ChaosStats;
        rng: ChaosRng;
    }) {
        this.id = params.id;
        this.chaosType = params.chaosType;
        this.config = params.config;
        this.startTime = params.startTime;
        this.stats = params.stats;
        this.rng = params.rng;
    }
}

// Chaos statistics class
class ChaosStats {
    packetsDropped: number = 0;
    packetsDelayed: number = 0;
    connectionsChurned: number = 0;
    errorCount: number = 0;
    totalDuration: number = 0;
}

// Simple chaos RNG for deterministic results
class ChaosRng {
    private state: number;

    constructor(seed: number) {
        this.state = seed;
    }

    nextFloat(): number {
        // Linear congruential generator
        this.state = (this.state * 1103515245 + 12345) % 2147483648;
        return this.state / 2147483648;
    }

    nextBool(probability: number): boolean {
        return this.nextFloat() < probability;
    }
}

// Metrics collector class
export class MetricsCollector {
    private config: MetricsConfig;
    private metrics: Map<string, Metric> = new Map();
    private exporters: MetricsExporter[] = [];

    constructor(config: MetricsConfig) {
        this.config = config;
    }

    // Add an exporter
    addExporter(exporter: MetricsExporter): void {
        this.exporters.push(exporter);
    }

    // Record a counter metric
    recordCounter(name: string, help: string, value: number, labels: Record<string, string> = {}): void {
        if (!this.config.enabled) {
            return;
        }

        const metric: Metric = {
            name,
            help,
            type: 'counter',
            labels,
            value: { type: 'counter', value },
            timestamp: Date.now(),
        };

        this.metrics.set(name, metric);
    }

    // Record a gauge metric
    recordGauge(name: string, help: string, value: number, labels: Record<string, string> = {}): void {
        if (!this.config.enabled) {
            return;
        }

        const metric: Metric = {
            name,
            help,
            type: 'gauge',
            labels,
            value: { type: 'gauge', value },
            timestamp: Date.now(),
        };

        this.metrics.set(name, metric);
    }

    // Record a histogram metric
    recordHistogram(name: string, help: string, values: number[], labels: Record<string, string> = {}): void {
        if (!this.config.enabled) {
            return;
        }

        // Create histogram buckets
        const buckets = this.createHistogramBuckets(values);
        const count = values.length;
        const sum = values.reduce((a, b) => a + b, 0);

        const metric: Metric = {
            name,
            help,
            type: 'histogram',
            labels,
            value: { 
                type: 'histogram', 
                value: { buckets, count, sum } 
            },
            timestamp: Date.now(),
        };

        this.metrics.set(name, metric);
    }

    // Record a summary metric
    recordSummary(name: string, help: string, values: number[], labels: Record<string, string> = {}): void {
        if (!this.config.enabled) {
            return;
        }

        const sortedValues = [...values].sort((a, b) => a - b);
        const quantiles = this.calculateQuantiles(sortedValues);
        const count = values.length;
        const sum = values.reduce((a, b) => a + b, 0);

        const metric: Metric = {
            name,
            help,
            type: 'summary',
            labels,
            value: { 
                type: 'summary', 
                value: { quantiles, count, sum } 
            },
            timestamp: Date.now(),
        };

        this.metrics.set(name, metric);
    }

    // Export metrics to all exporters
    async exportMetrics(): Promise<void> {
        const metrics = Array.from(this.metrics.values());
        
        for (const exporter of this.exporters) {
            await exporter.export(metrics);
        }
    }

    // Get all metrics
    getMetrics(): Metric[] {
        return Array.from(this.metrics.values());
    }

    // Create histogram buckets
    private createHistogramBuckets(values: number[]): Array<{ upperBound: number; count: number }> {
        const bucketBoundaries = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0, Infinity];
        const buckets: Array<{ upperBound: number; count: number }> = [];

        for (const boundary of bucketBoundaries) {
            const count = values.filter(v => v <= boundary).length;
            buckets.push({ upperBound: boundary, count });
        }

        return buckets;
    }

    // Calculate quantiles
    private calculateQuantiles(sortedValues: number[]): Array<{ quantile: number; value: number }> {
        const quantiles = [0.5, 0.9, 0.95, 0.99];
        const result: Array<{ quantile: number; value: number }> = [];

        for (const quantile of quantiles) {
            const index = Math.floor(quantile * (sortedValues.length - 1));
            const value = sortedValues[index] || 0;
            result.push({ quantile, value });
        }

        return result;
    }
}

// Metrics exporter interface
export interface MetricsExporter {
    export(metrics: Metric[]): Promise<void>;
}

// Prometheus exporter class
export class PrometheusExporter implements MetricsExporter {
    private port: number;
    private server?: http.Server;

    constructor(port: number) {
        this.port = port;
    }

    async export(metrics: Metric[]): Promise<void> {
        // Start HTTP server for Prometheus metrics
        if (!this.server) {
            this.server = http.createServer((req, res) => {
                if (req.url === '/metrics') {
                    res.setHeader('Content-Type', 'text/plain; version=0.0.4; charset=utf-8');
                    
                    let output = '';
                    
                    // Group metrics by type
                    const counters = metrics.filter(m => m.type === 'counter');
                    const gauges = metrics.filter(m => m.type === 'gauge');
                    const histograms = metrics.filter(m => m.type === 'histogram');
                    const summaries = metrics.filter(m => m.type === 'summary');
                    
                    // Export counters
                    for (const metric of counters) {
                        output += `# HELP ${metric.name} ${metric.help}\n`;
                        output += `# TYPE ${metric.name} counter\n`;
                        
                        const labelsStr = Object.keys(metric.labels).length > 0 
                            ? `{${Object.entries(metric.labels).map(([k, v]) => `${k}="${v}"`).join(',')}}`
                            : '';
                        
                        output += `${metric.name}${labelsStr} ${metric.value.value}\n`;
                    }
                    
                    // Export gauges
                    for (const metric of gauges) {
                        output += `# HELP ${metric.name} ${metric.help}\n`;
                        output += `# TYPE ${metric.name} gauge\n`;
                        
                        const labelsStr = Object.keys(metric.labels).length > 0 
                            ? `{${Object.entries(metric.labels).map(([k, v]) => `${k}="${v}"`).join(',')}}`
                            : '';
                        
                        output += `${metric.name}${labelsStr} ${metric.value.value}\n`;
                    }
                    
                    // Export histograms
                    for (const metric of histograms) {
                        output += `# HELP ${metric.name} ${metric.help}\n`;
                        output += `# TYPE ${metric.name} histogram\n`;
                        
                        const labelsStr = Object.keys(metric.labels).length > 0 
                            ? `{${Object.entries(metric.labels).map(([k, v]) => `${k}="${v}"`).join(',')}}`
                            : '';
                        
                        const histData = metric.value.value as { buckets: Array<{ upperBound: number; count: number }>; count: number; sum: number };
                        
                        // Export buckets
                        for (const bucket of histData.buckets) {
                            output += `${metric.name}_bucket${labelsStr}le="${bucket.upperBound}" ${bucket.count}\n`;
                        }
                        
                        // Export count and sum
                        output += `${metric.name}_count${labelsStr} ${histData.count}\n`;
                        output += `${metric.name}_sum${labelsStr} ${histData.sum}\n`;
                    }
                    
                    // Export summaries
                    for (const metric of summaries) {
                        output += `# HELP ${metric.name} ${metric.help}\n`;
                        output += `# TYPE ${metric.name} summary\n`;
                        
                        const labelsStr = Object.keys(metric.labels).length > 0 
                            ? `{${Object.entries(metric.labels).map(([k, v]) => `${k}="${v}"`).join(',')}}`
                            : '';
                        
                        const summaryData = metric.value.value as { quantiles: Array<{ quantile: number; value: number }>; count: number; sum: number };
                        
                        // Export quantiles
                        for (const quantile of summaryData.quantiles) {
                            output += `${metric.name}{quantile="${quantile.quantile}"} ${quantile.value}\n`;
                        }
                        
                        // Export count and sum
                        output += `${metric.name}_count${labelsStr} ${summaryData.count}\n`;
                        output += `${metric.name}_sum${labelsStr} ${summaryData.sum}\n`;
                    }
                    
                    res.end(output);
                } else {
                    res.statusCode = 404;
                    res.end('Not Found');
                }
            });
            
            this.server.listen(this.port, () => {
                console.log(`Prometheus metrics server listening on port ${this.port}`);
            });
        }
    }
}

// JSON file exporter class
export class JsonFileExporter implements MetricsExporter {
    private outputFile: string;

    constructor(outputFile: string) {
        this.outputFile = outputFile;
    }

    async export(metrics: Metric[]): Promise<void> {
        // Create output directory if it doesn't exist
        const dir = path.dirname(this.outputFile);
        await fs.promises.mkdir(dir, { recursive: true });

        // Write metrics to JSON file
        const data = JSON.stringify(metrics, null, 2);
        await fs.promises.writeFile(this.outputFile, data, 'utf8');
    }
}

// Network metrics collector class
export class NetworkMetricsCollector {
    private collector: MetricsCollector;
    private requestsTotal: number = 0;
    private errorsTotal: number = 0;
    private bytesTxTotal: number = 0;
    private bytesRxTotal: number = 0;
    private latencySamples: number[] = [];
    private cpuUsage: number = 0;
    private memoryUsage: number = 0;

    constructor(config: MetricsConfig) {
        this.collector = new MetricsCollector(config);
        
        // Add exporters
        if (config.prometheusPort) {
            this.collector.addExporter(new PrometheusExporter(config.prometheusPort));
        }
        
        if (config.jsonOutput) {
            this.collector.addExporter(new JsonFileExporter(config.jsonOutput));
        }
    }

    // Record a network request
    recordRequest(latencyMs: number, bytesTx: number, bytesRx: number): void {
        this.requestsTotal++;
        this.bytesTxTotal += bytesTx;
        this.bytesRxTotal += bytesRx;
        this.latencySamples.push(latencyMs);
    }

    // Record a network error
    recordError(): void {
        this.errorsTotal++;
    }

    // Update system metrics
    updateSystemMetrics(cpuPct: number, memoryMB: number): void {
        this.cpuUsage = cpuPct;
        this.memoryUsage = memoryMB;
    }

    // Export network metrics
    async exportNetworkMetrics(): Promise<void> {
        // Record counter metrics
        this.collector.recordCounter(
            'network_requests_total',
            'Total number of network requests',
            this.requestsTotal
        );

        this.collector.recordCounter(
            'network_errors_total',
            'Total number of network errors',
            this.errorsTotal
        );

        this.collector.recordCounter(
            'network_bytes_tx_total',
            'Total bytes transmitted',
            this.bytesTxTotal
        );

        this.collector.recordCounter(
            'network_bytes_rx_total',
            'Total bytes received',
            this.bytesRxTotal
        );

        // Record gauge metrics
        this.collector.recordGauge(
            'network_cpu_usage_percent',
            'CPU usage percentage',
            this.cpuUsage
        );

        this.collector.recordGauge(
            'network_memory_usage_mb',
            'Memory usage in MB',
            this.memoryUsage
        );

        // Record latency histogram
        if (this.latencySamples.length > 0) {
            this.collector.recordHistogram(
                'network_latency_ms',
                'Network latency in milliseconds',
                this.latencySamples
            );
        }

        // Export all metrics
        await this.collector.exportMetrics();
    }

    // Start the metrics collection loop
    startCollectionLoop(): void {
        setInterval(async () => {
            try {
                await this.exportNetworkMetrics();
            } catch (error) {
                console.error('Failed to export network metrics:', error);
            }
        }, this.collector['config'].exportInterval);
    }
}

// Main function for CLI usage
export async function main(): Promise<void> {
    const args = process.argv.slice(2);
    
    if (args.length === 0 || args[0] === '--help') {
        console.log(`
TypeScript Chaos Engineering and Metrics for Aetheris OS

Usage: node net-chaos.ts [command] [options]

Commands:
  chaos     Chaos engineering and fault injection
  metrics   Metrics collection and export

Examples:
  node net-chaos.ts chaos --loss 0.05 --latency 100 --duration 60000
  node net-chaos.ts metrics --port 9090 --json metrics.json
        `);
        return;
    }
    
    const command = args[0];
    
    if (command === 'chaos') {
        // Parse chaos command arguments
        const lossRate = parseFloat(args.find(arg => arg.startsWith('--loss'))?.split('=')[1] || '0');
        const latency = parseFloat(args.find(arg => arg.startsWith('--latency'))?.split('=')[1] || '0');
        const duration = parseInt(args.find(arg => arg.startsWith('--duration'))?.split('=')[1] || '60000');
        
        const config: ChaosConfig = {
            enabled: true,
            seed: 42,
            duration,
            packetLossRate: lossRate,
            latencyMs: latency,
            latencyJitterMs: latency * 0.1,
            connectionChurnRate: 0,
            connectionChurnDuration: 1000,
        };
        
        const manager = new ChaosManager(config);
        
        console.log('Starting chaos injection...');
        
        if (lossRate > 0) {
            await manager.startPacketLoss(lossRate);
            console.log(`Started packet loss injection: ${lossRate * 100}%`);
        }
        
        if (latency > 0) {
            await manager.startLatency(latency, latency * 0.1);
            console.log(`Started latency injection: ${latency}ms`);
        }
        
        console.log(`Chaos injection running for ${duration}ms...`);
        
        // Wait for duration
        await new Promise(resolve => setTimeout(resolve, duration));
        
        // Stop all injections
        const results = await manager.stopAllInjections();
        
        console.log('\nChaos Injection Results:');
        console.log('========================');
        for (const result of results) {
            console.log(`Type: ${result.chaosType}`);
            console.log(`Applied: ${result.applied}`);
            console.log(`Duration: ${result.duration}ms`);
            console.log(`Packets Dropped: ${result.packetsDropped}`);
            console.log(`Packets Delayed: ${result.packetsDelayed}`);
            console.log(`Connections Churned: ${result.connectionsChurned}`);
            console.log(`Errors: ${result.errorCount}`);
            console.log();
        }
        
    } else if (command === 'metrics') {
        // Parse metrics command arguments
        const port = parseInt(args.find(arg => arg.startsWith('--port'))?.split('=')[1] || '9090');
        const jsonOutput = args.find(arg => arg.startsWith('--json'))?.split('=')[1] || 'metrics.json';
        
        const config: MetricsConfig = {
            enabled: true,
            exportInterval: 15000,
            prometheusPort: port,
            jsonOutput,
            serviceName: 'aetheris-net',
            serviceVersion: '1.0.0',
        };
        
        const collector = new NetworkMetricsCollector(config);
        
        console.log(`Starting metrics collection...`);
        console.log(`Prometheus metrics available at: http://localhost:${port}/metrics`);
        console.log(`JSON metrics output: ${jsonOutput}`);
        
        // Start collection loop
        collector.startCollectionLoop();
        
        // Simulate some network activity
        setInterval(() => {
            const latency = 10 + Math.random() * 100;
            const bytesTx = 1024 + Math.floor(Math.random() * 1000);
            const bytesRx = 1024 + Math.floor(Math.random() * 1000);
            
            collector.recordRequest(latency, bytesTx, bytesRx);
            
            // Simulate occasional errors
            if (Math.random() < 0.05) {
                collector.recordError();
            }
            
            // Update system metrics
            const cpuUsage = 25 + Math.random() * 50;
            const memoryUsage = 128 + Math.random() * 100;
            collector.updateSystemMetrics(cpuUsage, memoryUsage);
        }, 1000);
        
        // Keep the process running
        process.on('SIGINT', () => {
            console.log('\nStopping metrics collection...');
            process.exit(0);
        });
        
    } else {
        console.error(`Unknown command: ${command}`);
        process.exit(1);
    }
}

// Run main function if this file is executed directly
if (require.main === module) {
    main().catch(console.error);
}
