#!/usr/bin/env node

/**
 * PR Annotations for Regressions & Failing Specs
 * 
 * This script parses performance and conformance JSON files from the Phase 2 matrix
 * and emits GitHub Actions annotations for immediate PR feedback.
 * 
 * Usage: node tooling/ci/annotate.js [perf_file] [tests_file] [baseline_file]
 */

const fs = require('fs');
const path = require('path');

// Configuration
const MAX_ANNOTATIONS = 50;
const ANNOTATION_COALESCING = true;

// Colors for console output
const colors = {
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    magenta: '\x1b[35m',
    cyan: '\x1b[36m',
    reset: '\x1b[0m',
    bold: '\x1b[1m'
};

// GitHub Actions annotation functions
function emitAnnotation(level, message, file, line, endLine, title) {
    const annotation = `::${level} file=${file || ''},line=${line || ''},endLine=${endLine || ''},title=${title || ''}::${message}`;
    console.log(annotation);
}

function emitError(message, file, line, endLine, title) {
    emitAnnotation('error', message, file, line, endLine, title);
}

function emitWarning(message, file, line, endLine, title) {
    emitAnnotation('warning', message, file, line, endLine, title);
}

function emitNotice(message, file, line, endLine, title) {
    emitAnnotation('notice', message, file, line, endLine, title);
}

// Performance analysis functions
function analyzePerformanceData(perfData, baselineData, configId) {
    const annotations = [];
    const summary = {
        regressions: 0,
        improvements: 0,
        warnings: 0,
        total_metrics: 0
    };

    if (!perfData || !baselineData) {
        return { annotations, summary };
    }

    // Extract baseline for this configuration
    const baseline = baselineData.matrix_configurations?.[configId]?.performance_metrics;
    if (!baseline) {
        emitWarning(`No baseline data found for configuration: ${configId}`, null, null, null, 'Missing Baseline');
        return { annotations, summary };
    }

    // Analyze IPC latency metrics
    if (perfData.ipc_latency && baseline.ipc_latency) {
        const current = perfData.ipc_latency;
        const baseline_metrics = baseline.ipc_latency;
        
        summary.total_metrics += 4; // p50, p95, p99, mean
        
        // Check p50 latency
        if (current.p50_ns > baseline_metrics.p50_ns * 1.15) {
            const delta = ((current.p50_ns - baseline_metrics.p50_ns) / baseline_metrics.p50_ns * 100).toFixed(1);
            annotations.push({
                level: 'error',
                message: `IPC latency p50 regression: ${current.p50_ns}ns (+${delta}% vs baseline ${baseline_metrics.p50_ns}ns)`,
                file: 'perf/ipc_latency.json',
                line: 1,
                title: 'Performance Regression'
            });
            summary.regressions++;
        } else if (current.p50_ns < baseline_metrics.p50_ns * 0.85) {
            const delta = ((baseline_metrics.p50_ns - current.p50_ns) / baseline_metrics.p50_ns * 100).toFixed(1);
            annotations.push({
                level: 'notice',
                message: `IPC latency p50 improvement: ${current.p50_ns}ns (-${delta}% vs baseline ${baseline_metrics.p50_ns}ns)`,
                file: 'perf/ipc_latency.json',
                line: 1,
                title: 'Performance Improvement'
            });
            summary.improvements++;
        }

        // Check p95 latency
        if (current.p95_ns > baseline_metrics.p95_ns * 1.20) {
            const delta = ((current.p95_ns - baseline_metrics.p95_ns) / baseline_metrics.p95_ns * 100).toFixed(1);
            annotations.push({
                level: 'warning',
                message: `IPC latency p95 regression: ${current.p95_ns}ns (+${delta}% vs baseline ${baseline_metrics.p95_ns}ns)`,
                file: 'perf/ipc_latency.json',
                line: 1,
                title: 'Performance Warning'
            });
            summary.warnings++;
        }

        // Check p99 latency
        if (current.p99_ns > baseline_metrics.p99_ns * 1.25) {
            const delta = ((current.p99_ns - baseline_metrics.p99_ns) / baseline_metrics.p99_ns * 100).toFixed(1);
            annotations.push({
                level: 'error',
                message: `IPC latency p99 regression: ${current.p99_ns}ns (+${delta}% vs baseline ${baseline_metrics.p99_ns}ns)`,
                file: 'perf/ipc_latency.json',
                line: 1,
                title: 'Performance Regression'
            });
            summary.regressions++;
        }
    }

    // Analyze wake-to-run metrics
    if (perfData.wake_to_run && baseline.wake_to_run) {
        const current = perfData.wake_to_run;
        const baseline_metrics = baseline.wake_to_run;
        
        summary.total_metrics += 4; // p50, p95, p99, mean
        
        // Check p50 wake-to-run
        if (current.p50_ns > baseline_metrics.p50_ns * 1.12) {
            const delta = ((current.p50_ns - baseline_metrics.p50_ns) / baseline_metrics.p50_ns * 100).toFixed(1);
            annotations.push({
                level: 'error',
                message: `Wake-to-run latency p50 regression: ${current.p50_ns}ns (+${delta}% vs baseline ${baseline_metrics.p50_ns}ns)`,
                file: 'perf/wake_to_run.json',
                line: 1,
                title: 'Performance Regression'
            });
            summary.regressions++;
        }
    }

    // Analyze PQC overhead metrics
    if (perfData.pqc_overhead && baseline.pqc_overhead) {
        const current = perfData.pqc_overhead;
        const baseline_metrics = baseline.pqc_overhead;
        
        summary.total_metrics += 5; // kyber, dilithium_sign, dilithium_verify, mac_gen, mac_verify
        
        // Check Kyber encapsulation
        if (current.kyber_encapsulation_ns > baseline_metrics.kyber_encapsulation_ns * 1.08) {
            const delta = ((current.kyber_encapsulation_ns - baseline_metrics.kyber_encapsulation_ns) / baseline_metrics.kyber_encapsulation_ns * 100).toFixed(1);
            annotations.push({
                level: 'warning',
                message: `Kyber encapsulation overhead: ${current.kyber_encapsulation_ns}ns (+${delta}% vs baseline ${baseline_metrics.kyber_encapsulation_ns}ns)`,
                file: 'perf/pqc_overhead.json',
                line: 1,
                title: 'PQC Performance Warning'
            });
            summary.warnings++;
        }

        // Check Dilithium signature
        if (current.dilithium_sign_ns > baseline_metrics.dilithium_sign_ns * 1.10) {
            const delta = ((current.dilithium_sign_ns - baseline_metrics.dilithium_sign_ns) / baseline_metrics.dilithium_sign_ns * 100).toFixed(1);
            annotations.push({
                level: 'error',
                message: `Dilithium signature overhead: ${current.dilithium_sign_ns}ns (+${delta}% vs baseline ${baseline_metrics.dilithium_sign_ns}ns)`,
                file: 'perf/pqc_overhead.json',
                line: 1,
                title: 'PQC Performance Regression'
            });
            summary.regressions++;
        }
    }

    // Analyze memory usage
    if (perfData.memory_usage && baseline.memory_usage) {
        const current = perfData.memory_usage;
        const baseline_metrics = baseline.memory_usage;
        
        summary.total_metrics += 3; // kernel_heap, user_heap, stack_usage
        
        // Check kernel heap usage
        if (current.kernel_heap_mb > baseline_metrics.kernel_heap_mb * 1.10) {
            const delta = ((current.kernel_heap_mb - baseline_metrics.kernel_heap_mb) / baseline_metrics.kernel_heap_mb * 100).toFixed(1);
            annotations.push({
                level: 'warning',
                message: `Kernel heap usage increased: ${current.kernel_heap_mb}MB (+${delta}% vs baseline ${baseline_metrics.kernel_heap_mb}MB)`,
                file: 'perf/memory_usage.json',
                line: 1,
                title: 'Memory Usage Warning'
            });
            summary.warnings++;
        }
    }

    return { annotations, summary };
}

// Test conformance analysis functions
function analyzeTestConformance(testData, configId) {
    const annotations = [];
    const summary = {
        failures: 0,
        warnings: 0,
        total_tests: 0
    };

    if (!testData) {
        return { annotations, summary };
    }

    // Analyze test results
    if (testData.test_results) {
        const results = testData.test_results;
        summary.total_tests = Object.keys(results).length;

        for (const [test_suite, status] of Object.entries(results)) {
            if (status === 'FAIL') {
                annotations.push({
                    level: 'error',
                    message: `Test suite '${test_suite}' failed in configuration ${configId}`,
                    file: `tests/${test_suite}_tests.log`,
                    line: 1,
                    title: 'Test Failure'
                });
                summary.failures++;
            } else if (status === 'WARN') {
                annotations.push({
                    level: 'warning',
                    message: `Test suite '${test_suite}' has warnings in configuration ${configId}`,
                    file: `tests/${test_suite}_tests.log`,
                    line: 1,
                    title: 'Test Warning'
                });
                summary.warnings++;
            }
        }
    }

    // Analyze specific test failures
    if (testData.failed_tests && Array.isArray(testData.failed_tests)) {
        for (const test of testData.failed_tests) {
            annotations.push({
                level: 'error',
                message: `Test '${test.name}' failed: ${test.error || 'Unknown error'}`,
                file: test.log_file || 'tests/test_results.log',
                line: 1,
                title: 'Individual Test Failure'
            });
            summary.failures++;
        }
    }

    return { annotations, summary };
}

// Coalesce annotations to stay under limit
function coalesceAnnotations(annotations) {
    if (annotations.length <= MAX_ANNOTATIONS) {
        return annotations;
    }

    if (!ANNOTATION_COALESCING) {
        return annotations.slice(0, MAX_ANNOTATIONS);
    }

    // Group annotations by level and file
    const grouped = {};
    for (const annotation of annotations) {
        const key = `${annotation.level}:${annotation.file}`;
        if (!grouped[key]) {
            grouped[key] = [];
        }
        grouped[key].push(annotation);
    }

    // Coalesce groups and prioritize errors
    const coalesced = [];
    
    // Add all errors first
    for (const [key, group] of Object.entries(grouped)) {
        if (key.startsWith('error:')) {
            if (group.length === 1) {
                coalesced.push(group[0]);
            } else {
                // Coalesce multiple errors for same file
                const first = group[0];
                coalesced.push({
                    ...first,
                    message: `${group.length} errors in ${path.basename(first.file)}: ${group.map(a => a.message.split(':')[0]).join(', ')}...`,
                    title: `Multiple Errors (${group.length})`
                });
            }
        }
    }

    // Add warnings if space allows
    for (const [key, group] of Object.entries(grouped)) {
        if (key.startsWith('warning:') && coalesced.length < MAX_ANNOTATIONS - 10) {
            if (group.length === 1) {
                coalesced.push(group[0]);
            } else {
                const first = group[0];
                coalesced.push({
                    ...first,
                    message: `${group.length} warnings in ${path.basename(first.file)}: ${group.map(a => a.message.split(':')[0]).join(', ')}...`,
                    title: `Multiple Warnings (${group.length})`
                });
            }
        }
    }

    // Add notices if space allows
    for (const [key, group] of Object.entries(grouped)) {
        if (key.startsWith('notice:') && coalesced.length < MAX_ANNOTATIONS) {
            if (group.length === 1) {
                coalesced.push(group[0]);
            } else {
                const first = group[0];
                coalesced.push({
                    ...first,
                    message: `${group.length} notices in ${path.basename(first.file)}: ${group.map(a => a.message.split(':')[0]).join(', ')}...`,
                    title: `Multiple Notices (${group.length})`
                });
            }
        }
    }

    return coalesced;
}

// Main annotation function
function annotatePR(perfFile, testsFile, baselineFile, configId) {
    console.log(`${colors.bold}${colors.blue}🔍 PR Annotations for Regressions & Failing Specs${colors.reset}\n`);

    let perfData = null;
    let testsData = null;
    let baselineData = null;

    // Load performance data
    if (perfFile && fs.existsSync(perfFile)) {
        try {
            perfData = JSON.parse(fs.readFileSync(perfFile, 'utf8'));
            console.log(`${colors.green}✅ Loaded performance data from: ${perfFile}${colors.reset}`);
        } catch (error) {
            console.log(`${colors.yellow}⚠️  Failed to parse performance data: ${error.message}${colors.reset}`);
        }
    }

    // Load test data
    if (testsFile && fs.existsSync(testsFile)) {
        try {
            testsData = JSON.parse(fs.readFileSync(testsFile, 'utf8'));
            console.log(`${colors.green}✅ Loaded test data from: ${testsFile}${colors.reset}`);
        } catch (error) {
            console.log(`${colors.yellow}⚠️  Failed to parse test data: ${error.message}${colors.reset}`);
        }
    }

    // Load baseline data
    if (baselineFile && fs.existsSync(baselineFile)) {
        try {
            baselineData = JSON.parse(fs.readFileSync(baselineFile, 'utf8'));
            console.log(`${colors.green}✅ Loaded baseline data from: ${baselineFile}${colors.reset}`);
        } catch (error) {
            console.log(`${colors.yellow}⚠️  Failed to parse baseline data: ${error.message}${colors.reset}`);
        }
    }

    // Analyze data
    const perfAnalysis = analyzePerformanceData(perfData, baselineData, configId);
    const testAnalysis = analyzeTestConformance(testsData, configId);

    // Combine all annotations
    let allAnnotations = [...perfAnalysis.annotations, ...testAnalysis.annotations];
    
    // Coalesce if needed
    allAnnotations = coalesceAnnotations(allAnnotations);

    // Emit annotations
    console.log(`\n${colors.bold}📊 Emitting ${allAnnotations.length} annotations:${colors.reset}\n`);

    for (const annotation of allAnnotations) {
        emitAnnotation(
            annotation.level,
            annotation.message,
            annotation.file,
            annotation.line,
            annotation.endLine,
            annotation.title
        );
    }

    // Print summary
    console.log(`\n${colors.bold}📈 Analysis Summary:${colors.reset}`);
    console.log(`${colors.blue}Performance:${colors.reset}`);
    console.log(`  - Regressions: ${perfAnalysis.summary.regressions}`);
    console.log(`  - Improvements: ${perfAnalysis.summary.improvements}`);
    console.log(`  - Warnings: ${perfAnalysis.summary.warnings}`);
    console.log(`  - Total Metrics: ${perfAnalysis.summary.total_metrics}`);
    
    console.log(`${colors.blue}Tests:${colors.reset}`);
    console.log(`  - Failures: ${testAnalysis.summary.failures}`);
    console.log(`  - Warnings: ${testAnalysis.summary.warnings}`);
    console.log(`  - Total Tests: ${testAnalysis.summary.total_tests}`);
    
    console.log(`${colors.blue}Annotations:${colors.reset}`);
    console.log(`  - Emitted: ${allAnnotations.length}`);
    console.log(`  - Limit: ${MAX_ANNOTATIONS}`);
    console.log(`  - Coalescing: ${ANNOTATION_COALESCING ? 'Enabled' : 'Disabled'}`);

    // Set GitHub Actions outputs
    if (process.env.GITHUB_OUTPUT) {
        const output = [
            `perf_regressions=${perfAnalysis.summary.regressions}`,
            `perf_improvements=${perfAnalysis.summary.improvements}`,
            `perf_warnings=${perfAnalysis.summary.warnings}`,
            `test_failures=${testAnalysis.summary.failures}`,
            `test_warnings=${testAnalysis.summary.warnings}`,
            `total_annotations=${allAnnotations.length}`,
            `has_regressions=${perfAnalysis.summary.regressions > 0 || testAnalysis.summary.failures > 0}`
        ].join('\n');
        
        fs.appendFileSync(process.env.GITHUB_OUTPUT, output);
    }

    return {
        perf: perfAnalysis.summary,
        tests: testAnalysis.summary,
        annotations: allAnnotations.length,
        hasRegressions: perfAnalysis.summary.regressions > 0 || testAnalysis.summary.failures > 0
    };
}

// CLI interface
if (require.main === module) {
    const args = process.argv.slice(2);
    
    if (args.length === 0) {
        console.log(`${colors.bold}Usage:${colors.reset}`);
        console.log(`  node tooling/ci/annotate.js [perf_file] [tests_file] [baseline_file] [config_id]`);
        console.log(`\n${colors.bold}Examples:${colors.reset}`);
        console.log(`  node tooling/ci/annotate.js perf/out.json tests/out.json perf/baselines/p2.json apic_hpet_jitter_auth_open`);
        console.log(`  node tooling/ci/annotate.js perf/ipc_latency.json tests/core_tests.json`);
        console.log(`\n${colors.bold}Environment Variables:${colors.reset}`);
        console.log(`  GITHUB_OUTPUT: Set to enable GitHub Actions outputs`);
        console.log(`  MAX_ANNOTATIONS: Override default limit (default: ${MAX_ANNOTATIONS})`);
        console.log(`  ANNOTATION_COALESCING: Enable/disable coalescing (default: true)`);
        process.exit(0);
    }

    const perfFile = args[0] || null;
    const testsFile = args[1] || null;
    const baselineFile = args[2] || null;
    const configId = args[3] || 'unknown';

    // Override defaults from environment
    if (process.env.MAX_ANNOTATIONS) {
        MAX_ANNOTATIONS = parseInt(process.env.MAX_ANNOTATIONS) || 50;
    }
    if (process.env.ANNOTATION_COALESCING !== undefined) {
        ANNOTATION_COALESCING = process.env.ANNOTATION_COALESCING === 'true';
    }

    try {
        const result = annotatePR(perfFile, testsFile, baselineFile, configId);
        
        if (result.hasRegressions) {
            console.log(`\n${colors.red}❌ Regressions detected!${colors.reset}`);
            process.exit(1);
        } else {
            console.log(`\n${colors.green}✅ No regressions detected${colors.reset}`);
            process.exit(0);
        }
    } catch (error) {
        console.error(`${colors.red}❌ Error during annotation: ${error.message}${colors.reset}`);
        process.exit(1);
    }
}

module.exports = {
    annotatePR,
    analyzePerformanceData,
    analyzeTestConformance,
    coalesceAnnotations
};
