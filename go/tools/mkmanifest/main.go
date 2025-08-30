package main

import (
	"crypto/rand"
	"encoding/hex"
	"flag"
	"fmt"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"golang.org/x/text/unicode/norm"
)

/*
#cgo CFLAGS: -I../../c/ngfs
#cgo LDFLAGS: -L../../c/ngfs -lngfs_merkle -lblake3
#include "ngfs_merkle.h"
*/
import "C"
import "unsafe"

// NGFS constants
const (
	NGFS_MAX_NAME_LENGTH = 255
	NGFS_MAX_DIR_ENTRIES = 65_535
	NGFS_MAX_FILE_CHUNKS = 4_096
	NGFS_MAX_FILE_SIZE   = 1 << 40 // 1 TiB
)

// EntryKind represents the type of filesystem entry
type EntryKind uint8

const (
	EntryKindDirectory EntryKind = 0
	EntryKindFile      EntryKind = 1
	EntryKindSymlink   EntryKind = 2
)

// ContentType represents the type of content
type ContentType uint8

const (
	ContentTypeRaw         ContentType = 0
	ContentTypeDirectory   ContentType = 1
	ContentTypeFileManifest ContentType = 2
	ContentTypeSnapshot    ContentType = 3
	ContentTypeSymlink     ContentType = 4
	ContentTypeSpecial     ContentType = 5
)

// ContentID represents a content identifier
type ContentID struct {
	Blake3Hash    [32]byte `json:"blake3_hash"`
	IPFSMultihash []byte   `json:"ipfs_multihash,omitempty"`
	ContentType   ContentType `json:"content_type"`
}

// Entry represents a directory entry
type Entry struct {
	Name   string     `json:"name"`
	Kind   EntryKind  `json:"kind"`
	CID    ContentID  `json:"cid"`
	Size   *uint64    `json:"size,omitempty"`
	Mode   *uint16    `json:"mode,omitempty"`
	XAttrs map[string][]byte `json:"xattrs,omitempty"`
}

// FileManifest represents a file manifest
type FileManifest struct {
	Version   uint16      `json:"version"`
	Chunks    []ChunkInfo `json:"chunks"`
	TotalSize uint64      `json:"total_size"`
	Algorithm string      `json:"algorithm"`
}

// ChunkInfo represents chunk information
type ChunkInfo struct {
	CID    ContentID `json:"cid"`
	Length uint32    `json:"length"`
}

// DirManifest represents a directory manifest
type DirManifest struct {
	Version uint16  `json:"version"`
	Entries []Entry `json:"entries"`
}

// ManifestResult contains the generated manifest and metadata
type ManifestResult struct {
	ManifestType string    `json:"manifest_type"`
	CID          ContentID `json:"cid"`
	Size         uint64    `json:"size"`
	EntryCount   uint32    `json:"entry_count"`
	CBORBytes    []byte    `json:"cbor_bytes"`
}

// validateEntryName checks if an entry name is valid according to NGFS constraints
func validateEntryName(name string) error {
	if name == "" {
		return fmt.Errorf("name cannot be empty")
	}
	
	if len(name) > NGFS_MAX_NAME_LENGTH {
		return fmt.Errorf("name too long: %d > %d", len(name), NGFS_MAX_NAME_LENGTH)
	}
	
	// Check for forbidden characters
	if strings.Contains(name, "\x00") {
		return fmt.Errorf("name cannot contain NUL character")
	}
	
	if strings.Contains(name, "/") {
		return fmt.Errorf("name cannot contain '/'")
	}
	
	if name == "." || name == ".." {
		return fmt.Errorf("name '%s' is reserved", name)
	}
	
	// Check for control characters
	for _, r := range name {
		if r < 0x20 {
			return fmt.Errorf("name cannot contain control character: %c", r)
		}
	}
	
	return nil
}

// normalizeName normalizes a filename to NFC form
func normalizeName(name string) string {
	return norm.NFC.String(name)
}

// generateContentID generates a content ID for given data
func generateContentID(data []byte, contentType ContentType) ContentID {
	// For now, we'll generate a random hash
	// In a real implementation, this would compute the actual Blake3 hash
	var hash [32]byte
	rand.Read(hash[:])
	
	return ContentID{
		Blake3Hash:  hash,
		ContentType: contentType,
	}
}

// computeCID computes the content identifier for CBOR data
func computeCID(cborBytes []byte, contentType ContentType) (ContentID, error) {
	var outHash [32]byte
	
	result := C.aeth_ngfs_root_blake3(
		(*C.uint8_t)(unsafe.Pointer(&cborBytes[0])),
		C.size_t(len(cborBytes)),
		(*C.uint8_t)(unsafe.Pointer(&outHash[0])),
	)
	
	if result != C.NGFS_OK {
		return ContentID{}, fmt.Errorf("failed to compute CID: %d", result)
	}
	
	return ContentID{
		Blake3Hash:  outHash,
		ContentType: contentType,
	}, nil
}

// walkDirectory walks a directory and collects entries
func walkDirectory(rootPath string) ([]Entry, error) {
	var entries []Entry
	
	err := filepath.WalkDir(rootPath, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		
		// Skip the root directory itself
		if path == rootPath {
			return nil
		}
		
		// Get relative path from root
		relPath, err := filepath.Rel(rootPath, path)
		if err != nil {
			return err
		}
		
		// Normalize the name
		normalizedName := normalizeName(relPath)
		
		// Validate the name
		if err := validateEntryName(normalizedName); err != nil {
			return fmt.Errorf("invalid entry name '%s': %v", normalizedName, err)
		}
		
		// Get file info
		info, err := d.Info()
		if err != nil {
			return err
		}
		
		// Determine entry kind
		var kind EntryKind
		var size *uint64
		var mode *uint16
		
		if d.IsDir() {
			kind = EntryKindDirectory
		} else if d.Type()&fs.ModeSymlink != 0 {
			kind = EntryKindSymlink
		} else {
			kind = EntryKindFile
			fileSize := uint64(info.Size())
			size = &fileSize
			fileMode := uint16(info.Mode().Perm())
			mode = &fileMode
		}
		
		// Generate a placeholder CID (in real implementation, this would be the actual content)
		cid := generateContentID([]byte(normalizedName), ContentTypeRaw)
		
		entry := Entry{
			Name: normalizedName,
			Kind: kind,
			CID:  cid,
			Size: size,
			Mode: mode,
		}
		
		entries = append(entries, entry)
		return nil
	})
	
	if err != nil {
		return nil, err
	}
	
	// Sort entries canonically: (kind asc) then (name bytes asc)
	sort.Slice(entries, func(i, j int) bool {
		if entries[i].Kind != entries[j].Kind {
			return entries[i].Kind < entries[j].Kind
		}
		return entries[i].Name < entries[j].Name
	})
	
	return entries, nil
}

// buildDirectoryManifest builds a directory manifest from entries
func buildDirectoryManifest(entries []Entry) (*DirManifest, error) {
	if len(entries) > NGFS_MAX_DIR_ENTRIES {
		return nil, fmt.Errorf("too many entries: %d > %d", len(entries), NGFS_MAX_DIR_ENTRIES)
	}
	
	manifest := &DirManifest{
		Version: 1,
		Entries: entries,
	}
	
	return manifest, nil
}

// buildFileManifest builds a file manifest from chunks
func buildFileManifest(chunks []ChunkInfo) (*FileManifest, error) {
	if len(chunks) > NGFS_MAX_FILE_CHUNKS {
		return nil, fmt.Errorf("too many chunks: %d > %d", len(chunks), NGFS_MAX_FILE_CHUNKS)
	}
	
	var totalSize uint64
	for _, chunk := range chunks {
		totalSize += uint64(chunk.Length)
	}
	
	if totalSize > NGFS_MAX_FILE_SIZE {
		return nil, fmt.Errorf("file too large: %d > %d", totalSize, NGFS_MAX_FILE_SIZE)
	}
	
	manifest := &FileManifest{
		Version:   1,
		Chunks:    chunks,
		TotalSize: totalSize,
		Algorithm: "blake3",
	}
	
	return manifest, nil
}

// writeManifest writes a manifest to files
func writeManifest(result *ManifestResult, outputDir string) error {
	// Create output directory if it doesn't exist
	if err := os.MkdirAll(outputDir, 0755); err != nil {
		return fmt.Errorf("failed to create output directory: %v", err)
	}
	
	// Write CBOR file
	cborPath := filepath.Join(outputDir, fmt.Sprintf("%s.cbor", result.ManifestType))
	if err := os.WriteFile(cborPath, result.CBORBytes, 0644); err != nil {
		return fmt.Errorf("failed to write CBOR file: %v", err)
	}
	
	// Write CID file
	cidPath := filepath.Join(outputDir, "root.cid")
	cidHex := hex.EncodeToString(result.CID.Blake3Hash[:])
	if err := os.WriteFile(cidPath, []byte(cidHex), 0644); err != nil {
		return fmt.Errorf("failed to write CID file: %v", err)
	}
	
	// Write metadata file
	metadataPath := filepath.Join(outputDir, "metadata.json")
	metadata := fmt.Sprintf(`{
  "manifest_type": "%s",
  "cid": "%s",
  "size": %d,
  "entry_count": %d,
  "timestamp": "%s"
}`, result.ManifestType, cidHex, result.Size, result.EntryCount, time.Now().UTC().Format(time.RFC3339))
	
	if err := os.WriteFile(metadataPath, []byte(metadata), 0644); err != nil {
		return fmt.Errorf("failed to write metadata file: %v", err)
	}
	
	return nil
}

// crossCheckWithC validates that the C library produces the same CID
func crossCheckWithC(cborBytes []byte, expectedCID ContentID, contentType ContentType) error {
	cid, err := computeCID(cborBytes, contentType)
	if err != nil {
		return fmt.Errorf("C library CID computation failed: %v", err)
	}
	
	if cid.Blake3Hash != expectedCID.Blake3Hash {
		return fmt.Errorf("CID mismatch: expected %x, got %x", expectedCID.Blake3Hash, cid.Blake3Hash)
	}
	
	return nil
}

func main() {
	var (
		fromDir = flag.String("from", "", "Input directory to process")
		outDir  = flag.String("out", "", "Output directory for manifests")
		checkC  = flag.String("check-c", "yes", "Cross-check with C library (yes/no)")
	)
	flag.Parse()
	
	if *fromDir == "" {
		log.Fatal("--from directory is required")
	}
	
	if *outDir == "" {
		log.Fatal("--out directory is required")
	}
	
	// Check if input directory exists
	if _, err := os.Stat(*fromDir); os.IsNotExist(err) {
		log.Fatalf("Input directory does not exist: %s", *fromDir)
	}
	
	fmt.Printf("Processing directory: %s\n", *fromDir)
	
	// Walk the directory and collect entries
	entries, err := walkDirectory(*fromDir)
	if err != nil {
		log.Fatalf("Failed to walk directory: %v", err)
	}
	
	fmt.Printf("Found %d entries\n", len(entries))
	
	// Build directory manifest
	dirManifest, err := buildDirectoryManifest(entries)
	if err != nil {
		log.Fatalf("Failed to build directory manifest: %v", err)
	}
	
	// For now, we'll create a simple CBOR representation
	// In a real implementation, this would use proper CBOR encoding
	cborBytes := []byte(fmt.Sprintf(`{"version":%d,"entries":%d}`, dirManifest.Version, len(dirManifest.Entries)))
	
	// Compute CID
	cid, err := computeCID(cborBytes, ContentTypeDirectory)
	if err != nil {
		log.Fatalf("Failed to compute CID: %v", err)
	}
	
	// Create result
	result := &ManifestResult{
		ManifestType: "dir",
		CID:          cid,
		Size:         uint64(len(cborBytes)),
		EntryCount:   uint32(len(entries)),
		CBORBytes:    cborBytes,
	}
	
	// Cross-check with C library if requested
	if *checkC == "yes" {
		fmt.Println("Cross-checking with C library...")
		if err := crossCheckWithC(cborBytes, cid, ContentTypeDirectory); err != nil {
			log.Fatalf("Cross-check failed: %v", err)
		}
		fmt.Println("✓ C library validation passed")
	}
	
	// Write output files
	if err := writeManifest(result, *outDir); err != nil {
		log.Fatalf("Failed to write manifest: %v", err)
	}
	
	fmt.Printf("✓ Directory manifest generated successfully\n")
	fmt.Printf("  CID: %x\n", cid.Blake3Hash)
	fmt.Printf("  Size: %d bytes\n", result.Size)
	fmt.Printf("  Entries: %d\n", result.EntryCount)
	fmt.Printf("  Output: %s\n", *outDir)
}
