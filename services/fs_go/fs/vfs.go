package fs

import (
	"fmt"
	"sync"
)

// Inode represents a file object
type Inode struct {
	ID   uint64
	Name string
	Type FileType
	Size int64
}

type FileType int

const (
	FileTypeRegular FileType = iota
	FileTypeDirectory
)

// VFS implements the Virtual Filesystem
type VFS struct {
	inodes map[uint64]*Inode
	mu     sync.RWMutex
	nextID uint64
}

// NewVFS creates a new VFS instance
func NewVFS() *VFS {
	return &VFS{
		inodes: make(map[uint64]*Inode),
		nextID: 1,
	}
}

// Create creates a new file
func (v *VFS) Create(name string, ftype FileType) (*Inode, error) {
	v.mu.Lock()
	defer v.mu.Unlock()

	id := v.nextID
	v.nextID++

	inode := &Inode{
		ID:   id,
		Name: name,
		Type: ftype,
		Size: 0,
	}

	v.inodes[id] = inode
	fmt.Printf("[VFS] Created file %s (ID: %d)\n", name, id)
	return inode, nil
}
