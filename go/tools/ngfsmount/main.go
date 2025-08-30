package main

import (
	"encoding/json"
	"flag"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
)

// MountResult represents the result of a mount operation
type MountResult struct {
	Tool    string `json:"tool"`
	Op      string `json:"op"`
	Ok      bool   `json:"ok"`
	Error   string `json:"error,omitempty"`
	Path    string `json:"path,omitempty"`
	RootCID string `json:"root_cid,omitempty"`
}

// UnmountResult represents the result of an unmount operation
type UnmountResult struct {
	Tool  string `json:"tool"`
	Op    string `json:"op"`
	Ok    bool   `json:"ok"`
	Error string `json:"error,omitempty"`
	Path  string `json:"path,omitempty"`
}

func main() {
	// Parse command line flags
	var (
		storePath = flag.String("store", "", "NGFS store file path")
		idxPath   = flag.String("idx", "", "NGFS index file path")
		mountPath = flag.String("at", "", "Mount point directory")
		rootCID   = flag.String("root", "", "Root directory CID")
		keydir    = flag.String("keydir", "", "Key directory for dev KEK/DEK fixtures")
		jsonOut   = flag.Bool("json", false, "Output results as JSON")
		help      = flag.Bool("help", false, "Show help")
	)
	flag.Parse()

	// Show help if requested
	if *help {
		showHelp()
		return
	}

	// Check if this is a mount or unmount operation
	args := flag.Args()
	if len(args) == 0 {
		fmt.Fprintf(os.Stderr, "Error: No operation specified. Use 'mount' or 'unmount'.\n")
		showHelp()
		os.Exit(1)
	}

	operation := args[0]

	switch operation {
	case "mount":
		handleMount(*storePath, *idxPath, *mountPath, *rootCID, *keydir, *jsonOut)
	case "unmount":
		if len(args) < 2 {
			fmt.Fprintf(os.Stderr, "Error: Unmount requires a mount point path.\n")
			showHelp()
			os.Exit(1)
		}
		handleUnmount(args[1], *jsonOut)
	default:
		fmt.Fprintf(os.Stderr, "Error: Unknown operation '%s'. Use 'mount' or 'unmount'.\n", operation)
		showHelp()
		os.Exit(1)
	}
}

func handleMount(storePath, idxPath, mountPath, rootCID, keydir string, jsonOut bool) {
	// Validate required parameters
	if storePath == "" {
		outputError("mount", "store path is required", jsonOut)
		os.Exit(1)
	}
	if idxPath == "" {
		outputError("mount", "index path is required", jsonOut)
		os.Exit(1)
	}
	if mountPath == "" {
		outputError("mount", "mount point is required", jsonOut)
		os.Exit(1)
	}
	if rootCID == "" {
		outputError("mount", "root CID is required", jsonOut)
		os.Exit(1)
	}

	// Check if store and index files exist
	if !fileExists(storePath) {
		outputError("mount", fmt.Sprintf("store file not found: %s", storePath), jsonOut)
		os.Exit(1)
	}
	if !fileExists(idxPath) {
		outputError("mount", fmt.Sprintf("index file not found: %s", idxPath), jsonOut)
		os.Exit(1)
	}

	// Check if mount point exists and is a directory
	if !fileExists(mountPath) {
		outputError("mount", fmt.Sprintf("mount point not found: %s", mountPath), jsonOut)
		os.Exit(1)
	}

	// Check if mount point is already mounted
	if isMounted(mountPath) {
		outputError("mount", fmt.Sprintf("mount point already in use: %s", mountPath), jsonOut)
		os.Exit(1)
	}

	// Build the ngfs-fuse command
	cmd := exec.Command("ngfs-fuse",
		"--store", storePath,
		"--idx", idxPath,
		"--at", mountPath,
		"--root", rootCID,
	)

	// Add keydir if specified
	if keydir != "" {
		if !fileExists(keydir) {
			outputError("mount", fmt.Sprintf("key directory not found: %s", keydir), jsonOut)
			os.Exit(1)
		}
		cmd.Args = append(cmd.Args, "--keydir", keydir)
	}

	// Start the mount process
	startTime := time.Now()
	err := cmd.Start()
	if err != nil {
		outputError("mount", fmt.Sprintf("failed to start ngfs-fuse: %v", err), jsonOut)
		os.Exit(1)
	}

	// Wait a bit for the mount to complete
	time.Sleep(2 * time.Second)

	// Check if the mount was successful
	if !isMounted(mountPath) {
		// Kill the process if mount failed
		cmd.Process.Kill()
		outputError("mount", "mount operation failed", jsonOut)
		os.Exit(1)
	}

	duration := time.Since(startTime)

	// Output success result
	result := MountResult{
		Tool:    "ngfsmount",
		Op:      "mount",
		Ok:      true,
		Path:    mountPath,
		RootCID: rootCID,
	}

	if jsonOut {
		outputJSON(result)
	} else {
		fmt.Printf("Successfully mounted NGFS at %s (root: %s) in %v\n", mountPath, rootCID, duration)
	}
}

func handleUnmount(mountPath string, jsonOut bool) {
	// Check if mount point exists
	if !fileExists(mountPath) {
		outputError("unmount", fmt.Sprintf("mount point not found: %s", mountPath), jsonOut)
		os.Exit(1)
	}

	// Check if it's actually mounted
	if !isMounted(mountPath) {
		outputError("unmount", fmt.Sprintf("mount point not mounted: %s", mountPath), jsonOut)
		os.Exit(1)
	}

	// Try to unmount using fusermount
	startTime := time.Now()
	cmd := exec.Command("fusermount", "-u", mountPath)
	err := cmd.Run()

	duration := time.Since(startTime)

	if err != nil {
		// Try alternative unmount command for macOS
		cmd = exec.Command("umount", mountPath)
		err = cmd.Run()
	}

	if err != nil {
		outputError("unmount", fmt.Sprintf("failed to unmount: %v", err), jsonOut)
		os.Exit(1)
	}

	// Output success result
	result := UnmountResult{
		Tool: "ngfsmount",
		Op:   "unmount",
		Ok:   true,
		Path: mountPath,
	}

	if jsonOut {
		outputJSON(result)
	} else {
		fmt.Printf("Successfully unmounted NGFS from %s in %v\n", mountPath, duration)
	}
}

func fileExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

func isMounted(mountPath string) bool {
	// Check if the mount point is in use by trying to list its contents
	cmd := exec.Command("ls", mountPath)
	err := cmd.Run()
	return err == nil
}

func outputError(op, message string, jsonOut bool) {
	if jsonOut {
		var result interface{}
		switch op {
		case "mount":
			result = MountResult{
				Tool:  "ngfsmount",
				Op:    op,
				Ok:    false,
				Error: message,
			}
		case "unmount":
			result = UnmountResult{
				Tool:  "ngfsmount",
				Op:    op,
				Ok:    false,
				Error: message,
			}
		}
		outputJSON(result)
	} else {
		fmt.Fprintf(os.Stderr, "Error: %s\n", message)
	}
}

func outputJSON(data interface{}) {
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetIndent("", "")
	encoder.Encode(data)
}

func showHelp() {
	fmt.Printf(`NGFS Mount Tool

Usage:
  ngfsmount mount [flags]
  ngfsmount unmount <mount_point>

Mount flags:
  -store string
        NGFS store file path
  -idx string
        NGFS index file path
  -at string
        Mount point directory
  -root string
        Root directory CID
  -keydir string
        Key directory for dev KEK/DEK fixtures (optional)
  -json
        Output results as JSON

Examples:
  # Mount NGFS snapshot
  ngfsmount mount --store ngfs.dat --idx ngfs.idx --at ./mnt --root <root_cid>

  # Mount with key directory
  ngfsmount mount --store ngfs.dat --idx ngfs.idx --at ./mnt --root <root_cid> --keydir ./keys

  # Unmount
  ngfsmount unmount ./mnt

  # JSON output
  ngfsmount mount --store ngfs.dat --idx ngfs.idx --at ./mnt --root <root_cid> --json
`)
}
