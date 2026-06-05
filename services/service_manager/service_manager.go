// Package servicemanager provides service lifecycle management for Polymera OS
//
// Implements:
// - Service spawning with zero capabilities (17.2)
// - Crash detection and recovery (17.3)
// - Idle service suspension (17.5)
package servicemanager

import (
	"fmt"
	"sync"
	"time"
)

// ServiceID is a unique identifier for a service
type ServiceID uint64

// ServiceState represents the state of a service
type ServiceState int

const (
	StateCreated ServiceState = iota
	StateRunning
	StateSuspended
	StateCrashed
	StateStopped
)

// Service represents a managed service
type Service struct {
	ID           ServiceID
	Name         string
	State        ServiceState
	PID          int
	RestartCount int
	LastActivity time.Time
	Capabilities []string // Empty for zero-capability spawn
}

// ServiceManager manages service lifecycle
type ServiceManager struct {
	services    map[ServiceID]*Service
	nextID      ServiceID
	mu          sync.RWMutex
	idleTimeout time.Duration
	maxRestarts int
}

// NewServiceManager creates a new service manager
func NewServiceManager() *ServiceManager {
	return &ServiceManager{
		services:    make(map[ServiceID]*Service),
		nextID:      1,
		idleTimeout: 5 * time.Minute,
		maxRestarts: 3,
	}
}

// SpawnService creates a new service with zero capabilities (17.2)
func (sm *ServiceManager) SpawnService(name string) (ServiceID, error) {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	id := sm.nextID
	sm.nextID++

	service := &Service{
		ID:           id,
		Name:         name,
		State:        StateCreated,
		PID:          0, // Would be actual PID from fork
		RestartCount: 0,
		LastActivity: time.Now(),
		Capabilities: []string{}, // Zero capabilities - security requirement
	}

	sm.services[id] = service

	// Start the service
	service.State = StateRunning
	fmt.Printf("[SVCMGR] Spawned service %s (ID: %d) with zero capabilities\n", name, id)

	return id, nil
}

// HandleCrash handles service crash with recovery (17.3)
func (sm *ServiceManager) HandleCrash(id ServiceID) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	service, exists := sm.services[id]
	if !exists {
		return fmt.Errorf("service %d not found", id)
	}

	service.State = StateCrashed
	service.RestartCount++

	fmt.Printf("[SVCMGR] Service %s crashed (restart count: %d)\n",
		service.Name, service.RestartCount)

	// Attempt restart if under limit
	if service.RestartCount <= sm.maxRestarts {
		return sm.restartService(service)
	}

	fmt.Printf("[SVCMGR] Service %s exceeded max restarts, not restarting\n", service.Name)
	return nil
}

// restartService restarts a crashed service
func (sm *ServiceManager) restartService(service *Service) error {
	fmt.Printf("[SVCMGR] Restarting service %s\n", service.Name)
	service.State = StateRunning
	service.LastActivity = time.Now()
	return nil
}

// SuspendIdleServices suspends services that have been idle (17.5)
func (sm *ServiceManager) SuspendIdleServices() {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	now := time.Now()
	for _, service := range sm.services {
		if service.State == StateRunning {
			if now.Sub(service.LastActivity) > sm.idleTimeout {
				service.State = StateSuspended
				fmt.Printf("[SVCMGR] Suspended idle service %s\n", service.Name)
			}
		}
	}
}

// ResumeService resumes a suspended service
func (sm *ServiceManager) ResumeService(id ServiceID) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	service, exists := sm.services[id]
	if !exists {
		return fmt.Errorf("service %d not found", id)
	}

	if service.State == StateSuspended {
		service.State = StateRunning
		service.LastActivity = time.Now()
		fmt.Printf("[SVCMGR] Resumed service %s\n", service.Name)
	}

	return nil
}

// UpdateActivity marks a service as active
func (sm *ServiceManager) UpdateActivity(id ServiceID) {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	if service, exists := sm.services[id]; exists {
		service.LastActivity = time.Now()
	}
}

// StopService stops a service
func (sm *ServiceManager) StopService(id ServiceID) error {
	sm.mu.Lock()
	defer sm.mu.Unlock()

	service, exists := sm.services[id]
	if !exists {
		return fmt.Errorf("service %d not found", id)
	}

	service.State = StateStopped
	fmt.Printf("[SVCMGR] Stopped service %s\n", service.Name)
	return nil
}

// ListServices returns all services
func (sm *ServiceManager) ListServices() []*Service {
	sm.mu.RLock()
	defer sm.mu.RUnlock()

	services := make([]*Service, 0, len(sm.services))
	for _, s := range sm.services {
		services = append(services, s)
	}
	return services
}
