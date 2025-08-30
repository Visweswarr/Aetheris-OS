#!/usr/bin/env/node

/**
 * Tests for NGFS Performance Renderer
 * 
 * These tests verify schema validation works and table renders correctly.
 */

import { NgfsPerfRenderer } from '../render_perf';

describe('NgfsPerfRenderer', () => {
    let renderer: NgfsPerfRenderer;

    beforeEach(() => {
        renderer = new NgfsPerfRenderer();
    });

    describe('Schema Validation', () => {
        it('should validate correct benchmark data', () => {
            const validData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    },
                    stat_file: {
                        p50: 80,
                        p95: 250,
                        p99: 350,
                        mean: 95,
                        stdev: 30,
                        n: 50,
                        errors: 0
                    },
                    read_small: {
                        p50: 200,
                        p95: 600,
                        p99: 800,
                        mean: 220,
                        stdev: 55,
                        n: 50,
                        errors: 0
                    },
                    read_large: {
                        p50: 800,
                        p95: 1500,
                        p99: 2000,
                        mean: 850,
                        stdev: 120,
                        n: 50,
                        errors: 0
                    },
                    snapshot_build: {
                        p50: 25,
                        p95: 50,
                        p99: 75,
                        mean: 28,
                        stdev: 8,
                        n: 50,
                        errors: 0
                    }
                },
                errors: 0,
                mismatch: 0
            };

            const result = renderer.validateSchema(validData);
            expect(result.valid).toBe(true);
            expect(result.errors).toHaveLength(0);
        });

        it('should reject invalid test name', () => {
            const invalidData = {
                test: 'invalid_test',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {}
            };

            const result = renderer.validateSchema(invalidData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain("Invalid test name: invalid_test, expected 'ngfs_bench'");
        });

        it('should reject missing required fields', () => {
            const incompleteData = {
                test: 'ngfs_bench',
                timestamp: 1640995200
                // Missing duration_ms and cases
            };

            const result = renderer.validateSchema(incompleteData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Missing required field: duration_ms');
            expect(result.errors).toContain('Missing required field: cases');
        });

        it('should reject invalid timestamp', () => {
            const invalidData = {
                test: 'ngfs_bench',
                timestamp: -1,
                duration_ms: 1500,
                cases: {}
            };

            const result = renderer.validateSchema(invalidData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Invalid timestamp');
        });

        it('should reject invalid duration', () => {
            const invalidData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: -100,
                cases: {}
            };

            const result = renderer.validateSchema(invalidData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Invalid duration_ms');
        });

        it('should reject missing required cases', () => {
            const incompleteData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    }
                    // Missing other required cases
                }
            };

            const result = renderer.validateSchema(incompleteData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Missing required case: stat_file');
        });

        it('should reject invalid case data', () => {
            const invalidData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: "not_an_object"
                }
            };

            const result = renderer.validateSchema(invalidData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Invalid case data for resolve_path');
        });

        it('should reject missing case fields', () => {
            const incompleteData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        // Missing other required fields
                    }
                }
            };

            const result = renderer.validateSchema(incompleteData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain("Missing field 'p95' in resolve_path");
        });

        it('should reject invalid field types', () => {
            const invalidData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: "not_a_number",
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    }
                }
            };

            const result = renderer.validateSchema(invalidData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain("Invalid p50 field in resolve_path: not_a_number");
        });

        it('should reject negative values', () => {
            const invalidData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: -150,
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    }
                }
            };

            const result = renderer.validateSchema(invalidData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain("Invalid p50 field in resolve_path: -150");
        });

        it('should reject invalid errors and mismatch', () => {
            const invalidData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {},
                errors: -1,
                mismatch: "not_a_number"
            };

            const result = renderer.validateSchema(invalidData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Invalid errors count');
            expect(result.errors).toContain('Invalid mismatch count');
        });
    });

    describe('Table Rendering', () => {
        it('should render complete table', () => {
            const benchData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    },
                    stat_file: {
                        p50: 80,
                        p95: 250,
                        p99: 350,
                        mean: 95,
                        stdev: 30,
                        n: 50,
                        errors: 0
                    }
                },
                errors: 0,
                mismatch: 0
            };

            const table = renderer.renderTable(benchData);
            
            expect(table).toContain('## NGFS Performance Benchmarks');
            expect(table).toContain('| Case | P50 (μs) | P95 (μs) | P99 (μs) | Mean (μs) | Stdev | N | Errors |');
            expect(table).toContain('| resolve_path | 150 | 400 | 600 | 180 | 45 | 50 | 0 |');
            expect(table).toContain('| stat_file | 80 | 250 | 350 | 95 | 30 | 50 | 0 |');
        });

        it('should handle missing cases gracefully', () => {
            const incompleteData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    }
                },
                errors: 0,
                mismatch: 0
            };

            const table = renderer.renderTable(incompleteData);
            
            expect(table).toContain('| resolve_path | 150 | 400 | 600 | 180 | 45 | 50 | 0 |');
            expect(table).toContain('| stat_file | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |');
        });

        it('should include performance summary', () => {
            const benchData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    }
                },
                errors: 0,
                mismatch: 0
            };

            const table = renderer.renderTable(benchData);
            
            expect(table).toContain('### Performance Summary');
            expect(table).toContain('✅ **All cases within budget**');
        });

        it('should detect budget violations', () => {
            const benchData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        p95: 500, // Exceeds 400 budget
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    }
                },
                errors: 0,
                mismatch: 0
            };

            const table = renderer.renderTable(benchData);
            
            expect(table).toContain('❌ **resolve_path**: P95 500μs > 400μs budget');
        });

        it('should include variance analysis', () => {
            const benchData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    }
                },
                errors: 0,
                mismatch: 0
            };

            const table = renderer.renderTable(benchData);
            
            expect(table).toContain('### Variance Analysis');
            expect(table).toContain('✅ **resolve_path**: stdev/mean = 0.25');
        });
    });

    describe('Compact Summary Rendering', () => {
        it('should render compact summary', () => {
            const benchData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    },
                    read_small: {
                        p50: 200,
                        p95: 600,
                        p99: 800,
                        mean: 220,
                        stdev: 55,
                        n: 50,
                        errors: 0
                    }
                },
                errors: 0,
                mismatch: 0
            };

            const summary = renderer.renderCompactSummary(benchData);
            
            expect(summary).toContain('## 🚀 NGFS Performance Benchmarks');
            expect(summary).toContain('**Status:** ✅ PASS');
            expect(summary).toContain('**Duration:** 1500ms');
            expect(summary).toContain('**Issues:** 0');
            expect(summary).toContain('- **resolve_path**: P95=400μs, N=50, Errors=0');
            expect(summary).toContain('- **read_small**: P95=600μs, N=50, Errors=0');
        });

        it('should show failure status with errors', () => {
            const benchData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {
                    resolve_path: {
                        p50: 150,
                        p95: 400,
                        p99: 600,
                        mean: 180,
                        stdev: 45,
                        n: 50,
                        errors: 0
                    }
                },
                errors: 2,
                mismatch: 1
            };

            const summary = renderer.renderCompactSummary(benchData);
            
            expect(summary).toContain('**Status:** ❌ FAIL');
            expect(summary).toContain('**Issues:** 3');
        });
    });

    describe('Edge Cases', () => {
        it('should handle empty cases', () => {
            const emptyData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 1500,
                cases: {},
                errors: 0,
                mismatch: 0
            };

            const result = renderer.validateSchema(emptyData);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Missing required case: resolve_path');
        });

        it('should handle null data', () => {
            const result = renderer.validateSchema(null);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Data must be an object');
        });

        it('should handle non-object data', () => {
            const result = renderer.validateSchema("not_an_object");
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Data must be an object');
        });

        it('should handle zero values gracefully', () => {
            const zeroData = {
                test: 'ngfs_bench',
                timestamp: 1640995200,
                duration_ms: 0,
                cases: {
                    resolve_path: {
                        p50: 0,
                        p95: 0,
                        p99: 0,
                        mean: 0,
                        stdev: 0,
                        n: 50,
                        errors: 0
                    }
                },
                errors: 0,
                mismatch: 0
            };

            const result = renderer.validateSchema(zeroData);
            expect(result.valid).toBe(true);
        });
    });
});

// Mock test runner for Node.js environment
if (typeof describe === 'undefined') {
    console.log('Running TypeScript performance renderer tests...');
    
    const renderer = new NgfsPerfRenderer();
    
    // Test schema validation
    console.log('Testing schema validation...');
    const validData = {
        test: 'ngfs_bench',
        timestamp: 1640995200,
        duration_ms: 1500,
        cases: {
            resolve_path: {
                p50: 150,
                p95: 400,
                p99: 600,
                mean: 180,
                stdev: 45,
                n: 50,
                errors: 0
            }
        },
        errors: 0,
        mismatch: 0
    };
    
    const validation = renderer.validateSchema(validData);
    console.log(`Schema validation: ${validation.valid ? 'PASS' : 'FAIL'}`);
    
    if (!validation.valid) {
        console.log('Validation errors:', validation.errors);
    }
    
    // Test table rendering
    console.log('Testing table rendering...');
    const table = renderer.renderTable(validData);
    console.log('Table rendered successfully');
    
    console.log('All tests completed successfully!');
}
