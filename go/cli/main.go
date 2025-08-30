package main

import (
	"crypto/rand"
	"encoding/hex"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
)

/*
#cgo CFLAGS: -I../../c/crypto
#cgo LDFLAGS: -L../../c/crypto -lpolycrypto -lsodium
#include "polycrypto.h"
*/
import "C"
import "unsafe"

// Seal encrypts data using XChaCha20-Poly1305
func Seal(key, nonce, ad, pt []byte) ([]byte, []byte, error) {
	if len(key) != C.AETH_KEY_SIZE {
		return nil, nil, fmt.Errorf("invalid key size: expected %d, got %d", C.AETH_KEY_SIZE, len(key))
	}
	if len(nonce) != C.AETH_NONCE_SIZE {
		return nil, nil, fmt.Errorf("invalid nonce size: expected %d, got %d", C.AETH_NONCE_SIZE, len(nonce))
	}
	if len(pt) > C.AETH_MAX_PAYLOAD_SIZE {
		return nil, nil, fmt.Errorf("payload too large: %d > %d", len(pt), C.AETH_MAX_PAYLOAD_SIZE)
	}

	ct := make([]byte, len(pt))
	tag := make([]byte, C.AETH_TAG_SIZE)

	var adPtr *C.uint8_t
	if len(ad) > 0 {
		adPtr = (*C.uint8_t)(unsafe.Pointer(&ad[0]))
	}

	result := C.aeth_xchacha20_seal(
		(*C.uint8_t)(unsafe.Pointer(&key[0])),
		(*C.uint8_t)(unsafe.Pointer(&nonce[0])),
		adPtr,
		C.size_t(len(ad)),
		(*C.uint8_t)(unsafe.Pointer(&pt[0])),
		C.size_t(len(pt)),
		(*C.uint8_t)(unsafe.Pointer(&ct[0])),
		(*C.uint8_t)(unsafe.Pointer(&tag[0])),
	)

	if result != C.AETH_OK {
		return nil, nil, fmt.Errorf("encryption failed with code %d", result)
	}

	return ct, tag, nil
}

// Open decrypts data using XChaCha20-Poly1305
func Open(key, nonce, ad, ct, tag []byte) ([]byte, error) {
	if len(key) != C.AETH_KEY_SIZE {
		return nil, fmt.Errorf("invalid key size: expected %d, got %d", C.AETH_KEY_SIZE, len(key))
	}
	if len(nonce) != C.AETH_NONCE_SIZE {
		return nil, fmt.Errorf("invalid nonce size: expected %d, got %d", C.AETH_NONCE_SIZE, len(nonce))
	}
	if len(tag) != C.AETH_TAG_SIZE {
		return nil, fmt.Errorf("invalid tag size: expected %d, got %d", C.AETH_TAG_SIZE, len(tag))
	}

	pt := make([]byte, len(ct))

	var adPtr *C.uint8_t
	if len(ad) > 0 {
		adPtr = (*C.uint8_t)(unsafe.Pointer(&ad[0]))
	}

	result := C.aeth_xchacha20_open(
		(*C.uint8_t)(unsafe.Pointer(&key[0])),
		(*C.uint8_t)(unsafe.Pointer(&nonce[0])),
		adPtr,
		C.size_t(len(ad)),
		(*C.uint8_t)(unsafe.Pointer(&ct[0])),
		C.size_t(len(ct)),
		(*C.uint8_t)(unsafe.Pointer(&tag[0])),
		(*C.uint8_t)(unsafe.Pointer(&pt[0])),
	)

	if result != C.AETH_OK {
		return nil, fmt.Errorf("decryption failed with code %d", result)
	}

	return pt, nil
}

// generateTestVectors creates test vectors for cross-language validation
func generateTestVectors() error {
	// Fixed test data
	key := make([]byte, C.AETH_KEY_SIZE)
	for i := range key {
		key[i] = byte(i + 1)
	}

	nonce := make([]byte, C.AETH_NONCE_SIZE)
	for i := range nonce {
		nonce[i] = byte(i + 100)
	}

	ad := []byte("associated data for testing")
	pt := []byte("plaintext data for encryption testing")

	// Encrypt
	ct, tag, err := Seal(key, nonce, ad, pt)
	if err != nil {
		return fmt.Errorf("seal failed: %v", err)
	}

	// Decrypt to verify
	pt2, err := Open(key, nonce, ad, ct, tag)
	if err != nil {
		return fmt.Errorf("open failed: %v", err)
	}

	if string(pt) != string(pt2) {
		return fmt.Errorf("roundtrip failed: plaintext mismatch")
	}

	// Write test vectors to files
	testVectors := map[string][]byte{
		"key.bin":     key,
		"nonce.bin":   nonce,
		"ad.bin":      ad,
		"pt.bin":      pt,
		"ct.bin":      ct,
		"tag.bin":     tag,
	}

	for filename, data := range testVectors {
		if err := os.WriteFile(filename, data, 0644); err != nil {
			return fmt.Errorf("failed to write %s: %v", filename, err)
		}
		fmt.Printf("Generated %s (%d bytes)\n", filename, len(data))
	}

	return nil
}

// runFixtures runs the fixtures test
func runFixtures(fixturesDir string) error {
	// Read fixture files
	files := map[string]string{
		"key":   filepath.Join(fixturesDir, "key.bin"),
		"nonce": filepath.Join(fixturesDir, "nonce.bin"),
		"ad":    filepath.Join(fixturesDir, "ad.bin"),
		"pt":    filepath.Join(fixturesDir, "pt.bin"),
		"ct":    filepath.Join(fixturesDir, "ct.bin"),
		"tag":   filepath.Join(fixturesDir, "tag.bin"),
	}

	data := make(map[string][]byte)
	for name, path := range files {
		content, err := os.ReadFile(path)
		if err != nil {
			return fmt.Errorf("failed to read %s: %v", path, err)
		}
		data[name] = content
	}

	// Verify encryption
	ct, tag, err := Seal(data["key"], data["nonce"], data["ad"], data["pt"])
	if err != nil {
		return fmt.Errorf("seal failed: %v", err)
	}

	// Check if ciphertext matches
	if !bytesEqual(ct, data["ct"]) {
		return fmt.Errorf("ciphertext mismatch")
	}

	if !bytesEqual(tag, data["tag"]) {
		return fmt.Errorf("tag mismatch")
	}

	// Verify decryption
	pt, err := Open(data["key"], data["nonce"], data["ad"], data["ct"], data["tag"])
	if err != nil {
		return fmt.Errorf("open failed: %v", err)
	}

	if !bytesEqual(pt, data["pt"]) {
		return fmt.Errorf("plaintext mismatch")
	}

	fmt.Println("All fixtures passed validation")
	return nil
}

// bytesEqual compares two byte slices
func bytesEqual(a, b []byte) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

// randomTest runs a random test
func randomTest() error {
	key := make([]byte, C.AETH_KEY_SIZE)
	nonce := make([]byte, C.AETH_NONCE_SIZE)
	pt := make([]byte, 1024)

	// Fill with random data
	if _, err := io.ReadFull(rand.Reader, key); err != nil {
		return fmt.Errorf("failed to generate random key: %v", err)
	}
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return fmt.Errorf("failed to generate random nonce: %v", err)
	}
	if _, err := io.ReadFull(rand.Reader, pt); err != nil {
		return fmt.Errorf("failed to generate random plaintext: %v", err)
	}

	ad := []byte("random test associated data")

	// Encrypt
	ct, tag, err := Seal(key, nonce, ad, pt)
	if err != nil {
		return fmt.Errorf("seal failed: %v", err)
	}

	// Decrypt
	pt2, err := Open(key, nonce, ad, ct, tag)
	if err != nil {
		return fmt.Errorf("open failed: %v", err)
	}

	if !bytesEqual(pt, pt2) {
		return fmt.Errorf("roundtrip failed")
	}

	fmt.Println("Random test passed")
	return nil
}

func main() {
	var (
		generate = flag.Bool("generate", false, "Generate test vectors")
		fixtures = flag.String("fixtures", "", "Run fixtures test with directory path")
		random   = flag.Bool("random", false, "Run random test")
	)
	flag.Parse()

	if *generate {
		if err := generateTestVectors(); err != nil {
			log.Fatalf("Failed to generate test vectors: %v", err)
		}
		return
	}

	if *fixtures != "" {
		if err := runFixtures(*fixtures); err != nil {
			log.Fatalf("Fixtures test failed: %v", err)
		}
		return
	}

	if *random {
		if err := randomTest(); err != nil {
			log.Fatalf("Random test failed: %v", err)
		}
		return
	}

	// Default: run basic test
	fmt.Println("Running basic encryption test...")
	if err := randomTest(); err != nil {
		log.Fatalf("Basic test failed: %v", err)
	}
	fmt.Println("Basic test passed")
}
