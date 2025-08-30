import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { NgfsIntegrityChecker, NgfsManifestSchema, NgfsBenchmarkSchema, IntegrityManifestSchema } from '../integrity_check';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';

describe('NgfsIntegrityChecker', () => {
    let checker: NgfsIntegrityChecker;
    let tempDir: string;

    beforeEach(() => {
        checker = new NgfsIntegrityChecker();
        tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'ngfs-test-'));
    });

    afterEach(() => {
        // Clean up temp directory
        try {
            fs.rmSync(tempDir, { recursive: true, force: true });
        } catch (error) {
            // Ignore cleanup errors
        }
    });

    describe('Schema Validation', () => {
        it('should validate correct NGFS manifest schema', () => {
            const validManifest = {
                version: 1,
                created: "2024-01-01T00:00:00Z",
                updated: "2024-01-01T00:00:00Z",
                root_cid: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32],
                entries: [
                    {
                        name: "test.txt",
                        cid: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32],
                        kind: "file" as const,
                        size: 1024,
                        permissions: {
                            owner: "user",
                            group: "group",
                            mode: 0o644
                        }
                    }
                ],
                metadata: {
                    description: "Test manifest"
                }
            };

            const result = NgfsManifestSchema.safeParse(validManifest);
            expect(result.success).toBe(true);
        });

        it('should validate correct benchmark schema', () => {
            const validBenchmark = {
                test: "ngfs_bench",
                timestamp: 1704067200000,
                duration_ms: 15000,
                cases: {
                    "resolve_path": {
                        p50: 100,
                        p95: 200,
                        p99: 300,
                        mean: 150,
                        stdev: 50,
                        n: 100,
                        errors: 0
                    }
                },
                errors: 0,
                mismatch: 0
            };

            const result = NgfsBenchmarkSchema.safeParse(validBenchmark);
            expect(result.success).toBe(true);
        });

        it('should validate correct integrity manifest schema', () => {
            const validIntegrityManifest = {
                version: 1,
                description: "Test integrity manifest",
                created: "2024-01-01T00:00:00Z",
                last_updated: "2024-01-01T00:00:00Z",
                phase: "P3-01-A8",
                schemas: [
                    {
                        path: "services/ngfs/src/schema.rs",
                        digest: "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
                        last_phase: "P3-01-A1",
                        description: "NGFS core schema definitions",
                        critical: true
                    }
                ],
                cbor_fixtures: [],
                performance: [],
                ipfs_exports: [],
                test_vectors: [],
                generated_outputs: [],
                configs: [],
                settings: {
                    hash_algorithm: "blake3-256",
                    hash_format: "hex-lowercase",
                    max_file_size_mb: 10,
                    critical_failure_threshold: 0,
                    warning_threshold: 1
                },
                rebaseline: {
                    require_reason: true,
                    require_ticket: true,
                    require_phase_update: true,
                    audit_logging: true
                },
                ci: {
                    job_name: "ngfs-integrity",
                    timeout_minutes: 15,
                    artifact_retention_days: 30,
                    block_on_failure: true,
                    pre_test_gate: true
                }
            };

            const result = IntegrityManifestSchema.safeParse(validIntegrityManifest);
            expect(result.success).toBe(true);
        });

        it('should reject invalid NGFS manifest schema', () => {
            const invalidManifest = {
                version: "invalid", // Should be number
                created: "2024-01-01T00:00:00Z",
                updated: "2024-01-01T00:00:00Z",
                root_cid: [1, 2, 3], // Too short
                entries: []
            };

            const result = NgfsManifestSchema.safeParse(invalidManifest);
            expect(result.success).toBe(false);
            if (!result.success) {
                expect(result.error.errors.length).toBeGreaterThan(0);
            }
        });

        it('should reject invalid benchmark schema', () => {
            const invalidBenchmark = {
                test: "invalid_test", // Should be "ngfs_bench"
                timestamp: 1704067200000,
                duration_ms: 15000,
                cases: {},
                errors: 0,
                mismatch: 0
            };

            const result = NgfsBenchmarkSchema.safeParse(invalidBenchmark);
            expect(result.success).toBe(false);
            if (!result.success) {
                expect(result.error.errors.length).toBeGreaterThan(0);
            }
        });
    });

    describe('File Validation', () => {
        it('should validate existing JSON file', () => {
            const testFile = path.join(tempDir, 'test.json');
            const testData = { key: 'value', number: 42 };
            fs.writeFileSync(testFile, JSON.stringify(testData));

            const result = checker.validateFile(testFile, NgfsBenchmarkSchema, 'Test file');
            expect(result).toBe(false); // Should fail because it doesn't match benchmark schema
        });

        it('should handle non-existent file', () => {
            const nonExistentFile = path.join(tempDir, 'nonexistent.json');
            
            const result = checker.validateFile(nonExistentFile, NgfsBenchmarkSchema, 'Non-existent file');
            expect(result).toBe(false);
            expect(checker.getErrors().length).toBeGreaterThan(0);
        });

        it('should handle invalid JSON file', () => {
            const invalidJsonFile = path.join(tempDir, 'invalid.json');
            fs.writeFileSync(invalidJsonFile, 'invalid json content');

            const result = checker.validateFile(invalidJsonFile, NgfsBenchmarkSchema, 'Invalid JSON file');
            expect(result).toBe(false);
            expect(checker.getErrors().length).toBeGreaterThan(0);
        });
    });

    describe('CBOR File Validation', () => {
        it('should validate existing CBOR file', () => {
            const testFile = path.join(tempDir, 'test.cbor');
            const testData = Buffer.from([0x82, 0x01, 0x02]); // CBOR: [1, 2]
            fs.writeFileSync(testFile, testData);

            const result = checker.validateCBORFile(testFile);
            expect(result).toBe(true);
        });

        it('should reject empty CBOR file', () => {
            const emptyFile = path.join(tempDir, 'empty.cbor');
            fs.writeFileSync(emptyFile, '');

            const result = checker.validateCBORFile(emptyFile);
            expect(result).toBe(false);
            expect(checker.getErrors().length).toBeGreaterThan(0);
        });

        it('should handle non-existent CBOR file', () => {
            const nonExistentFile = path.join(tempDir, 'nonexistent.cbor');
            
            const result = checker.validateCBORFile(nonExistentFile);
            expect(result).toBe(false);
            expect(checker.getErrors().length).toBeGreaterThan(0);
        });

        it('should warn about potentially invalid CBOR headers', () => {
            const suspiciousFile = path.join(tempDir, 'suspicious.cbor');
            fs.writeFileSync(suspiciousFile, Buffer.from([0x00, 0x01, 0x02])); // Starts with 0x00

            const result = checker.validateCBORFile(suspiciousFile);
            expect(result).toBe(true); // Should still pass
            expect(checker.getWarnings().length).toBeGreaterThan(0);
        });
    });

    describe('Cross-Check CBOR Vectors', () => {
        it('should validate CBOR vectors in fixtures directory', () => {
            // Create mock fixtures directory structure
            const fixturesDir = path.join(tempDir, 'fixtures');
            fs.mkdirSync(fixturesDir, { recursive: true });
            
            const ipfsDir = path.join(fixturesDir, 'ipfs');
            fs.mkdirSync(ipfsDir, { recursive: true });

            // Create mock CBOR files
            fs.writeFileSync(path.join(fixturesDir, 'dir_simple.cbor'), Buffer.from([0x82, 0x01, 0x02]));
            fs.writeFileSync(path.join(fixturesDir, 'file_1chunk.cbor'), Buffer.from([0x82, 0x03, 0x04]));
            fs.writeFileSync(path.join(ipfsDir, 'dir_simple.cbor'), Buffer.from([0x82, 0x05, 0x06]));
            fs.writeFileSync(path.join(ipfsDir, 'file_1chunk.cbor'), Buffer.from([0x82, 0x07, 0x08]));

            const result = checker.crossCheckCBORVectors(fixturesDir);
            expect(result).toBe(true);
            expect(checker.getErrors().length).toBe(0);
        });

        it('should handle missing fixtures directory', () => {
            const nonExistentDir = path.join(tempDir, 'nonexistent_fixtures');
            
            const result = checker.crossCheckCBORVectors(nonExistentDir);
            expect(result).toBe(false);
            expect(checker.getErrors().length).toBeGreaterThan(0);
        });
    });

    describe('Performance Files Validation', () => {
        it('should validate performance files in perf directory', () => {
            // Create mock perf directory structure
            const perfDir = path.join(tempDir, 'perf');
            const baselinesDir = path.join(perfDir, 'baselines');
            fs.mkdirSync(baselinesDir, { recursive: true });

            // Create mock baseline file
            const baselineData = {
                version: 1,
                description: "Test performance baseline",
                created: "2024-01-01T00:00:00Z",
                last_updated: "2024-01-01T00:00:00Z",
                phase: "P3-01-A7",
                schemas: [],
                cbor_fixtures: [],
                performance: [],
                ipfs_exports: [],
                test_vectors: [],
                generated_outputs: [],
                configs: [],
                settings: {
                    hash_algorithm: "blake3-256",
                    hash_format: "hex-lowercase",
                    max_file_size_mb: 10,
                    critical_failure_threshold: 0,
                    warning_threshold: 1
                },
                rebaseline: {
                    require_reason: true,
                    require_ticket: true,
                    require_phase_update: true,
                    audit_logging: true
                },
                ci: {
                    job_name: "ngfs-perf",
                    timeout_minutes: 30,
                    artifact_retention_days: 30,
                    block_on_failure: true,
                    pre_test_gate: true
                }
            };

            fs.writeFileSync(path.join(baselinesDir, 'p3_ngfs.json'), JSON.stringify(baselineData));

            const result = checker.validatePerformanceFiles(perfDir);
            expect(result).toBe(true);
            expect(checker.getErrors().length).toBe(0);
        });

        it('should handle missing perf directory', () => {
            const nonExistentDir = path.join(tempDir, 'nonexistent_perf');
            
            const result = checker.validatePerformanceFiles(nonExistentDir);
            expect(result).toBe(false);
            expect(checker.getErrors().length).toBeGreaterThan(0);
        });
    });

    describe('Integrity Manifest Validation', () => {
        it('should validate integrity manifest file', () => {
            const manifestFile = path.join(tempDir, 'manifest.yml');
            const manifestData = {
                version: 1,
                description: "Test integrity manifest",
                created: "2024-01-01T00:00:00Z",
                last_updated: "2024-01-01T00:00:00Z",
                phase: "P3-01-A8",
                schemas: [],
                cbor_fixtures: [],
                performance: [],
                ipfs_exports: [],
                test_vectors: [],
                generated_outputs: [],
                configs: [],
                settings: {
                    hash_algorithm: "blake3-256",
                    hash_format: "hex-lowercase",
                    max_file_size_mb: 10,
                    critical_failure_threshold: 0,
                    warning_threshold: 1
                },
                rebaseline: {
                    require_reason: true,
                    require_ticket: true,
                    require_phase_update: true,
                    audit_logging: true
                },
                ci: {
                    job_name: "ngfs-integrity",
                    timeout_minutes: 15,
                    artifact_retention_days: 30,
                    block_on_failure: true,
                    pre_test_gate: true
                }
            };

            fs.writeFileSync(manifestFile, JSON.stringify(manifestData));

            const result = checker.validateIntegrityManifest(manifestFile);
            expect(result).toBe(true);
            expect(checker.getErrors().length).toBe(0);
        });

        it('should handle missing integrity manifest file', () => {
            const nonExistentFile = path.join(tempDir, 'nonexistent_manifest.yml');
            
            const result = checker.validateIntegrityManifest(nonExistentFile);
            expect(result).toBe(false);
            expect(checker.getErrors().length).toBeGreaterThan(0);
        });
    });

    describe('Schema Drift Detection', () => {
        it('should check for schema drift', () => {
            const result = checker.checkSchemaDrift();
            expect(result).toBe(true);
            expect(checker.getErrors().length).toBe(0);
        });
    });

    describe('State Management', () => {
        it('should clear state correctly', () => {
            // First, create some errors and warnings
            const testFile = path.join(tempDir, 'test.json');
            fs.writeFileSync(testFile, 'invalid json');
            
            checker.validateFile(testFile, NgfsBenchmarkSchema, 'Test file');
            expect(checker.getErrors().length).toBeGreaterThan(0);

            // Clear state
            checker.clear();
            expect(checker.getErrors().length).toBe(0);
            expect(checker.getWarnings().length).toBe(0);
        });

        it('should return copies of errors and warnings', () => {
            // Create some errors
            const testFile = path.join(tempDir, 'test.json');
            fs.writeFileSync(testFile, 'invalid json');
            
            checker.validateFile(testFile, NgfsBenchmarkSchema, 'Test file');
            
            const errors = checker.getErrors();
            const warnings = checker.getWarnings();
            
            // Verify we got copies
            expect(errors).toEqual(checker.getErrors());
            expect(warnings).toEqual(checker.getWarnings());
            expect(errors).not.toBe(checker.getErrors());
            expect(warnings).not.toBe(checker.getWarnings());
        });
    });

    describe('End-to-End Integration', () => {
        it('should run complete integrity check successfully', () => {
            // Create complete test environment
            const fixturesDir = path.join(tempDir, 'fixtures');
            const perfDir = path.join(tempDir, 'perf');
            const manifestFile = path.join(tempDir, 'manifest.yml');
            
            fs.mkdirSync(fixturesDir, { recursive: true });
            fs.mkdirSync(path.join(perfDir, 'baselines'), { recursive: true });

            // Create test files
            fs.writeFileSync(path.join(fixturesDir, 'dir_simple.cbor'), Buffer.from([0x82, 0x01, 0x02]));
            fs.writeFileSync(path.join(fixturesDir, 'file_1chunk.cbor'), Buffer.from([0x82, 0x03, 0x04]));
            
            const baselineData = {
                version: 1,
                description: "Test baseline",
                created: "2024-01-01T00:00:00Z",
                last_updated: "2024-01-01T00:00:00Z",
                phase: "P3-01-A7",
                schemas: [],
                cbor_fixtures: [],
                performance: [],
                ipfs_exports: [],
                test_vectors: [],
                generated_outputs: [],
                configs: [],
                settings: {
                    hash_algorithm: "blake3-256",
                    hash_format: "hex-lowercase",
                    max_file_size_mb: 10,
                    critical_failure_threshold: 0,
                    warning_threshold: 1
                },
                rebaseline: {
                    require_reason: true,
                    require_ticket: true,
                    require_phase_update: true,
                    audit_logging: true
                },
                ci: {
                    job_name: "ngfs-perf",
                    timeout_minutes: 30,
                    artifact_retention_days: 30,
                    block_on_failure: true,
                    pre_test_gate: true
                }
            };

            fs.writeFileSync(path.join(perfDir, 'baselines', 'p3_ngfs.json'), JSON.stringify(baselineData));
            
            const manifestData = {
                version: 1,
                description: "Test manifest",
                created: "2024-01-01T00:00:00Z",
                last_updated: "2024-01-01T00:00:00Z",
                phase: "P3-01-A8",
                schemas: [],
                cbor_fixtures: [],
                performance: [],
                ipfs_exports: [],
                test_vectors: [],
                generated_outputs: [],
                configs: [],
                settings: {
                    hash_algorithm: "blake3-256",
                    hash_format: "hex-lowercase",
                    max_file_size_mb: 10,
                    critical_failure_threshold: 0,
                    warning_threshold: 1
                },
                rebaseline: {
                    require_reason: true,
                    require_ticket: true,
                    require_phase_update: true,
                    audit_logging: true
                },
                ci: {
                    job_name: "ngfs-integrity",
                    timeout_minutes: 15,
                    artifact_retention_days: 30,
                    block_on_failure: true,
                    pre_test_gate: true
                }
            };

            fs.writeFileSync(manifestFile, JSON.stringify(manifestData));

            const result = checker.runIntegrityCheck(manifestFile, fixturesDir, perfDir);
            expect(result).toBe(true);
            expect(checker.getErrors().length).toBe(0);
        });
    });
});
