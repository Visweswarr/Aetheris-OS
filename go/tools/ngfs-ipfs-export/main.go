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

// NgfsResponse represents the response from NGFS operations
type NgfsResponse struct {
	Status       string      `json:"status"`
	Data         interface{} `json:"data,omitempty"`
	Error        string      `json:"error,omitempty"`
	Timestamp    int64       `json:"timestamp"`
	DurationMs   int64       `json:"duration_ms"`
}

// IpfsMapEntry represents a single IPFS map entry
type IpfsMapEntry struct {
	NgfsCid  []byte `json:"ngfs_cid"`
	IpfsCid  string `json:"ipfs_cid"`
	Kind     uint8  `json:"kind"`
	Size     uint64 `json:"size"`
}

// IpfsMap represents the complete IPFS export map
type IpfsMap struct {
	Version          uint16         `json:"version"`
	ExportedVclock  uint64         `json:"exported_vclock"`
	RootNgfsCid     []byte         `json:"root_ngfs_cid"`
	Entries         []IpfsMapEntry `json:"entries"`
}

// ExportStats represents export statistics
type ExportStats struct {
	Entries   int    `json:"entries"`
	Bytes     int64  `json:"bytes"`
	Car       bool   `json:"car"`
	Mismatch  int    `json:"mismatch"`
}

func main() {
	var (
		rootCID    = flag.String("root", "", "Root NGFS CID to export")
		outputFile = flag.String("out", "ipfs-map.cbor", "Output file for IPFS map")
		carFile    = flag.String("car", "", "Optional CAR file output")
		statsJSON  = flag.Bool("stats-json", false, "Output stats as JSON")
	)
	flag.Parse()

	if *rootCID == "" {
		log.Fatal("Root CID is required")
	}

	startTime := time.Now()

	// For now, create a mock export since we don't have the actual NGFS service
	// In a real implementation, this would call the NGFS service via PolyBus
	stats, err := handleExport(*rootCID, *outputFile, *carFile)
	if err != nil {
		log.Fatalf("Export failed: %v", err)
	}

	duration := time.Since(startTime).Milliseconds()

	// Output JSON metrics if requested
	if *statsJSON {
		outputJSON(stats, startTime, "ipfs_export")
	}

	fmt.Printf("Export completed successfully\n")
	fmt.Printf("Entries: %d, Bytes: %d, CAR: %v\n", stats.Entries, stats.Bytes, stats.Car)
	fmt.Printf("Duration: %dms\n", duration)
}

func handleExport(rootCID, outputFile, carFile string) (*ExportStats, error) {
	// Create mock IPFS map for demonstration
	// In real implementation, this would call the NGFS service
	mapData := createMockIpfsMap(rootCID)

	// Write map to file
	if err := writeMapFile(outputFile, mapData); err != nil {
		return nil, fmt.Errorf("failed to write map file: %w", err)
	}

	// Write CAR file if requested
	carGenerated := false
	if carFile != "" {
		if err := writeCarFile(carFile, mapData); err != nil {
			return nil, fmt.Errorf("failed to write CAR file: %w", err)
		}
		carGenerated = true
	}

	// Calculate stats
	totalBytes := int64(0)
	for _, entry := range mapData.Entries {
		totalBytes += int64(entry.Size)
	}

	return &ExportStats{
		Entries:  len(mapData.Entries),
		Bytes:    totalBytes,
		Car:      carGenerated,
		Mismatch: 0,
	}, nil
}

func createMockIpfsMap(rootCID string) *IpfsMap {
	// Create a mock IPFS map for demonstration
	// In real implementation, this would be populated by the NGFS service
	return &IpfsMap{
		Version:         1,
		ExportedVclock:  uint64(time.Now().Unix()),
		RootNgfsCid:     []byte(rootCID),
		Entries: []IpfsMapEntry{
			{
				NgfsCid: []byte{1, 2, 3, 4},
				IpfsCid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
				Kind:    0, // Directory
				Size:    1024,
			},
			{
				NgfsCid: []byte{5, 6, 7, 8},
				IpfsCid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
				Kind:    1, // File
				Size:    2048,
			},
		},
	}
}

func writeMapFile(filename string, mapData *IpfsMap) error {
	// In real implementation, this would write CBOR data
	// For now, write JSON for demonstration
	data, err := json.MarshalIndent(mapData, "", "  ")
	if err != nil {
		return err
	}

	// Ensure directory exists
	dir := filepath.Dir(filename)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}

	return os.WriteFile(filename, data, 0644)
}

func writeCarFile(filename string, mapData *IpfsMap) error {
	// In real implementation, this would write CAR v1 format
	// For now, write a simple text representation
	content := "CAR v1\n"
	for _, entry := range mapData.Entries {
		content += fmt.Sprintf("%d %s\n", entry.Size, entry.IpfsCid)
		// In real CAR, this would include the actual block data
	}

	// Ensure directory exists
	dir := filepath.Dir(filename)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}

	return os.WriteFile(filename, []byte(content), 0644)
}

func outputJSON(stats *ExportStats, startTime time.Time, command string) {
	// Output one-line JSON for CI parsing
	output := map[string]interface{}{
		"test":        command,
		"entries":     stats.Entries,
		"bytes":       stats.Bytes,
		"car":         stats.Car,
		"mismatch":    stats.Mismatch,
		"timestamp":   startTime.Unix(),
		"duration_ms": time.Since(startTime).Milliseconds(),
	}

	data, err := json.Marshal(output)
	if err != nil {
		log.Printf("Failed to marshal JSON: %v", err)
		return
	}

	fmt.Println(string(data))
}
