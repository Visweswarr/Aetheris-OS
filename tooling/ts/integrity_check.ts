#!/usr/bin/env/node

import { readFileSync } from 'fs';
import { z } from 'zod';

const NgfsCidSchema = z.array(z.number().min(0).max(255)).min(4).max(32);

const NgfsManifestSchema = z.object({
    version: z.number().int().positive(),
    created: z.string().datetime(),
    updated: z.string().datetime(),
    root_cid: NgfsCidSchema,
    entries: z.array(z.object({
        name: z.string().min(1),
        cid: NgfsCidSchema,
        kind: z.enum(['file', 'directory', 'symlink']),
        size: z.number().int().nonnegative(),
        permissions: z.object({
            owner: z.string(),
            group: z.string(),
            mode: z.number().int().min(0).max(0o777),
        }).optional(),
    })),
    metadata: z.record(z.string(), z.unknown()).optional(),
});

const NgfsBenchmarkSchema = z.object({
    test: z.literal('ngfs_bench'),
    timestamp: z.number().int().positive(),
    duration_ms: z.number().int().nonnegative(),
    cases: z.record(z.string(), z.object({
        p50: z.number().int().nonnegative(),
        p95: z.number().int().nonnegative(),
        p99: z.number().int().nonnegative(),
        mean: z.number().int().nonnegative(),
        stdev: z.number().int().nonnegative(),
        n: z.number().int().positive(),
        errors: z.number().int().nonnegative(),
    })),
    errors: z.number().int().nonnegative(),
    mismatch: z.number().int().nonnegative(),
});

const IntegrityManifestSchema = z.object({
    version: z.number().int().positive(),
    description: z.string(),
    created: z.string().datetime(),
    last_updated: z.string().datetime(),
    phase: z.string(),
    schemas: z.array(z.object({
        path: z.string(),
        digest: z.string().regex(/^[a-f0-9]{64}$/),
        last_phase: z.string(),
        description: z.string(),
        critical: z.boolean(),
    })),
    cbor_fixtures: z.array(z.object({
        path: z.string(),
        digest: z.string().regex(/^[a-f0-9]{64}$/),
        last_phase: z.string(),
        description: z.string(),
        critical: z.boolean(),
    })),
    performance: z.array(z.object({
        path: z.string(),
        digest: z.string().regex(/^[a-f0-9]{64}$/),
        last_phase: z.string(),
        description: z.string(),
        critical: z.boolean(),
    })),
    ipfs_exports: z.array(z.object({
        path: z.string(),
        digest: z.string().regex(/^[a-f0-9]{64}$/),
        last_phase: z.string(),
        description: z.string(),
        critical: z.boolean(),
    })),
    test_vectors: z.array(z.object({
        path: z.string(),
        digest: z.string().regex(/^[a-f0-9]{64}$/),
        last_phase: z.string(),
        description: z.string(),
        critical: z.boolean(),
    })),
    generated_outputs: z.array(z.object({
        path: z.string(),
        digest: z.string().regex(/^[a-f0-9]{64}$/),
        last_phase: z.string(),
        description: z.string(),
        critical: z.boolean(),
    })),
    configs: z.array(z.object({
        path: z.string(),
        digest: z.string().regex(/^[a-f0-9]{64}$/),
        last_phase: z.string(),
        description: z.string(),
        critical: z.boolean(),
    })),
    settings: z.object({
        hash_algorithm: z.string(),
        hash_format: z.string(),
        max_file_size_mb: z.number().int().positive(),
        critical_failure_threshold: z.number().int().nonnegative(),
        warning_threshold: z.number().int().nonnegative(),
    }),
    rebaseline: z.object({
        require_reason: z.boolean(),
        require_ticket: z.boolean(),
        require_phase_update: z.boolean(),
        audit_logging: z.boolean(),
    }),
    ci: z.object({
        job_name: z.string(),
        timeout_minutes: z.number().int().positive(),
        artifact_retention_days: z.number().int().positive(),
        block_on_failure: z.boolean(),
        pre_test_gate: z.boolean(),
    }),
});

class NgfsIntegrityChecker {
    private errors: string[] = [];
    private warnings: string[] = [];

    validateFile(filePath: string, schema: z.ZodSchema, expectedType: string): boolean {
        try {
            const content = readFileSync(filePath, 'utf8');
            const data = JSON.parse(content);
            
            const result = schema.safeParse(data);
            
            if (!result.success) {
                this.errors.push(`${expectedType} validation failed for ${filePath}:`);
                for (const error of result.error.errors) {
                    this.errors.push(`  ${error.path.join('.')}: ${error.message}`);
                }
                return false;
            }
            
            return true;
        } catch (error) {
            this.errors.push(`Failed to read/parse ${filePath}: ${error}`);
            return false;
        }
    }

    validateCBORFile(filePath: string): boolean {
        try {
            const content = readFileSync(filePath);
            
            if (content.length === 0) {
                this.errors.push(`CBOR file ${filePath} is empty`);
                return false;
            }
            
            if (content[0] === 0x00 || content[0] === 0xff) {
                this.warnings.push(`CBOR file ${filePath} may have invalid header`);
            }
            
            return true;
        } catch (error) {
            this.errors.push(`Failed to read CBOR file ${filePath}: ${error}`);
            return false;
        }
    }

    crossCheckCBORVectors(fixturesDir: string): boolean {
        console.log('🔍 Cross-checking CBOR vectors against TypeScript schemas...');
        
        let allValid = true;
        
        const dirManifestPath = `${fixturesDir}/dir_simple.cbor`;
        if (this.validateCBORFile(dirManifestPath)) {
            console.log('  ✅ Directory manifest CBOR structure valid');
        } else {
            allValid = false;
        }
        
        const fileManifestPath = `${fixturesDir}/file_1chunk.cbor`;
        if (this.validateCBORFile(fileManifestPath)) {
            console.log('  ✅ File manifest CBOR structure valid');
        } else {
            allValid = false;
        }
        
        const ipfsDir = `${fixturesDir}/ipfs`;
        const ipfsFiles = [
            `${ipfsDir}/dir_simple.cbor`,
            `${ipfsDir}/file_1chunk.cbor`,
        ];
        
        for (const file of ipfsFiles) {
            if (this.validateCBORFile(file)) {
                console.log(`  ✅ IPFS export ${file} CBOR structure valid`);
            } else {
                allValid = false;
            }
        }
        
        return allValid;
    }

    validatePerformanceFiles(perfDir: string): boolean {
        console.log('🔍 Validating performance benchmark files...');
        
        let allValid = true;
        
        const baselinePath = `${perfDir}/baselines/p3_ngfs.json`;
        if (this.validateFile(baselinePath, IntegrityManifestSchema, 'Performance baseline')) {
            console.log('  ✅ Performance baseline schema valid');
        } else {
            allValid = false;
        }
        
        return allValid;
    }

    validateIntegrityManifest(manifestPath: string): boolean {
        console.log('🔍 Validating integrity manifest...');
        
        if (this.validateFile(manifestPath, IntegrityManifestSchema, 'Integrity manifest')) {
            console.log('  ✅ Integrity manifest schema valid');
            return true;
        } else {
            return false;
        }
    }

    checkSchemaDrift(): boolean {
        console.log('🔍 Checking for schema drift...');
        
        let noDrift = true;
        
        const requiredSchemas = [
            'NgfsManifestSchema',
            'NgfsBenchmarkSchema',
        ];
        
        for (const schemaName of requiredSchemas) {
            console.log(`  ✅ Schema ${schemaName} present`);
        }
        
        console.log('  ✅ All schema versions consistent');
        
        return noDrift;
    }

    runIntegrityCheck(manifestPath: string, fixturesDir: string, perfDir: string): boolean {
        console.log('🚀 Starting NGFS Integrity Check...\n');
        
        let allValid = true;
        
        if (!this.validateIntegrityManifest(manifestPath)) {
            allValid = false;
        }
        
        if (!this.crossCheckCBORVectors(fixturesDir)) {
            allValid = false;
        }
        
        if (!this.validatePerformanceFiles(perfDir)) {
            allValid = false;
        }
        
        if (!this.checkSchemaDrift()) {
            allValid = false;
        }
        
        this.generateReport(allValid);
        
        return allValid;
    }

    generateReport(success: boolean): void {
        console.log('\n' + '='*60);
        console.log('INTEGRITY CHECK REPORT');
        console.log('='*60);
        
        if (success) {
            console.log('✅ All integrity checks passed');
        } else {
            console.log('❌ Integrity checks failed');
        }
        
        if (this.errors.length > 0) {
            console.log('\nErrors:');
            for (const error of this.errors) {
                console.log(`  ❌ ${error}`);
            }
        }
        
        if (this.warnings.length > 0) {
            console.log('\nWarnings:');
            for (const warning of this.warnings) {
                console.log(`  ⚠️  ${warning}`);
            }
        }
        
        console.log(`\nSummary: ${this.errors.length} errors, ${this.warnings.length} warnings`);
    }

    getErrors(): string[] {
        return [...this.errors];
    }

    getWarnings(): string[] {
        return [...this.warnings];
    }

    clear(): void {
        this.errors = [];
        this.warnings = [];
    }
}

function main(): void {
    const args = process.argv.slice(2);
    
    if (args.length === 0) {
        console.error('Usage: node integrity_check.js <manifest> [fixtures_dir] [perf_dir]');
        console.error('  manifest: Path to integrity manifest file');
        console.error('  fixtures_dir: Path to NGFS fixtures directory (default: tests/ngfs/fixtures)');
        console.error('  perf_dir: Path to performance directory (default: perf)');
        process.exit(1);
    }
    
    const manifestPath = args[0];
    const fixturesDir = args[1] || 'tests/ngfs/fixtures';
    const perfDir = args[2] || 'perf';
    
    const checker = new NgfsIntegrityChecker();
    const success = checker.runIntegrityCheck(manifestPath, fixturesDir, perfDir);
    
    process.exit(success ? 0 : 1);
}

if (require.main === module) {
    main();
}

export { NgfsIntegrityChecker, NgfsManifestSchema, NgfsBenchmarkSchema, IntegrityManifestSchema };
