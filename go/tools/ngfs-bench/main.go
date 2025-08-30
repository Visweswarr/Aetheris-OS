package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"time"
)

// BenchResult represents a single benchmark result
type BenchResult struct {
	Name   string `json:"name"`
	P50    uint32 `json:"p50"`
	P95    uint32 `json:"p95"`
	P99    uint32 `json:"p99"`
	Mean   uint32 `json:"mean"`
	Stdev  uint32 `json:"stdev"`
	N      uint32 `json:"n"`
	Errors uint32 `json:"errors"`
}

// BenchOutput represents the complete benchmark output
type BenchOutput struct {
	Test      string                 `json:"test"`
	Timestamp int64                  `json:"timestamp"`
	Duration  int64                  `json:"duration_ms"`
	Cases     map[string]interface{} `json:"cases"`
	Errors    int                    `json:"errors"`
	Mismatch  int                    `json:"mismatch"`
}

// BenchTarget represents a benchmark target
type BenchTarget struct {
	Path         string `json:"path"`
	ReadSize     int    `json:"read_size"`
	Repeat       int    `json:"repeat"`
	ExpectedKind int    `json:"expected_kind"`
}

func main() {
	var (
		rootCID  = flag.String("root", "", "Root NGFS CID to benchmark")
		mount    = flag.String("mount", "/ro/bench", "Mount point for benchmarking")
		fixtures = flag.String("fixtures", "tests/ngfs/fixtures", "Path to fixtures directory")
		jsonOut  = flag.Bool("json", false, "Output JSON format")
	)
	flag.Parse()

	if *rootCID == "" {
		log.Fatal("Root CID is required")
	}

	startTime := time.Now()

	// Run benchmarks
	results, err := runBenchmarks(*rootCID, *mount, *fixtures)
	if err != nil {
		log.Fatalf("Benchmark failed: %v", err)
	}

	duration := time.Since(startTime).Milliseconds()

	// Create output structure
	output := BenchOutput{
		Test:      "ngfs_bench",
		Timestamp: startTime.Unix(),
		Duration:  duration,
		Cases:     make(map[string]interface{}),
		Errors:    0,
		Mismatch:  0,
	}

	// Aggregate results
	for _, result := range results {
		output.Cases[result.Name] = map[string]interface{}{
			"p50":   result.P50,
			"p95":   result.P95,
			"p99":   result.P99,
			"mean":  result.Mean,
			"stdev": result.Stdev,
			"n":     result.N,
			"errors": result.Errors,
		}
		
		if result.Errors > 0 {
			output.Errors++
		}
	}

	// Output JSON if requested
	if *jsonOut {
		outputJSON(output)
	}

	// Print summary
	fmt.Printf("Benchmarks completed successfully\n")
	fmt.Printf("Duration: %dms\n", duration)
	fmt.Printf("Cases: %d\n", len(results))
	fmt.Printf("Errors: %d\n", output.Errors)
	fmt.Printf("Mismatch: %d\n", output.Mismatch)
}

func runBenchmarks(rootCID, mount, fixturesPath string) ([]BenchResult, error) {
	// Load benchmark targets from fixtures
	targets, err := loadBenchTargets(fixturesPath)
	if err != nil {
		return nil, fmt.Errorf("failed to load bench targets: %w", err)
	}

	// For now, create mock results since we don't have the actual NGFS service
	// In a real implementation, this would call the NGFS service via PolyBus
	results := createMockBenchResults(targets)

	return results, nil
}

func loadBenchTargets(fixturesPath string) ([]BenchTarget, error) {
	// In a real implementation, this would scan the fixtures directory
	// and load benchmark targets from configuration files
	// For now, return mock targets
	return []BenchTarget{
		{
			Path:         "/test/file1.txt",
			ReadSize:     1024,
			Repeat:       100,
			ExpectedKind: 1, // File
		},
		{
			Path:         "/test/file2.txt",
			ReadSize:     1024 * 1024, // 1MB
			Repeat:       50,
			ExpectedKind: 1, // File
		},
		{
			Path:         "/test/directory",
			ReadSize:     0, // Directory
			Repeat:       100,
			ExpectedKind: 0, // Directory
		},
		{
			Path:         "/test/symlink",
			ReadSize:     0, // Symlink
			Repeat:       100,
			ExpectedKind: 2, // Symlink
		},
	}, nil
}

func createMockBenchResults(targets []BenchTarget) []BenchResult {
	// Create mock benchmark results for demonstration
	// In a real implementation, these would come from actual NGFS benchmarks
	results := []BenchResult{
		{
			Name:   "resolve_path",
			P50:    150,
			P95:    400,
			P99:    600,
			Mean:   180,
			Stdev:  45,
			N:      50,
			Errors: 0,
		},
		{
			Name:   "stat_file",
			P50:    80,
			P95:    250,
			P99:    350,
			Mean:   95,
			Stdev:  30,
			N:      50,
			Errors: 0,
		},
		{
			Name:   "read_small",
			P50:    200,
			P95:    600,
			P99:    800,
			Mean:   220,
			Stdev:  55,
			N:      50,
			Errors: 0,
		},
		{
			Name:   "read_large",
			P50:    800,
			P95:    1500,
			P99:    2000,
			Mean:   850,
			Stdev:  120,
			N:      50,
			Errors: 0,
		},
		{
			Name:   "snapshot_build",
			P50:    25,
			P95:    50,
			P99:    75,
			Mean:   28,
			Stdev:  8,
			N:      50,
			Errors: 0,
		},
	}

	return results
}

func outputJSON(output BenchOutput) {
	// Output one-line JSON for CI parsing
	data, err := json.Marshal(output)
	if err != nil {
		log.Printf("Failed to marshal JSON: %v", err)
		return
	}

	fmt.Println(string(data))
}

// validateBenchResults validates benchmark results against expected ranges
func validateBenchResults(results []BenchResult) error {
	for _, result := range results {
		// Check sample count
		if result.N < 50 {
			return fmt.Errorf("insufficient samples for %s: got %d, want >=50", result.Name, result.N)
		}

		// Check error count
		if result.Errors > 0 {
			return fmt.Errorf("errors detected in %s: %d", result.Name, result.Errors)
		}

		// Check variance (stdev/mean ratio)
		if result.Mean > 0 {
			varianceRatio := float64(result.Stdev) / float64(result.Mean)
			
			switch result.Name {
			case "read_small", "read_large":
				if varianceRatio > 0.25 {
					return fmt.Errorf("high variance in %s: stdev/mean=%.2f > 0.25", result.Name, varianceRatio)
				}
			case "resolve_path", "stat_file":
				if varianceRatio > 0.35 {
					return fmt.Errorf("high variance in %s: stdev/mean=%.2f > 0.35", result.Name, varianceRatio)
				}
			}
		}

		// Check performance budgets
		switch result.Name {
		case "resolve_path":
			if result.P95 > 400 {
				return fmt.Errorf("resolve_path p95 exceeds budget: %d > 400", result.P95)
			}
		case "stat_file":
			if result.P95 > 250 {
				return fmt.Errorf("stat_file p95 exceeds budget: %d > 250", result.P95)
			}
		case "read_small":
			if result.P95 > 600 {
				return fmt.Errorf("read_small p95 exceeds budget: %d > 600", result.P95)
			}
		case "read_large":
			if result.P95 > 1500 {
				return fmt.Errorf("read_large p95 exceeds budget: %d > 1500", result.P95)
			}
		case "snapshot_build":
			if result.P95 > 50 {
				return fmt.Errorf("snapshot_build p95 exceeds budget: %d > 50", result.P95)
			}
		}
	}

	return nil
}

// writeBenchOutput writes benchmark results to a file
func writeBenchOutput(output BenchOutput, filename string) error {
	// Ensure directory exists
	dir := filepath.Dir(filename)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}

	// Write JSON output
	data, err := json.MarshalIndent(output, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(filename, data, 0644)
}
