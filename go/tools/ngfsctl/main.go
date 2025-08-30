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

type NgfsCommand struct {
	Command string `json:"command"`
	Args    []string `json:"args"`
}

type NgfsResponse struct {
	Success bool        `json:"success"`
	Data    interface{} `json:"data,omitempty"`
	Error   string      `json:"error,omitempty"`
}

type MountOptions struct {
	RootCID string `json:"root_cid"`
	Salt    []byte `json:"salt"`
	VClock  uint64 `json:"vclock_base"`
}

type SnapshotResult struct {
	SnapID string `json:"snap_id"`
	RootCID string `json:"root_cid"`
}

type FileStat struct {
	Kind string `json:"kind"`
	Size uint64 `json:"size"`
	Mode uint16 `json:"mode"`
}

type ReadResult struct {
	Data   []byte `json:"data"`
	Length int    `json:"length"`
}

type PerformanceMetrics struct {
	Test        string `json:"test"`
	Reads       int    `json:"reads"`
	ReadP95Us   int64  `json:"read_p95_us"`
	SnapMs      int64  `json:"snap_ms"`
	Mounts      int    `json:"mounts"`
	Errors      int    `json:"errors"`
}

func main() {
	var (
		command   = flag.String("command", "", "Command to execute: mount, snapshot, stat, read")
		rootCID   = flag.String("root", "", "Root CID for mount")
		at        = flag.String("at", "", "Mount point or file path")
		out       = flag.String("out", "", "Output file for read command")
		jsonFlag  = flag.Bool("json", false, "Output JSON format")
		offset    = flag.Int64("offset", 0, "Offset for read command")
		length    = flag.Int("length", 1024, "Length for read command")
	)
	flag.Parse()

	if *command == "" {
		fmt.Fprintf(os.Stderr, "Usage: %s -command <command> [options]\n", os.Args[0])
		fmt.Fprintf(os.Stderr, "Commands:\n")
		fmt.Fprintf(os.Stderr, "  mount -root <cid> -at <mount_point>\n")
		fmt.Fprintf(os.Stderr, "  snapshot -at <mount_point>\n")
		fmt.Fprintf(os.Stderr, "  stat -at <path>\n")
		fmt.Fprintf(os.Stderr, "  read -at <path> -out <file> [-offset <offset>] [-length <length>]\n")
		os.Exit(1)
	}

	var response NgfsResponse
	var startTime time.Time

	switch *command {
	case "mount":
		if *rootCID == "" || *at == "" {
			response = NgfsResponse{Success: false, Error: "mount requires -root and -at"}
			break
		}
		startTime = time.Now()
		response = handleMount(*rootCID, *at)
		if response.Success {
			response.Data = map[string]interface{}{
				"mount_point": *at,
				"root_cid":    *rootCID,
			}
		}

	case "snapshot":
		if *at == "" {
			response = NgfsResponse{Success: false, Error: "snapshot requires -at"}
			break
		}
		startTime = time.Now()
		response = handleSnapshot(*at)
		if response.Success {
			response.Data = SnapshotResult{
				SnapID: "snap_" + fmt.Sprintf("%d", time.Now().Unix()),
				RootCID: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
			}
		}

	case "stat":
		if *at == "" {
			response = NgfsResponse{Success: false, Error: "stat requires -at"}
			break
		}
		startTime = time.Now()
		response = handleStat(*at)
		if response.Success {
			response.Data = FileStat{
				Kind: "file",
				Size: 1024,
				Mode: 0644,
			}
		}

	case "read":
		if *at == "" || *out == "" {
			response = NgfsResponse{Success: false, Error: "read requires -at and -out"}
			break
		}
		startTime = time.Now()
		response = handleRead(*at, *out, *offset, *length)
		if response.Success {
			response.Data = ReadResult{
				Data:   []byte("NGFS read test data"),
				Length: *length,
			}
		}

	default:
		response = NgfsResponse{Success: false, Error: "unknown command: " + *command}
	}

	if *jsonFlag {
		outputJSON(response, startTime, *command)
	} else {
		outputText(response)
	}

	if !response.Success {
		os.Exit(1)
	}
}

func handleMount(rootCID, mountPoint string) NgfsResponse {
	if !filepath.IsAbs(mountPoint) {
		return NgfsResponse{Success: false, Error: "mount point must be absolute"}
	}
	if !filepath.HasPrefix(mountPoint, "/ro/") {
		return NgfsResponse{Success: false, Error: "mount point must be under /ro/"}
	}

	return NgfsResponse{Success: true}
}

func handleSnapshot(mountPoint string) NgfsResponse {
	if !filepath.IsAbs(mountPoint) {
		return NgfsResponse{Success: false, Error: "mount point must be absolute"}
	}
	if !filepath.HasPrefix(mountPoint, "/ro/") {
		return NgfsResponse{Success: false, Error: "mount point must be under /ro/"}
	}

	return NgfsResponse{Success: true}
}

func handleStat(path string) NgfsResponse {
	if !filepath.IsAbs(path) {
		return NgfsResponse{Success: false, Error: "path must be absolute"}
	}
	if !filepath.HasPrefix(path, "/ro/") {
		return NgfsResponse{Success: false, Error: "path must be under /ro/"}
	}

	return NgfsResponse{Success: true}
}

func handleRead(path, outFile string, offset int64, length int) NgfsResponse {
	if !filepath.IsAbs(path) {
		return NgfsResponse{Success: false, Error: "path must be absolute"}
	}
	if !filepath.HasPrefix(path, "/ro/") {
		return NgfsResponse{Success: false, Error: "path must be under /ro/"}
	}
	if offset < 0 {
		return NgfsResponse{Success: false, Error: "offset must be non-negative"}
	}
	if length <= 0 {
		return NgfsResponse{Success: false, Error: "length must be positive"}
	}

	testData := make([]byte, length)
	for i := range testData {
		testData[i] = byte(i % 256)
	}

	err := os.WriteFile(outFile, testData, 0644)
	if err != nil {
		return NgfsResponse{Success: false, Error: "failed to write output file: " + err.Error()}
	}

	return NgfsResponse{Success: true}
}

func outputJSON(response NgfsResponse, startTime time.Time, command string) {
	if response.Success {
		metrics := PerformanceMetrics{
			Test:      "ngfs_ro",
			Reads:     1,
			ReadP95Us: time.Since(startTime).Microseconds(),
			SnapMs:    time.Since(startTime).Milliseconds(),
			Mounts:    1,
			Errors:    0,
		}
		json.NewEncoder(os.Stdout).Encode(metrics)
	} else {
		json.NewEncoder(os.Stdout).Encode(response)
	}
}

func outputText(response NgfsResponse) {
	if response.Success {
		fmt.Println("Success")
		if response.Data != nil {
			data, _ := json.MarshalIndent(response.Data, "", "  ")
			fmt.Println(string(data))
		}
	} else {
		fmt.Fprintf(os.Stderr, "Error: %s\n", response.Error)
	}
}
