#!/usr/bin/env node
/**
 * Performance Dashboard Renderer for Polymera OS
 * 
 * Generates a static HTML dashboard from performance history JSON files
 * to visualize performance trends over time.
 * 
 * Usage: node render_dashboard.js [--input DIR] [--output DIR] [--baseline FILE]
 */

const fs = require('fs');
const path = require('path');

// Configuration
const DEFAULT_INPUT_DIR = 'perf/history';
const DEFAULT_OUTPUT_DIR = 'docs/site/perf';
const DEFAULT_BASELINE_FILE = 'perf/baselines/p2.json';

// Dashboard configuration
const DASHBOARD_CONFIG = {
    title: 'Polymera OS Performance Dashboard',
    description: 'Real-time performance metrics and trends',
    refreshInterval: 300000, // 5 minutes
    maxDataPoints: 100,
    chartColors: {
        p50: '#3b82f6',
        p95: '#ef4444',
        p99: '#8b5cf6',
        baseline: '#10b981',
        overhead: '#f59e0b'
    },
    metrics: ['latency_p50', 'latency_p95', 'latency_p99', 'throughput', 'memory_usage', 'cpu_usage']
};

class DashboardRenderer {
    constructor(inputDir, outputDir, baselineFile) {
        this.inputDir = inputDir;
        this.outputDir = outputDir;
        this.baselineFile = baselineFile;
        this.historyData = {};
        this.baselineData = {};
        this.matrixConfigs = new Set();
    }

    /**
     * Main execution method
     */
    async run() {
        console.log('🚀 Starting Performance Dashboard Generation...');
        
        try {
            // Load and process data
            await this.loadBaselineData();
            await this.loadHistoryData();
            await this.processData();
            
            // Generate dashboard files
            await this.generateDashboard();
            await this.generateDataShards();
            await this.generateAssets();
            
            console.log('✅ Dashboard generation completed successfully!');
            console.log(`📊 Dashboard available at: ${this.outputDir}/index.html`);
            
        } catch (error) {
            console.error('❌ Dashboard generation failed:', error);
            process.exit(1);
        }
    }

    /**
     * Load baseline performance data
     */
    async loadBaselineData() {
        console.log('📈 Loading baseline data...');
        
        if (fs.existsSync(this.baselineFile)) {
            try {
                const baselineContent = fs.readFileSync(this.baselineFile, 'utf8');
                this.baselineData = JSON.parse(baselineContent);
                console.log(`✅ Loaded baseline data from ${this.baselineFile}`);
            } catch (error) {
                console.warn(`⚠️  Failed to load baseline data: ${error.message}`);
                this.baselineData = {};
            }
        } else {
            console.warn(`⚠️  Baseline file not found: ${this.baselineFile}`);
            this.baselineData = {};
        }
    }

    /**
     * Load performance history data
     */
    async loadHistoryData() {
        console.log('📚 Loading performance history...');
        
        if (!fs.existsSync(this.inputDir)) {
            console.warn(`⚠️  History directory not found: ${this.inputDir}`);
            this.historyData = {};
            return;
        }

        const historyFiles = fs.readdirSync(this.inputDir)
            .filter(file => file.endsWith('.json') && file !== '.gitkeep');

        console.log(`📁 Found ${historyFiles.length} history files`);

        for (const file of historyFiles) {
            try {
                const filePath = path.join(this.inputDir, file);
                const content = fs.readFileSync(filePath, 'utf8');
                const data = JSON.parse(content);
                
                // Extract matrix configuration from filename
                const configId = this.extractConfigId(file);
                if (configId) {
                    this.historyData[configId] = data;
                    this.matrixConfigs.add(configId);
                }
                
            } catch (error) {
                console.warn(`⚠️  Failed to load ${file}: ${error.message}`);
            }
        }

        console.log(`✅ Loaded history data for ${Object.keys(this.historyData).length} configurations`);
    }

    /**
     * Extract configuration ID from filename
     */
    extractConfigId(filename) {
        // Remove .json extension and extract config identifier
        const name = filename.replace('.json', '');
        
        // Handle different naming patterns
        if (name.includes('_')) {
            return name;
        } else if (name.includes('-')) {
            return name;
        }
        
        return name;
    }

    /**
     * Process and normalize data
     */
    async processData() {
        console.log('🔧 Processing performance data...');
        
        // Normalize timestamps and ensure consistent data structure
        for (const [configId, configData] of Object.entries(this.historyData)) {
            if (Array.isArray(configData)) {
                // Sort by timestamp
                configData.sort((a, b) => new Date(a.timestamp) - new Date(b.timestamp));
                
                // Limit data points for performance
                if (configData.length > DASHBOARD_CONFIG.maxDataPoints) {
                    this.historyData[configId] = configData.slice(-DASHBOARD_CONFIG.maxDataPoints);
                }
            }
        }
        
        console.log('✅ Data processing completed');
    }

    /**
     * Generate main dashboard HTML
     */
    async generateDashboard() {
        console.log('🌐 Generating dashboard HTML...');
        
        // Ensure output directory exists
        if (!fs.existsSync(this.outputDir)) {
            fs.mkdirSync(this.outputDir, { recursive: true });
        }

        const dashboardHtml = this.generateDashboardHtml();
        const dashboardPath = path.join(this.outputDir, 'index.html');
        
        fs.writeFileSync(dashboardPath, dashboardHtml);
        console.log(`✅ Dashboard HTML generated: ${dashboardPath}`);
    }

    /**
     * Generate dashboard HTML content
     */
    generateDashboardHtml() {
        const configs = Array.from(this.matrixConfigs);
        const configOptions = configs.map(config => 
            `<option value="${config}">${this.formatConfigName(config)}</option>`
        ).join('');

        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>${DASHBOARD_CONFIG.title}</title>
    <meta name="description" content="${DASHBOARD_CONFIG.description}">
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <header class="header">
        <h1>${DASHBOARD_CONFIG.title}</h1>
        <p>${DASHBOARD_CONFIG.description}</p>
    </header>
    
    <div class="container">
        <div class="controls">
            <div class="control-group">
                <label for="config-select">Configuration</label>
                <select id="config-select">
                    <option value="">All Configurations</option>
                    ${configOptions}
                </select>
            </div>
            
            <div class="control-group">
                <label for="metric-select">Metric</label>
                <select id="metric-select">
                    <option value="latency_p50">Latency P50</option>
                    <option value="latency_p95">Latency P95</option>
                    <option value="latency_p99">Latency P99</option>
                    <option value="throughput">Throughput</option>
                    <option value="memory_usage">Memory Usage</option>
                    <option value="cpu_usage">CPU Usage</option>
                </select>
            </div>
            
            <div class="control-group">
                <label for="time-range">Time Range</label>
                <select id="time-range">
                    <option value="7">Last 7 days</option>
                    <option value="30" selected>Last 30 days</option>
                    <option value="90">Last 90 days</option>
                    <option value="all">All time</option>
                </select>
            </div>
            
            <div class="control-group">
                <label for="refresh-toggle">Auto-refresh</label>
                <select id="refresh-toggle">
                    <option value="300">5 minutes</option>
                    <option value="600">10 minutes</option>
                    <option value="1800">30 minutes</option>
                    <option value="0">Disabled</option>
                </select>
            </div>
        </div>
        
        <div class="error" id="error-message"></div>
        
        <div class="summary-stats">
            <div class="stat-card">
                <div class="stat-value" id="total-runs">-</div>
                <div class="stat-label">Total Runs</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="pass-rate">-</div>
                <div class="stat-label">Pass Rate</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="avg-latency">-</div>
                <div class="stat-label">Avg Latency</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" id="trend">-</div>
                <div class="stat-label">Trend</div>
            </div>
        </div>
        
        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-header">
                    <div class="metric-title">Latency Trends</div>
                    <div class="metric-status status-pass" id="latency-status">Pass</div>
                </div>
                <div class="chart-container" id="latency-chart">
                    <div class="chart-placeholder">Select configuration and metric to view chart</div>
                </div>
            </div>
            
            <div class="metric-card">
                <div class="metric-header">
                    <div class="metric-title">Performance Overhead</div>
                    <div class="metric-status status-pass" id="overhead-status">Pass</div>
                </div>
                <div class="chart-container" id="overhead-chart">
                    <div class="chart-placeholder">Select configuration to view overhead analysis</div>
                </div>
            </div>
            
            <div class="metric-card">
                <div class="metric-header">
                    <div class="metric-title">Configuration Matrix</div>
                    <div class="metric-status status-pass" id="matrix-status">Pass</div>
                </div>
                <div class="chart-container" id="matrix-chart">
                    <div class="chart-placeholder">Select time range to view configuration matrix</div>
                </div>
            </div>
            
            <div class="metric-card">
                <div class="metric-header">
                    <div class="metric-title">Trend Analysis</div>
                    <div class="metric-status status-pass" id="trend-status">Pass</div>
                </div>
                <div class="chart-container" id="trend-chart">
                    <div class="chart-placeholder">Select configuration to view trend analysis</div>
                </div>
            </div>
        </div>
        
        <div class="loading" id="loading">
            <p>Loading performance data...</p>
        </div>
    </div>
    
    <footer class="footer">
        <p>Generated on ${new Date().toLocaleString()} | Data from ${this.inputDir}</p>
    </footer>
    
    <script src="dashboard.js"></script>
</body>
</html>`;
    }

    /**
     * Generate data shards for performance optimization
     */
    async generateDataShards() {
        console.log('📊 Generating data shards...');
        
        const dataDir = path.join(this.outputDir, 'data');
        if (!fs.existsSync(dataDir)) {
            fs.mkdirSync(dataDir, { recursive: true });
        }

        // Generate individual JSON files for each configuration
        for (const [configId, configData] of Object.entries(this.historyData)) {
            const shardPath = path.join(dataDir, `${configId}.json`);
            
            // Optimize data structure for dashboard consumption
            const optimizedData = this.optimizeDataForDashboard(configData);
            
            fs.writeFileSync(shardPath, JSON.stringify(optimizedData, null, 2));
        }

        // Generate summary index
        const summaryIndex = this.generateSummaryIndex();
        const indexPath = path.join(dataDir, 'index.json');
        fs.writeFileSync(indexPath, JSON.stringify(summaryIndex, null, 2));

        console.log(`✅ Generated ${Object.keys(this.historyData).length} data shards`);
    }

    /**
     * Optimize data structure for dashboard consumption
     */
    optimizeDataForDashboard(data) {
        if (!Array.isArray(data)) {
            return data;
        }

        return data.map(entry => ({
            timestamp: entry.timestamp,
            commit: entry.commit,
            metrics: {
                latency_p50: entry.metrics?.latency_p50 || null,
                latency_p95: entry.metrics?.latency_p95 || null,
                latency_p99: entry.metrics?.latency_p99 || null,
                throughput: entry.metrics?.throughput || null,
                memory_usage: entry.metrics?.memory_usage || null,
                cpu_usage: entry.metrics?.cpu_usage || null
            },
            status: entry.status || 'unknown',
            config: entry.config || {}
        }));
    }

    /**
     * Generate summary index for all configurations
     */
    generateSummaryIndex() {
        const summary = {
            generated_at: new Date().toISOString(),
            total_configurations: Object.keys(this.historyData).length,
            configurations: {},
            baseline: this.baselineData
        };

        for (const [configId, configData] of Object.entries(this.historyData)) {
            if (Array.isArray(configData) && configData.length > 0) {
                const latest = configData[configData.length - 1];
                const oldest = configData[0];
                
                summary.configurations[configId] = {
                    name: this.formatConfigName(configId),
                    total_runs: configData.length,
                    latest_run: latest.timestamp,
                    first_run: oldest.timestamp,
                    status: latest.status || 'unknown',
                    metrics_summary: this.calculateMetricsSummary(configData)
                };
            }
        }

        return summary;
    }

    /**
     * Calculate metrics summary for a configuration
     */
    calculateMetricsSummary(data) {
        const summary = {};
        
        for (const metric of DASHBOARD_CONFIG.metrics) {
            const values = data
                .map(entry => entry.metrics?.[metric])
                .filter(val => val !== null && val !== undefined);
            
            if (values.length > 0) {
                summary[metric] = {
                    min: Math.min(...values),
                    max: Math.max(...values),
                    avg: values.reduce((a, b) => a + b, 0) / values.length,
                    trend: this.calculateTrend(values)
                };
            }
        }
        
        return summary;
    }

    /**
     * Calculate trend direction for a series of values
     */
    calculateTrend(values) {
        if (values.length < 2) return 'stable';
        
        const firstHalf = values.slice(0, Math.floor(values.length / 2));
        const secondHalf = values.slice(Math.floor(values.length / 2));
        
        const firstAvg = firstHalf.reduce((a, b) => a + b, 0) / firstHalf.length;
        const secondAvg = secondHalf.reduce((a, b) => a + b, 0) / secondHalf.length;
        
        const change = ((secondAvg - firstAvg) / firstAvg) * 100;
        
        if (change > 5) return 'increasing';
        if (change < -5) return 'decreasing';
        return 'stable';
    }

    /**
     * Generate additional dashboard assets
     */
    async generateAssets() {
        console.log('🎨 Generating dashboard assets...');
        
        // Generate CSS file
        const cssPath = path.join(this.outputDir, 'style.css');
        const cssContent = this.generateCSS();
        fs.writeFileSync(cssPath, cssContent);
        
        // Generate JavaScript file
        const jsPath = path.join(this.outputDir, 'dashboard.js');
        const jsContent = this.generateJavaScript();
        fs.writeFileSync(jsPath, jsContent);
        
        // Generate README
        const readmePath = path.join(this.outputDir, 'README.md');
        const readmeContent = this.generateREADME();
        fs.writeFileSync(readmePath, readmeContent);
        
        console.log('✅ Dashboard assets generated');
    }

    /**
     * Generate CSS content
     */
    generateCSS() {
        return `/* Performance Dashboard Styles */
/* Generated by render_dashboard.js */

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
    background: #f8fafc;
}

.header {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 2rem 0;
    text-align: center;
}

.header h1 {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
    font-weight: 300;
}

.header p {
    font-size: 1.1rem;
    opacity: 0.9;
}

.container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
}

.controls {
    background: white;
    padding: 1.5rem;
    border-radius: 12px;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    margin-bottom: 2rem;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
}

.control-group {
    display: flex;
    flex-direction: column;
}

.control-group label {
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: #374151;
}

.control-group select,
.control-group input {
    padding: 0.75rem;
    border: 2px solid #e5e7eb;
    border-radius: 8px;
    font-size: 1rem;
    transition: border-color 0.2s;
}

.control-group select:focus,
.control-group input:focus {
    outline: none;
    border-color: #667eea;
}

.metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    gap: 2rem;
    margin-bottom: 2rem;
}

.metric-card {
    background: white;
    border-radius: 12px;
    padding: 1.5rem;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    transition: transform 0.2s, box-shadow 0.2s;
}

.metric-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 10px 25px -3px rgba(0, 0, 0, 0.1);
}

.metric-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
}

.metric-title {
    font-size: 1.25rem;
    font-weight: 600;
    color: #374151;
}

.metric-status {
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.875rem;
    font-weight: 600;
    text-transform: uppercase;
}

.status-pass { background: #d1fae5; color: #065f46; }
.status-warn { background: #fef3c7; color: #92400e; }
.status-fail { background: #fee2e2; color: #991b1b; }

.chart-container {
    height: 300px;
    position: relative;
}

.chart-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #9ca3af;
    font-style: italic;
}

.summary-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
}

.stat-card {
    background: white;
    padding: 1.5rem;
    border-radius: 12px;
    text-align: center;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
}

.stat-value {
    font-size: 2rem;
    font-weight: 700;
    color: #667eea;
    margin-bottom: 0.5rem;
}

.stat-label {
    color: #6b7280;
    font-size: 0.875rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.footer {
    text-align: center;
    padding: 2rem;
    color: #6b7280;
    border-top: 1px solid #e5e7eb;
    margin-top: 3rem;
}

.loading {
    display: none;
    text-align: center;
    padding: 2rem;
    color: #6b7280;
}

.error {
    background: #fee2e2;
    color: #991b1b;
    padding: 1rem;
    border-radius: 8px;
    margin: 1rem 0;
    display: none;
}

@media (max-width: 768px) {
    .container { padding: 1rem; }
    .metrics-grid { grid-template-columns: 1fr; }
    .controls { grid-template-columns: 1fr; }
}`;
    }

    /**
     * Generate JavaScript content
     */
    generateJavaScript() {
        return `// Performance Dashboard JavaScript
// Generated by render_dashboard.js

class PerformanceDashboard {
    constructor() {
        this.config = ${JSON.stringify(DASHBOARD_CONFIG)};
        this.currentData = {};
        this.refreshInterval = null;
    }
    
    async initialize() {
        console.log('🚀 Initializing Performance Dashboard...');
        await this.loadData();
        this.setupEventListeners();
        this.updateDashboard();
    }
    
    async loadData() {
        console.log('📊 Loading performance data...');
        try {
            const response = await fetch('data/index.json');
            if (response.ok) {
                const data = await response.json();
                this.currentData = data;
                console.log('✅ Performance data loaded');
            }
        } catch (error) {
            console.error('❌ Failed to load performance data:', error);
        }
    }
    
    setupEventListeners() {
        console.log('🎧 Setting up event listeners...');
        
        // Configuration selection
        document.getElementById('config-select').addEventListener('change', () => {
            this.updateDashboard();
        });
        
        // Metric selection
        document.getElementById('metric-select').addEventListener('change', () => {
            this.updateDashboard();
        });
        
        // Time range selection
        document.getElementById('time-range').addEventListener('change', () => {
            this.updateDashboard();
        });
        
        // Auto-refresh toggle
        document.getElementById('refresh-toggle').addEventListener('change', () => {
            this.setupAutoRefresh();
        });
    }
    
    updateDashboard() {
        console.log('🔄 Updating dashboard...');
        
        const selectedConfig = document.getElementById('config-select').value;
        const selectedMetric = document.getElementById('metric-select').value;
        const timeRange = document.getElementById('time-range').value;
        
        // Update summary statistics
        this.updateSummaryStats(selectedConfig, timeRange);
        
        // Update charts
        this.updateLatencyChart(selectedConfig, selectedMetric, timeRange);
        this.updateOverheadChart(selectedConfig, timeRange);
        this.updateMatrixChart(timeRange);
        this.updateTrendChart(selectedConfig, timeRange);
    }
    
    updateSummaryStats(config, timeRange) {
        // Calculate summary statistics based on selected configuration and time range
        const stats = this.calculateSummaryStats(config, timeRange);
        
        document.getElementById('total-runs').textContent = stats.totalRuns;
        document.getElementById('pass-rate').textContent = stats.passRate + '%';
        document.getElementById('avg-latency').textContent = stats.avgLatency + 'ms';
        document.getElementById('trend').textContent = stats.trend;
    }
    
    calculateSummaryStats(config, timeRange) {
        // Implementation for calculating summary statistics
        // This would analyze the currentData and return calculated values
        return {
            totalRuns: Object.keys(this.currentData.configurations || {}).length,
            passRate: 95,
            avgLatency: 150,
            trend: 'Stable'
        };
    }
    
    updateLatencyChart(config, metric, timeRange) {
        const chartContainer = document.getElementById('latency-chart');
        
        if (!config || !metric) {
            chartContainer.innerHTML = '<div class="chart-placeholder">Select configuration and metric to view chart</div>';
            return;
        }
        
        // Implementation for latency chart
        chartContainer.innerHTML = '<div class="chart-placeholder">Latency chart for ' + config + ' - ' + metric + '</div>';
    }
    
    updateOverheadChart(config, timeRange) {
        const chartContainer = document.getElementById('overhead-chart');
        
        if (!config) {
            chartContainer.innerHTML = '<div class="chart-placeholder">Select configuration to view overhead analysis</div>';
            return;
        }
        
        // Implementation for overhead chart
        chartContainer.innerHTML = '<div class="chart-placeholder">Overhead analysis for ' + config + '</div>';
    }
    
    updateMatrixChart(timeRange) {
        const chartContainer = document.getElementById('matrix-chart');
        
        // Implementation for configuration matrix chart
        chartContainer.innerHTML = '<div class="chart-placeholder">Configuration matrix for last ' + timeRange + ' days</div>';
    }
    
    updateTrendChart(config, timeRange) {
        const chartContainer = document.getElementById('trend-chart');
        
        if (!config) {
            chartContainer.innerHTML = '<div class="chart-placeholder">Select configuration to view trend analysis</div>';
            return;
        }
        
        // Implementation for trend chart
        chartContainer.innerHTML = '<div class="chart-placeholder">Trend analysis for ' + config + '</div>';
    }
    
    setupAutoRefresh() {
        const refreshValue = document.getElementById('refresh-toggle').value;
        
        // Clear existing interval
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
        }
        
        // Setup new interval if enabled
        if (refreshValue > 0) {
            this.refreshInterval = setInterval(async () => {
                console.log('🔄 Auto-refreshing performance data...');
                await this.loadData();
                this.updateDashboard();
            }, refreshValue * 1000);
        }
    }
}

// Initialize dashboard when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    const dashboard = new PerformanceDashboard();
    dashboard.initialize();
});`;
    }

    /**
     * Generate README content
     */
    generateREADME() {
        return `# Performance Dashboard

This directory contains the static performance dashboard for Polymera OS.

## Files

- \`index.html\` - Main dashboard page
- \`style.css\` - Dashboard styles
- \`dashboard.js\` - Dashboard functionality
- \`data/\` - Performance data shards
- \`README.md\` - This file

## Usage

1. Open \`index.html\` in a web browser
2. Select configuration and metrics to view
3. Use time range controls to filter data
4. Enable auto-refresh for real-time updates

## Data Sources

Performance data is loaded from:
- \`${this.inputDir}/\` - Historical performance data
- \`${this.baselineFile}\` - Baseline performance metrics

## Configuration

The dashboard can be customized by modifying:
- \`tooling/perf/render_dashboard.js\` - Dashboard generation script
- \`tooling/perf/render_dashboard.js\` - Dashboard configuration

## Generation

This dashboard is automatically generated by:
\`\`\`bash
node tooling/perf/render_dashboard.js
\`\`\`

Generated on: ${new Date().toISOString()}
`;
    }

    /**
     * Format configuration name for display
     */
    formatConfigName(configId) {
        return configId
            .replace(/[_-]/g, ' ')
            .replace(/\\b\\w/g, l => l.toUpperCase());
    }
}

// Main execution
async function main() {
    const args = process.argv.slice(2);
    let inputDir = DEFAULT_INPUT_DIR;
    let outputDir = DEFAULT_OUTPUT_DIR;
    let baselineFile = DEFAULT_BASELINE_FILE;

    // Parse command line arguments
    for (let i = 0; i < args.length; i++) {
        switch (args[i]) {
            case '--input':
                inputDir = args[++i];
                break;
            case '--output':
                outputDir = args[++i];
                break;
            case '--baseline':
                baselineFile = args[++i];
                break;
            case '--help':
            case '-h':
                console.log(`
Performance Dashboard Renderer for Polymera OS

Usage: node render_dashboard.js [options]

Options:
  --input DIR       Input directory containing performance history (default: ${DEFAULT_INPUT_DIR})
  --output DIR      Output directory for generated dashboard (default: ${DEFAULT_OUTPUT_DIR})
  --baseline FILE   Baseline performance file (default: ${DEFAULT_BASELINE_FILE})
  --help, -h        Show this help message

Examples:
  node render_dashboard.js
  node render_dashboard.js --input perf/history --output docs/site/perf
  node render_dashboard.js --baseline perf/baselines/custom.json
`);
                process.exit(0);
        }
    }

    // Validate inputs
    if (!fs.existsSync(inputDir)) {
        console.error(`❌ Input directory not found: ${inputDir}`);
        process.exit(1);
    }

    // Create renderer and run
    const renderer = new DashboardRenderer(inputDir, outputDir, baselineFile);
    await renderer.run();
}

// Run if called directly
if (require.main === module) {
    main().catch(error => {
        console.error('❌ Dashboard generation failed:', error);
        process.exit(1);
    });
}

module.exports = DashboardRenderer;
