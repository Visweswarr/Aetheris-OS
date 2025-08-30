#!/usr/bin/env/node

/**
 * NGFS Performance Renderer
 * 
 * This tool validates benchmark JSON schema and renders performance results
 * in a compact table format for PR comments and CI artifacts.
 */

import { readFileSync } from 'fs';
import { join } from 'path';

// Performance data interfaces
interface BenchCase {
    p50: number;
    p95: number;
    p99: number;
    mean: number;
    stdev: number;
    n: number;
    errors: number;
}

interface BenchOutput {
    test: string;
    timestamp: number;
    duration_ms: number;
    cases: Record<string, BenchCase>;
    errors: number;
    mismatch: number;
}

interface ValidationResult {
    valid: boolean;
    errors: string[];
    warnings: string[];
}

class NgfsPerfRenderer {
    private errors: string[] = [];
    private warnings: string[] = [];

    /**
     * Validate benchmark JSON schema
     */
    validateSchema(data: any): ValidationResult {
        this.errors = [];
        this.warnings = [];

        // Check top-level structure
        if (typeof data !== 'object' || data === null) {
            this.errors.push('Data must be an object');
            return this.getValidationResult();
        }

        // Check required fields
        const requiredFields = ['test', 'timestamp', 'duration_ms', 'cases'];
        for (const field of requiredFields) {
            if (!(field in data)) {
                this.errors.push(`Missing required field: ${field}`);
            }
        }

        // Check test field
        if (data.test !== 'ngfs_bench') {
            this.errors.push(`Invalid test name: ${data.test}, expected 'ngfs_bench'`);
        }

        // Check timestamp
        if (typeof data.timestamp !== 'number' || data.timestamp <= 0) {
            this.errors.push('Invalid timestamp');
        }

        // Check duration
        if (typeof data.duration_ms !== 'number' || data.duration_ms < 0) {
            this.errors.push('Invalid duration_ms');
        }

        // Check cases
        if (typeof data.cases !== 'object' || data.cases === null) {
            this.errors.push('Cases must be an object');
        } else {
            this.validateCases(data.cases);
        }

        // Check errors and mismatch
        if (typeof data.errors !== 'number' || data.errors < 0) {
            this.errors.push('Invalid errors count');
        }
        if (typeof data.mismatch !== 'number' || data.mismatch < 0) {
            this.errors.push('Invalid mismatch count');
        }

        return this.getValidationResult();
    }

    /**
     * Validate individual benchmark cases
     */
    private validateCases(cases: Record<string, any>): void {
        const requiredCases = ['resolve_path', 'stat_file', 'read_small', 'read_large', 'snapshot_build'];
        const requiredFields = ['p50', 'p95', 'p99', 'mean', 'stdev', 'n', 'errors'];

        for (const caseName of requiredCases) {
            if (!(caseName in cases)) {
                this.errors.push(`Missing required case: ${caseName}`);
                continue;
            }

            const caseData = cases[caseName];
            if (typeof caseData !== 'object' || caseData === null) {
                this.errors.push(`Invalid case data for ${caseName}`);
                continue;
            }

            // Check case fields
            for (const field of requiredFields) {
                if (!(field in caseData)) {
                    this.errors.push(`Missing field '${field}' in ${caseName}`);
                    continue;
                }

                const value = caseData[field];
                if (field === 'errors' && (typeof value !== 'number' || value < 0)) {
                    this.errors.push(`Invalid errors field in ${caseName}: ${value}`);
                } else if (field !== 'errors' && (typeof value !== 'number' || value < 0)) {
                    this.errors.push(`Invalid ${field} field in ${caseName}: ${value}`);
                }
            }

            // Check sample count
            if (caseData.n < 50) {
                this.warnings.push(`Low sample count in ${caseName}: ${caseData.n} < 50`);
            }

            // Check variance ratios
            if (caseData.mean > 0) {
                const varianceRatio = caseData.stdev / caseData.mean;
                if (caseName.includes('read') && varianceRatio > 0.25) {
                    this.warnings.push(`High variance in ${caseName}: stdev/mean=${varianceRatio.toFixed(2)} > 0.25`);
                } else if (varianceRatio > 0.35) {
                    this.warnings.push(`High variance in ${caseName}: stdev/mean=${varianceRatio.toFixed(2)} > 0.35`);
                }
            }
        }
    }

    /**
     * Get validation result
     */
    private getValidationResult(): ValidationResult {
        return {
            valid: this.errors.length === 0,
            errors: [...this.errors],
            warnings: [...this.warnings]
        };
    }

    /**
     * Render performance table
     */
    renderTable(data: BenchOutput): string {
        const lines: string[] = [];
        
        // Header
        lines.push('## NGFS Performance Benchmarks');
        lines.push('');
        lines.push(`**Test:** ${data.test} | **Duration:** ${data.duration_ms}ms | **Errors:** ${data.errors} | **Mismatch:** ${data.mismatch}`);
        lines.push('');
        
        // Table header
        lines.push('| Case | P50 (μs) | P95 (μs) | P99 (μs) | Mean (μs) | Stdev | N | Errors |');
        lines.push('|------|-----------|-----------|-----------|-----------|-------|----|--------|');
        
        // Table rows
        const caseOrder = ['resolve_path', 'stat_file', 'read_small', 'read_large', 'snapshot_build'];
        
        for (const caseName of caseOrder) {
            const caseData = data.cases[caseName];
            if (!caseData) {
                lines.push(`| ${caseName} | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |`);
                continue;
            }

            const row = [
                caseName,
                caseData.p50.toString(),
                caseData.p95.toString(),
                caseData.p99.toString(),
                caseData.mean.toString(),
                caseData.stdev.toString(),
                caseData.n.toString(),
                caseData.errors.toString()
            ];
            
            lines.push(`| ${row.join(' | ')} |`);
        }
        
        lines.push('');
        
        // Performance summary
        lines.push('### Performance Summary');
        lines.push('');
        
        // Check against budgets
        const budgets = {
            resolve_path: 400,
            stat_file: 250,
            read_small: 600,
            read_large: 1500,
            snapshot_build: 50
        };
        
        let budgetViolations = 0;
        for (const [caseName, budget] of Object.entries(budgets)) {
            const caseData = data.cases[caseName];
            if (caseData && caseData.p95 > budget) {
                lines.push(`❌ **${caseName}**: P95 ${caseData.p95}μs > ${budget}μs budget`);
                budgetViolations++;
            }
        }
        
        if (budgetViolations === 0) {
            lines.push('✅ **All cases within budget**');
        }
        
        lines.push('');
        
        // Variance summary
        lines.push('### Variance Analysis');
        lines.push('');
        
        for (const caseName of caseOrder) {
            const caseData = data.cases[caseName];
            if (!caseData || caseData.mean === 0) continue;
            
            const varianceRatio = caseData.stdev / caseData.mean;
            const status = varianceRatio <= 0.25 ? '✅' : varianceRatio <= 0.35 ? '⚠️' : '❌';
            lines.push(`${status} **${caseName}**: stdev/mean = ${varianceRatio.toFixed(2)}`);
        }
        
        return lines.join('\n');
    }

    /**
     * Render compact summary for PR comments
     */
    renderCompactSummary(data: BenchOutput): string {
        const lines: string[] = [];
        
        lines.push('## 🚀 NGFS Performance Benchmarks');
        lines.push('');
        
        // Quick status
        const totalErrors = data.errors + data.mismatch;
        const status = totalErrors === 0 ? '✅ PASS' : '❌ FAIL';
        lines.push(`**Status:** ${status} | **Duration:** ${data.duration_ms}ms | **Issues:** ${totalErrors}`);
        lines.push('');
        
        // Key metrics
        const keyCases = ['resolve_path', 'read_small', 'read_large'];
        for (const caseName of keyCases) {
            const caseData = data.cases[caseName];
            if (caseData) {
                lines.push(`- **${caseName}**: P95=${caseData.p95}μs, N=${caseData.n}, Errors=${caseData.errors}`);
            }
        }
        
        return lines.join('\n');
    }
}

/**
 * Main function
 */
function main(): void {
    const args = process.argv.slice(2);
    
    if (args.length === 0) {
        console.error('Usage: node render_perf.js <benchmark-file> [--compact]');
        process.exit(1);
    }
    
    const inputFile = args[0];
    const compactMode = args.includes('--compact');
    
    try {
        // Read and parse input file
        const fileContent = readFileSync(inputFile, 'utf8');
        const benchData: BenchOutput = JSON.parse(fileContent);
        
        // Create renderer and validate
        const renderer = new NgfsPerfRenderer();
        const validation = renderer.validateSchema(benchData);
        
        if (!validation.valid) {
            console.error('❌ Schema validation failed:');
            for (const error of validation.errors) {
                console.error(`  ERROR: ${error}`);
            }
            process.exit(1);
        }
        
        if (validation.warnings.length > 0) {
            console.warn('⚠️  Schema validation warnings:');
            for (const warning of validation.warnings) {
                console.warn(`  WARNING: ${warning}`);
            }
        }
        
        // Render output
        if (compactMode) {
            console.log(renderer.renderCompactSummary(benchData));
        } else {
            console.log(renderer.renderTable(benchData));
        }
        
    } catch (error) {
        console.error('❌ Failed to process benchmark file:', error);
        process.exit(1);
    }
}

// Run if called directly
if (require.main === module) {
    main();
}

export { NgfsPerfRenderer, NgfsPerfRenderer as default };
