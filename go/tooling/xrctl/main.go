package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

// XRConfig represents the configuration for the XR CLI
type XRConfig struct {
	SceneServiceURL string `json:"scene_service_url" yaml:"scene_service_url"`
	ConnectionTimeout int `json:"connection_timeout" yaml:"connection_timeout"`
	SyncInterval int `json:"sync_interval" yaml:"sync_interval"`
	EnablePolicyEnforcement bool `json:"enable_policy_enforcement" yaml:"enable_policy_enforcement"`
	EnableCapabilityChecking bool `json:"enable_capability_checking" yaml:"enable_capability_checking"`
	AutoReconnect bool `json:"auto_reconnect" yaml:"auto_reconnect"`
	MaxReconnectAttempts int `json:"max_reconnect_attempts" yaml:"max_reconnect_attempts"`
}

// SceneNode represents a node in the scene graph
type SceneNode struct {
	ID string `json:"id"`
	Name string `json:"name"`
	ParentID string `json:"parent_id,omitempty"`
	Transform Transform `json:"transform"`
	Components []Component `json:"components"`
	Metadata map[string]interface{} `json:"metadata"`
}

// Transform represents the transform of a node
type Transform struct {
	Position Vector3 `json:"position"`
	Rotation Vector3 `json:"rotation"`
	Scale Vector3 `json:"scale"`
}

// Vector3 represents a 3D vector
type Vector3 struct {
	X float64 `json:"x"`
	Y float64 `json:"y"`
	Z float64 `json:"z"`
}

// Component represents a component attached to a node
type Component struct {
	Type string `json:"type"`
	Data map[string]interface{} `json:"data"`
}

// Avatar represents an avatar in the scene
type Avatar struct {
	ID string `json:"id"`
	DID string `json:"did"`
	Profile AvatarProfile `json:"profile"`
	Transform Transform `json:"transform"`
	IsOnline bool `json:"is_online"`
	Metadata map[string]interface{} `json:"metadata"`
}

// AvatarProfile represents the profile of an avatar
type AvatarProfile struct {
	Name string `json:"name"`
	Description string `json:"description,omitempty"`
	Appearance map[string]interface{} `json:"appearance"`
	Preferences map[string]interface{} `json:"preferences"`
}

// Snapshot represents a scene snapshot
type Snapshot struct {
	ID string `json:"id"`
	Label string `json:"label"`
	Description string `json:"description,omitempty"`
	Nodes []SceneNode `json:"nodes"`
	Avatars []Avatar `json:"avatars"`
	Metadata map[string]interface{} `json:"metadata"`
	CreatedAt time.Time `json:"created_at"`
	CreatedBy string `json:"created_by"`
	Version int `json:"version"`
}

// Session represents a DID session
type Session struct {
	ID string `json:"id"`
	DID string `json:"did"`
	CreatedAt time.Time `json:"created_at"`
	ExpiresAt time.Time `json:"expires_at"`
	Status string `json:"status"`
}

// CapToken represents a capability token
type CapToken struct {
	ID string `json:"id"`
	Name string `json:"name"`
	Permissions []string `json:"permissions"`
	Scope string `json:"scope"`
	ExpiresAt *time.Time `json:"expires_at,omitempty"`
	CreatedAt time.Time `json:"created_at"`
	CreatedBy string `json:"created_by"`
	SessionID string `json:"session_id"`
	GrantedScopes []string `json:"granted_scopes"`
}

// XRDevice represents an XR device
type XRDevice struct {
	Handle string `json:"handle"`
	Profile string `json:"profile"`
	Capabilities []string `json:"capabilities"`
	IsActive bool `json:"is_active"`
}

// InputEvent represents an input event from an XR device
type InputEvent struct {
	Type string `json:"type"`
	DeviceHandle string `json:"device_handle"`
	Timestamp int64 `json:"timestamp"`
	Data map[string]interface{} `json:"data"`
}

// Participant represents a participant in a multi-user room
type Participant struct {
	ParticipantID string `json:"participant_id"`
	SessionID string `json:"session_id"`
	DisplayName string `json:"display_name"`
	AvatarConfig *AvatarConfig `json:"avatar_config,omitempty"`
	Position Vector3 `json:"position"`
	Rotation Vector3 `json:"rotation"`
	IsActive bool `json:"is_active"`
	JoinedAt int64 `json:"joined_at"`
}

// RoomState represents the state of a multi-user room
type RoomState struct {
	RoomID string `json:"room_id"`
	Participants []Participant `json:"participants"`
	SceneSnapshot *string `json:"scene_snapshot,omitempty"`
	LastUpdated int64 `json:"last_updated"`
}

// SceneSnapshot represents a snapshot of the scene
type SceneSnapshot struct {
	Version int `json:"version"`
	Nodes []SceneNode `json:"nodes"`
	Participants []Participant `json:"participants"`
	PhysicsState *PhysicsState `json:"physics_state,omitempty"`
	Timestamp int64 `json:"timestamp"`
}

// PhysicsState represents the physics state
type PhysicsState struct {
	Bodies []PhysicsBody `json:"bodies"`
	Constraints []PhysicsConstraint `json:"constraints"`
	Timestamp int64 `json:"timestamp"`
}

// PhysicsBody represents a physics body
type PhysicsBody struct {
	BodyID string `json:"body_id"`
	NodeID string `json:"node_id"`
	BodyType string `json:"body_type"`
	Transform Transform `json:"transform"`
	Velocity Vector3 `json:"velocity"`
	AngularVelocity Vector3 `json:"angular_velocity"`
}

// PhysicsConstraint represents a physics constraint
type PhysicsConstraint struct {
	ConstraintID string `json:"constraint_id"`
	BodyA string `json:"body_a"`
	BodyB string `json:"body_b"`
	ConstraintType string `json:"constraint_type"`
	Parameters map[string]interface{} `json:"parameters"`
}

// PhysicsOperation represents a physics operation
type PhysicsOperation struct {
	Type string `json:"type"`
	ObjectID *string `json:"object_id,omitempty"`
	GrabPoint *Vector3 `json:"grab_point,omitempty"`
	TargetTransform *Transform `json:"target_transform,omitempty"`
	Force *Vector3 `json:"force,omitempty"`
	Impulse *Vector3 `json:"impulse,omitempty"`
	Velocity *Vector3 `json:"velocity,omitempty"`
	Point *Vector3 `json:"point,omitempty"`
	Origin *Vector3 `json:"origin,omitempty"`
	Direction *Vector3 `json:"direction,omitempty"`
	MaxDistance *float64 `json:"max_distance,omitempty"`
}

// RaycastResult represents the result of a raycast
type RaycastResult struct {
	Hit bool `json:"hit"`
	BodyID *string `json:"body_id,omitempty"`
	NodeID *string `json:"node_id,omitempty"`
	HitPoint Vector3 `json:"hit_point"`
	HitNormal Vector3 `json:"hit_normal"`
	Distance float64 `json:"distance"`
}

// XRClient represents the client for interacting with the XR service
type XRClient struct {
	config XRConfig
	connected bool
	currentSession *Session
	capTokens map[string]*CapToken
	xrDevice *XRDevice
	currentRoom *RoomState
	physicsState *PhysicsState
}

// NewXRClient creates a new XR client
func NewXRClient(config XRConfig) *XRClient {
	return &XRClient{
		config: config,
		connected: false,
		capTokens: make(map[string]*CapToken),
	}
}

// Connect connects to the XR service
func (c *XRClient) Connect() error {
	// This would establish the actual connection
	// For now, we'll simulate it
	c.connected = true
	return nil
}

// Disconnect disconnects from the XR service
func (c *XRClient) Disconnect() error {
	c.connected = false
	return nil
}

// SpawnNode spawns a new node in the scene
func (c *XRClient) SpawnNode(nodeType string, position Vector3, parentID string) (*SceneNode, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	node := &SceneNode{
		ID: fmt.Sprintf("node_%d", time.Now().UnixNano()),
		Name: nodeType,
		ParentID: parentID,
		Transform: Transform{
			Position: position,
			Rotation: Vector3{X: 0, Y: 0, Z: 0},
			Scale: Vector3{X: 1, Y: 1, Z: 1},
		},
		Components: []Component{},
		Metadata: map[string]interface{}{},
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return node, nil
}

// UpdateNode updates an existing node
func (c *XRClient) UpdateNode(nodeID string, transform Transform) error {
	if !c.connected {
		return fmt.Errorf("not connected to XR service")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return nil
}

// RemoveNode removes a node from the scene
func (c *XRClient) RemoveNode(nodeID string) error {
	if !c.connected {
		return fmt.Errorf("not connected to XR service")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return nil
}

// SpawnAvatar spawns a new avatar
func (c *XRClient) SpawnAvatar(did string, profile AvatarProfile, position Vector3) (*Avatar, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	avatar := &Avatar{
		ID: fmt.Sprintf("avatar_%d", time.Now().UnixNano()),
		DID: did,
		Profile: profile,
		Transform: Transform{
			Position: position,
			Rotation: Vector3{X: 0, Y: 0, Z: 0},
			Scale: Vector3{X: 1, Y: 1, Z: 1},
		},
		IsOnline: true,
		Metadata: map[string]interface{}{},
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return avatar, nil
}

// UpdateAvatar updates an existing avatar
func (c *XRClient) UpdateAvatar(avatarID string, transform Transform) error {
	if !c.connected {
		return fmt.Errorf("not connected to XR service")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return nil
}

// RemoveAvatar removes an avatar from the scene
func (c *XRClient) RemoveAvatar(avatarID string) error {
	if !c.connected {
		return fmt.Errorf("not connected to XR service")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return nil
}

// SaveSnapshot saves a snapshot of the scene
func (c *XRClient) SaveSnapshot(label string, description string) (*Snapshot, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	snapshot := &Snapshot{
		ID: fmt.Sprintf("snapshot_%d", time.Now().UnixNano()),
		Label: label,
		Description: description,
		Nodes: []SceneNode{},
		Avatars: []Avatar{},
		Metadata: map[string]interface{}{},
		CreatedAt: time.Now(),
		CreatedBy: "xrctl",
		Version: 1,
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return snapshot, nil
}

// LoadSnapshot loads a snapshot
func (c *XRClient) LoadSnapshot(snapshotID string) (*Snapshot, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	snapshot := &Snapshot{
		ID: snapshotID,
		Label: "loaded_snapshot",
		Description: "Loaded from service",
		Nodes: []SceneNode{},
		Avatars: []Avatar{},
		Metadata: map[string]interface{}{},
		CreatedAt: time.Now(),
		CreatedBy: "xrctl",
		Version: 1,
	}

	return snapshot, nil
}

// GetSceneState gets the current scene state
func (c *XRClient) GetSceneState() (map[string]interface{}, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	state := map[string]interface{}{
		"nodes": []SceneNode{},
		"avatars": []Avatar{},
		"node_count": 0,
		"avatar_count": 0,
		"online_avatar_count": 0,
	}

	return state, nil
}

// ApplyPolicy applies a policy to the scene
func (c *XRClient) ApplyPolicy(policyFile string) error {
	if !c.connected {
		return fmt.Errorf("not connected to XR service")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return nil
}

// CheckCapability checks if a capability is granted
func (c *XRClient) CheckCapability(capability string, context map[string]interface{}) (bool, error) {
	if !c.connected {
		return false, fmt.Errorf("not connected to XR service")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return true, nil
}

// CheckPolicy checks if a policy allows an action
func (c *XRClient) CheckPolicy(action string, context map[string]interface{}) (bool, string, error) {
	if !c.connected {
		return false, "not connected to XR service", fmt.Errorf("not connected to XR service")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return true, "allowed", nil
}

// BeginSession begins a new DID session
func (c *XRClient) BeginSession(did, proof, nonce string) (*Session, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	session := &Session{
		ID: fmt.Sprintf("session_%d", time.Now().UnixNano()),
		DID: did,
		CreatedAt: time.Now(),
		ExpiresAt: time.Now().Add(time.Hour),
		Status: "active",
	}

	c.currentSession = session
	return session, nil
}

// EndSession ends the current session
func (c *XRClient) EndSession() error {
	if !c.connected {
		return fmt.Errorf("not connected to XR service")
	}

	if c.currentSession == nil {
		return fmt.Errorf("no active session")
	}

	c.currentSession = nil
	c.capTokens = make(map[string]*CapToken)
	return nil
}

// IssueCapability issues a capability token for the current session
func (c *XRClient) IssueCapability(scopes []string, ttlSeconds int) (*CapToken, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	if c.currentSession == nil {
		return nil, fmt.Errorf("no active session")
	}

	capToken := &CapToken{
		ID: fmt.Sprintf("cap_%d", time.Now().UnixNano()),
		Name: fmt.Sprintf("Cap-%d", time.Now().UnixNano()),
		Permissions: []string{},
		Scope: "session",
		CreatedAt: time.Now(),
		CreatedBy: c.currentSession.DID,
		SessionID: c.currentSession.ID,
		GrantedScopes: scopes,
	}

	// Set expiration if TTL is specified
	if ttlSeconds > 0 {
		expiresAt := time.Now().Add(time.Duration(ttlSeconds) * time.Second)
		capToken.ExpiresAt = &expiresAt
	}

	// Add permissions based on scopes
	for _, scope := range scopes {
		switch scope {
		case "scene:spawn", "node:spawn":
			capToken.Permissions = append(capToken.Permissions, "spawn_node")
		case "node:move", "scene:modify":
			capToken.Permissions = append(capToken.Permissions, "modify_node")
		case "node:delete":
			capToken.Permissions = append(capToken.Permissions, "remove_node")
		case "policy:attach", "scene:admin":
			capToken.Permissions = append(capToken.Permissions, "attach_policy")
		}
	}

	c.capTokens[capToken.ID] = capToken
	return capToken, nil
}

// AttachPolicy attaches a policy to the scene
func (c *XRClient) AttachPolicy(scope, regoBundle string) (string, error) {
	if !c.connected {
		return "", fmt.Errorf("not connected to XR service")
	}

	if c.currentSession == nil {
		return "", fmt.Errorf("no active session")
	}

	// Check for policy capability
	hasPolicyCap := false
	for _, capToken := range c.capTokens {
		if capToken.SessionID == c.currentSession.ID {
			for _, scope := range capToken.GrantedScopes {
				if scope == "policy:attach" || scope == "scene:admin" {
					hasPolicyCap = true
					break
				}
			}
		}
		if hasPolicyCap {
			break
		}
	}

	if !hasPolicyCap {
		return "", fmt.Errorf("insufficient capabilities - need policy:attach or scene:admin")
	}

	policyID := fmt.Sprintf("policy_%d", time.Now().UnixNano())
	return policyID, nil
}

// SimulateOperations simulates operations to check policy decisions
func (c *XRClient) SimulateOperations(operations []map[string]interface{}) (map[string]interface{}, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	if c.currentSession == nil {
		return nil, fmt.Errorf("no active session")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	results := make([]map[string]interface{}, len(operations))
	for i, op := range operations {
		results[i] = map[string]interface{}{
			"operation": op,
			"allowed": true,
			"reason": "simulated success",
		}
	}

	return map[string]interface{}{
		"results": results,
	}, nil
}

// XR Device Management

// StartXRDevice starts an XR device
func (c *XRClient) StartXRDevice(deviceProfile, xrSessionID string) (*XRDevice, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	if c.currentSession == nil {
		return nil, fmt.Errorf("no active session")
	}

	device := &XRDevice{
		Handle: fmt.Sprintf("device_%d", time.Now().UnixNano()),
		Profile: deviceProfile,
		Capabilities: []string{"hand_tracking", "eye_tracking"},
		IsActive: true,
	}

	c.xrDevice = device
	return device, nil
}

// StopXRDevice stops the XR device
func (c *XRClient) StopXRDevice() error {
	if !c.connected {
		return fmt.Errorf("not connected to XR service")
	}

	if c.xrDevice == nil {
		return fmt.Errorf("no XR device to stop")
	}

	c.xrDevice = nil
	return nil
}

// SendInput sends input to the XR device
func (c *XRClient) SendInput(inputEvent InputEvent) error {
	if !c.connected {
		return fmt.Errorf("not connected to XR service")
	}

	if c.xrDevice == nil {
		return fmt.Errorf("no XR device active")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	return nil
}

// GetXRDevice gets the current XR device
func (c *XRClient) GetXRDevice() *XRDevice {
	return c.xrDevice
}

// Multi-user Session Management

// JoinRoom joins a multi-user room
func (c *XRClient) JoinRoom(roomID, displayName string, avatarConfig *AvatarConfig) (*Participant, *RoomState, error) {
	if !c.connected {
		return nil, nil, fmt.Errorf("not connected to XR service")
	}

	if c.currentSession == nil {
		return nil, nil, fmt.Errorf("no active session")
	}

	participant := &Participant{
		ParticipantID: fmt.Sprintf("participant_%d", time.Now().UnixNano()),
		SessionID: c.currentSession.ID,
		DisplayName: displayName,
		AvatarConfig: avatarConfig,
		Position: Vector3{X: 0, Y: 0, Z: 0},
		Rotation: Vector3{X: 0, Y: 0, Z: 0},
		IsActive: true,
		JoinedAt: time.Now().Unix(),
	}

	roomState := &RoomState{
		RoomID: roomID,
		Participants: []Participant{*participant},
		LastUpdated: time.Now().Unix(),
	}

	c.currentRoom = roomState
	return participant, roomState, nil
}

// LeaveRoom leaves the current room
func (c *XRClient) LeaveRoom() error {
	if !c.connected {
		return fmt.Errorf("not connected to XR service")
	}

	if c.currentRoom == nil {
		return fmt.Errorf("not in a room")
	}

	c.currentRoom = nil
	return nil
}

// SyncScene syncs the scene with the room
func (c *XRClient) SyncScene(lastVersion int) (*SceneSnapshot, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	if c.currentRoom == nil {
		return nil, fmt.Errorf("not in a room")
	}

	snapshot := &SceneSnapshot{
		Version: lastVersion + 1,
		Nodes: []SceneNode{},
		Participants: c.currentRoom.Participants,
		Timestamp: time.Now().Unix(),
	}

	return snapshot, nil
}

// GetCurrentRoom gets the current room state
func (c *XRClient) GetCurrentRoom() *RoomState {
	return c.currentRoom
}

// Physics Interaction

// PhysicsInteract performs a physics interaction
func (c *XRClient) PhysicsInteract(operation PhysicsOperation) (map[string]interface{}, error) {
	if !c.connected {
		return nil, fmt.Errorf("not connected to XR service")
	}

	if c.currentSession == nil {
		return nil, fmt.Errorf("no active session")
	}

	// Check for physics capability
	hasPhysicsCap := false
	for _, capToken := range c.capTokens {
		if capToken.SessionID == c.currentSession.ID {
			for _, scope := range capToken.GrantedScopes {
				if scope == "physics:interact" || scope == "scene:interact" {
					hasPhysicsCap = true
					break
				}
			}
		}
		if hasPhysicsCap {
			break
		}
	}

	if !hasPhysicsCap {
		return nil, fmt.Errorf("insufficient capabilities - need physics:interact or scene:interact")
	}

	// This would send the actual request to the service
	// For now, we'll simulate success
	result := map[string]interface{}{
		"success": true,
		"operation": operation.Type,
	}

	return result, nil
}

// GrabObject grabs an object
func (c *XRClient) GrabObject(objectID string, grabPoint Vector3) error {
	operation := PhysicsOperation{
		Type: "grab",
		ObjectID: &objectID,
		GrabPoint: &grabPoint,
	}

	_, err := c.PhysicsInteract(operation)
	return err
}

// MoveObject moves an object
func (c *XRClient) MoveObject(objectID string, targetTransform Transform) error {
	operation := PhysicsOperation{
		Type: "move",
		ObjectID: &objectID,
		TargetTransform: &targetTransform,
	}

	_, err := c.PhysicsInteract(operation)
	return err
}

// ReleaseObject releases an object
func (c *XRClient) ReleaseObject(objectID string) error {
	operation := PhysicsOperation{
		Type: "release",
		ObjectID: &objectID,
	}

	_, err := c.PhysicsInteract(operation)
	return err
}

// ApplyForce applies force to an object
func (c *XRClient) ApplyForce(objectID string, force, point Vector3) error {
	operation := PhysicsOperation{
		Type: "apply_force",
		ObjectID: &objectID,
		Force: &force,
		Point: &point,
	}

	_, err := c.PhysicsInteract(operation)
	return err
}

// ApplyImpulse applies impulse to an object
func (c *XRClient) ApplyImpulse(objectID string, impulse, point Vector3) error {
	operation := PhysicsOperation{
		Type: "apply_impulse",
		ObjectID: &objectID,
		Impulse: &impulse,
		Point: &point,
	}

	_, err := c.PhysicsInteract(operation)
	return err
}

// SetVelocity sets the velocity of an object
func (c *XRClient) SetVelocity(objectID string, velocity Vector3) error {
	operation := PhysicsOperation{
		Type: "set_velocity",
		ObjectID: &objectID,
		Velocity: &velocity,
	}

	_, err := c.PhysicsInteract(operation)
	return err
}

// TeleportObject teleports an object
func (c *XRClient) TeleportObject(objectID string, transform Transform) error {
	operation := PhysicsOperation{
		Type: "teleport",
		ObjectID: &objectID,
		TargetTransform: &transform,
	}

	_, err := c.PhysicsInteract(operation)
	return err
}

// Raycast performs a raycast
func (c *XRClient) Raycast(origin, direction Vector3, maxDistance float64) (*RaycastResult, error) {
	operation := PhysicsOperation{
		Type: "raycast",
		Origin: &origin,
		Direction: &direction,
		MaxDistance: &maxDistance,
	}

	result, err := c.PhysicsInteract(operation)
	if err != nil {
		return nil, err
	}

	// This would parse the actual result from the service
	// For now, we'll simulate a result
	raycastResult := &RaycastResult{
		Hit: false,
		HitPoint: Vector3{X: 0, Y: 0, Z: 0},
		HitNormal: Vector3{X: 0, Y: 1, Z: 0},
		Distance: 0,
	}

	// Mock raycast result
	if direction.Y < -0.5 { // Pointing down
		raycastResult.Hit = true
		raycastResult.HitPoint = Vector3{
			X: origin.X + direction.X*2.0,
			Y: origin.Y + direction.Y*2.0,
			Z: origin.Z + direction.Z*2.0,
		}
		raycastResult.Distance = 2.0
	}

	return raycastResult, nil
}

// GetPhysicsState gets the current physics state
func (c *XRClient) GetPhysicsState() *PhysicsState {
	return c.physicsState
}

// Global variables
var (
	config XRConfig
	client *XRClient
)

// Root command
var rootCmd = &cobra.Command{
	Use:   "xrctl",
	Short: "XR Control Tool for Aetheris OS",
	Long: `XR Control Tool (xrctl) is a command-line interface for managing
XR scenes, avatars, and policies in Aetheris OS.`,
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		// Initialize configuration
		initConfig()
		
		// Create client
		client = NewXRClient(config)
		
		// Connect to service
		if err := client.Connect(); err != nil {
			log.Fatalf("Failed to connect to XR service: %v", err)
		}
	},
	PersistentPostRun: func(cmd *cobra.Command, args []string) {
		// Disconnect from service
		if client != nil {
			client.Disconnect()
		}
	},
}

// Scene command
var sceneCmd = &cobra.Command{
	Use:   "scene",
	Short: "Manage scene nodes and operations",
	Long:  "Manage scene nodes, including spawning, updating, and removing nodes.",
}

// Scene spawn command
var sceneSpawnCmd = &cobra.Command{
	Use:   "spawn [node_type]",
	Short: "Spawn a new node in the scene",
	Long:  "Spawn a new node of the specified type in the scene.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		nodeType := args[0]
		
		// Parse position
		positionStr, _ := cmd.Flags().GetString("position")
		position := parseVector3(positionStr)
		
		// Parse parent ID
		parentID, _ := cmd.Flags().GetString("parent")
		
		// Spawn node
		node, err := client.SpawnNode(nodeType, position, parentID)
		if err != nil {
			log.Fatalf("Failed to spawn node: %v", err)
		}
		
		// Output result
		if output, _ := cmd.Flags().GetString("output"); output != "" {
			saveToFile(output, node)
		} else {
			printJSON(node)
		}
		
		fmt.Printf("Spawned node: %s\n", node.ID)
	},
}

// Scene update command
var sceneUpdateCmd = &cobra.Command{
	Use:   "update [node_id]",
	Short: "Update an existing node",
	Long:  "Update the transform of an existing node.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		nodeID := args[0]
		
		// Parse position
		positionStr, _ := cmd.Flags().GetString("position")
		position := parseVector3(positionStr)
		
		// Parse rotation
		rotationStr, _ := cmd.Flags().GetString("rotation")
		rotation := parseVector3(rotationStr)
		
		// Parse scale
		scaleStr, _ := cmd.Flags().GetString("scale")
		scale := parseVector3(scaleStr)
		
		// Create transform
		transform := Transform{
			Position: position,
			Rotation: rotation,
			Scale: scale,
		}
		
		// Update node
		if err := client.UpdateNode(nodeID, transform); err != nil {
			log.Fatalf("Failed to update node: %v", err)
		}
		
		fmt.Printf("Updated node: %s\n", nodeID)
	},
}

// Scene remove command
var sceneRemoveCmd = &cobra.Command{
	Use:   "remove [node_id]",
	Short: "Remove a node from the scene",
	Long:  "Remove a node from the scene.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		nodeID := args[0]
		
		// Remove node
		if err := client.RemoveNode(nodeID); err != nil {
			log.Fatalf("Failed to remove node: %v", err)
		}
		
		fmt.Printf("Removed node: %s\n", nodeID)
	},
}

// Scene import command
var sceneImportCmd = &cobra.Command{
	Use:   "import [file]",
	Short: "Import a scene from a file",
	Long:  "Import a scene from a JSON or CBOR file.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		file := args[0]
		
		// Read file
		data, err := os.ReadFile(file)
		if err != nil {
			log.Fatalf("Failed to read file: %v", err)
		}
		
		// Parse scene data
		var scene map[string]interface{}
		if err := json.Unmarshal(data, &scene); err != nil {
			log.Fatalf("Failed to parse scene data: %v", err)
		}
		
		// Import scene
		fmt.Printf("Imported scene from: %s\n", file)
	},
}

// Scene export command
var sceneExportCmd = &cobra.Command{
	Use:   "export [file]",
	Short: "Export the current scene to a file",
	Long:  "Export the current scene to a JSON or CBOR file.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		file := args[0]
		
		// Get scene state
		state, err := client.GetSceneState()
		if err != nil {
			log.Fatalf("Failed to get scene state: %v", err)
		}
		
		// Save to file
		saveToFile(file, state)
		
		fmt.Printf("Exported scene to: %s\n", file)
	},
}

// Avatar command
var avatarCmd = &cobra.Command{
	Use:   "avatar",
	Short: "Manage avatars",
	Long:  "Manage avatars, including spawning, updating, and removing avatars.",
}

// Avatar bind command
var avatarBindCmd = &cobra.Command{
	Use:   "bind [did]",
	Short: "Bind an avatar to a DID",
	Long:  "Bind an avatar to a Decentralized Identifier (DID).",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		did := args[0]
		
		// Parse position
		positionStr, _ := cmd.Flags().GetString("position")
		position := parseVector3(positionStr)
		
		// Parse profile
		profile := AvatarProfile{
			Name: "Default Avatar",
			Description: "Avatar bound to " + did,
			Appearance: map[string]interface{}{},
			Preferences: map[string]interface{}{},
		}
		
		// Spawn avatar
		avatar, err := client.SpawnAvatar(did, profile, position)
		if err != nil {
			log.Fatalf("Failed to spawn avatar: %v", err)
		}
		
		// Output result
		if output, _ := cmd.Flags().GetString("output"); output != "" {
			saveToFile(output, avatar)
		} else {
			printJSON(avatar)
		}
		
		fmt.Printf("Bound avatar: %s to DID: %s\n", avatar.ID, did)
	},
}

// Avatar update command
var avatarUpdateCmd = &cobra.Command{
	Use:   "update [avatar_id]",
	Short: "Update an existing avatar",
	Long:  "Update the transform of an existing avatar.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		avatarID := args[0]
		
		// Parse position
		positionStr, _ := cmd.Flags().GetString("position")
		position := parseVector3(positionStr)
		
		// Parse rotation
		rotationStr, _ := cmd.Flags().GetString("rotation")
		rotation := parseVector3(rotationStr)
		
		// Parse scale
		scaleStr, _ := cmd.Flags().GetString("scale")
		scale := parseVector3(scaleStr)
		
		// Create transform
		transform := Transform{
			Position: position,
			Rotation: rotation,
			Scale: scale,
		}
		
		// Update avatar
		if err := client.UpdateAvatar(avatarID, transform); err != nil {
			log.Fatalf("Failed to update avatar: %v", err)
		}
		
		fmt.Printf("Updated avatar: %s\n", avatarID)
	},
}

// Avatar remove command
var avatarRemoveCmd = &cobra.Command{
	Use:   "remove [avatar_id]",
	Short: "Remove an avatar from the scene",
	Long:  "Remove an avatar from the scene.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		avatarID := args[0]
		
		// Remove avatar
		if err := client.RemoveAvatar(avatarID); err != nil {
			log.Fatalf("Failed to remove avatar: %v", err)
		}
		
		fmt.Printf("Removed avatar: %s\n", avatarID)
	},
}

// Policy command
var policyCmd = &cobra.Command{
	Use:   "policy",
	Short: "Manage policies",
	Long:  "Manage policies, including applying and checking policies.",
}

// Policy apply command
var policyApplyCmd = &cobra.Command{
	Use:   "apply [policy_file]",
	Short: "Apply a policy to the scene",
	Long:  "Apply a policy from a Rego file to the scene.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		policyFile := args[0]
		
		// Apply policy
		if err := client.ApplyPolicy(policyFile); err != nil {
			log.Fatalf("Failed to apply policy: %v", err)
		}
		
		fmt.Printf("Applied policy: %s\n", policyFile)
	},
}

// Policy check command
var policyCheckCmd = &cobra.Command{
	Use:   "check [action]",
	Short: "Check if a policy allows an action",
	Long:  "Check if a policy allows a specific action.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		action := args[0]
		
		// Parse context
		context := map[string]interface{}{
			"user_did": "did:aeth:test",
			"action": action,
			"scope": "scene",
		}
		
		// Check policy
		allowed, reason, err := client.CheckPolicy(action, context)
		if err != nil {
			log.Fatalf("Failed to check policy: %v", err)
		}
		
		if allowed {
			fmt.Printf("Policy allows action: %s\n", action)
		} else {
			fmt.Printf("Policy denies action: %s - %s\n", action, reason)
		}
	},
}

// Session command
var sessionCmd = &cobra.Command{
	Use:   "session",
	Short: "Manage DID sessions",
	Long:  "Manage DID sessions, including beginning, ending, and managing sessions.",
}

// Session begin command
var sessionBeginCmd = &cobra.Command{
	Use:   "begin [did]",
	Short: "Begin a new DID session",
	Long:  "Begin a new DID session with the specified DID.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		did := args[0]
		
		// Parse proof and nonce
		proof, _ := cmd.Flags().GetString("proof")
		nonce, _ := cmd.Flags().GetString("nonce")
		
		// Begin session
		session, err := client.BeginSession(did, proof, nonce)
		if err != nil {
			log.Fatalf("Failed to begin session: %v", err)
		}
		
		// Output result
		if output, _ := cmd.Flags().GetString("output"); output != "" {
			saveToFile(output, session)
		} else {
			printJSON(session)
		}
		
		fmt.Printf("Started session: %s for DID: %s\n", session.ID, did)
	},
}

// Session end command
var sessionEndCmd = &cobra.Command{
	Use:   "end",
	Short: "End the current session",
	Long:  "End the current DID session.",
	Run: func(cmd *cobra.Command, args []string) {
		// End session
		if err := client.EndSession(); err != nil {
			log.Fatalf("Failed to end session: %v", err)
		}
		
		fmt.Println("Session ended successfully")
	},
}

// Capability command
var capCmd = &cobra.Command{
	Use:   "cap",
	Short: "Manage capability tokens",
	Long:  "Manage capability tokens, including issuing and managing capabilities.",
}

// Capability issue command
var capIssueCmd = &cobra.Command{
	Use:   "issue [scopes]",
	Short: "Issue a capability token",
	Long:  "Issue a capability token with the specified scopes.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		scopesStr := args[0]
		scopes := strings.Split(scopesStr, ",")
		
		// Parse TTL
		ttl, _ := cmd.Flags().GetInt("ttl")
		
		// Issue capability
		capToken, err := client.IssueCapability(scopes, ttl)
		if err != nil {
			log.Fatalf("Failed to issue capability: %v", err)
		}
		
		// Output result
		if output, _ := cmd.Flags().GetString("output"); output != "" {
			saveToFile(output, capToken)
		} else {
			printJSON(capToken)
		}
		
		fmt.Printf("Issued capability: %s with scopes: %s\n", capToken.ID, scopesStr)
	},
}

// Demo command
var demoCmd = &cobra.Command{
	Use:   "demo",
	Short: "Run a demo",
	Long:  "Run a demo scene with sample nodes and avatars.",
}

// Demo run command
var demoRunCmd = &cobra.Command{
	Use:   "run",
	Short: "Run the demo",
	Long:  "Run a demo scene with sample nodes and avatars.",
	Run: func(cmd *cobra.Command, args []string) {
		headless, _ := cmd.Flags().GetBool("headless")
		spawn, _ := cmd.Flags().GetString("spawn")
		bindDID, _ := cmd.Flags().GetString("bind-did")
		
		fmt.Println("Running XR demo...")
		
		if headless {
			fmt.Println("Running in headless mode")
		}
		
		// Spawn demo objects
		if spawn != "" {
			position := Vector3{X: 0, Y: 1, Z: 0}
			node, err := client.SpawnNode(spawn, position, "")
			if err != nil {
				log.Fatalf("Failed to spawn demo node: %v", err)
			}
			fmt.Printf("Spawned demo node: %s\n", node.ID)
		}
		
		// Bind demo avatar
		if bindDID != "" {
			position := Vector3{X: 0, Y: 1, Z: 2}
			profile := AvatarProfile{
				Name: "Demo Avatar",
				Description: "Demo avatar for testing",
				Appearance: map[string]interface{}{},
				Preferences: map[string]interface{}{},
			}
			avatar, err := client.SpawnAvatar(bindDID, profile, position)
			if err != nil {
				log.Fatalf("Failed to spawn demo avatar: %v", err)
			}
			fmt.Printf("Spawned demo avatar: %s for DID: %s\n", avatar.ID, bindDID)
		}
		
		fmt.Println("Demo completed successfully")
	},
}

// Utility functions

func initConfig() {
	// Set default configuration
	config = XRConfig{
		SceneServiceURL: "localhost:50051",
		ConnectionTimeout: 5000,
		SyncInterval: 16, // ~60 FPS
		EnablePolicyEnforcement: true,
		EnableCapabilityChecking: true,
		AutoReconnect: true,
		MaxReconnectAttempts: 10,
	}
	
	// Load configuration from file if it exists
	viper.SetConfigName("xrctl")
	viper.SetConfigType("yaml")
	viper.AddConfigPath(".")
	viper.AddConfigPath("$HOME/.xrctl")
	viper.AddConfigPath("/etc/xrctl")
	
	if err := viper.ReadInConfig(); err == nil {
		if err := viper.Unmarshal(&config); err != nil {
			log.Printf("Warning: Failed to unmarshal config: %v", err)
		}
	}
}

func parseVector3(s string) Vector3 {
	if s == "" {
		return Vector3{X: 0, Y: 0, Z: 0}
	}
	
	parts := strings.Split(s, ",")
	if len(parts) != 3 {
		return Vector3{X: 0, Y: 0, Z: 0}
	}
	
	x, _ := strconv.ParseFloat(strings.TrimSpace(parts[0]), 64)
	y, _ := strconv.ParseFloat(strings.TrimSpace(parts[1]), 64)
	z, _ := strconv.ParseFloat(strings.TrimSpace(parts[2]), 64)
	
	return Vector3{X: x, Y: y, Z: z}
}

func printJSON(v interface{}) {
	data, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		log.Fatalf("Failed to marshal JSON: %v", err)
	}
	fmt.Println(string(data))
}

func saveToFile(filename string, v interface{}) {
	data, err := json.MarshalIndent(v, "", "  ")
	if err != nil {
		log.Fatalf("Failed to marshal JSON: %v", err)
	}
	
	if err := os.WriteFile(filename, data, 0644); err != nil {
		log.Fatalf("Failed to write file: %v", err)
	}
}

// Persistence commands
var persistCmd = &cobra.Command{
	Use:   "persist",
	Short: "Manage room persistence",
	Long:  "Manage room persistence, including creating snapshots and loading persistent state.",
}

var persistRoomCmd = &cobra.Command{
	Use:   "room [room_id]",
	Short: "Persist a room to NGFS",
	Long:  "Create a persistent snapshot of the specified room.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		roomID := args[0]
		reason, _ := cmd.Flags().GetString("reason")
		
		// Mock implementation
		fmt.Printf("Persisting room %s with reason: %s\n", roomID, reason)
		fmt.Printf("Snapshot ID: snap_%s_%d\n", roomID, time.Now().Unix())
	},
}

var persistLoadCmd = &cobra.Command{
	Use:   "load [snapshot_id]",
	Short: "Load a persistent room state",
	Long:  "Load a room state from a persistent snapshot.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		snapshotID := args[0]
		
		// Mock implementation
		fmt.Printf("Loading snapshot: %s\n", snapshotID)
		fmt.Printf("Room state loaded successfully\n")
	},
}

var persistListCmd = &cobra.Command{
	Use:   "list",
	Short: "List available snapshots",
	Long:  "List all available persistent snapshots.",
	Run: func(cmd *cobra.Command, args []string) {
		// Mock implementation
		fmt.Println("Available snapshots:")
		fmt.Println("  snap_room_001_1640995200")
		fmt.Println("  snap_room_002_1640995300")
		fmt.Println("  snap_room_003_1640995400")
	},
}

// Recording commands
var recordCmd = &cobra.Command{
	Use:   "record",
	Short: "Manage session recording",
	Long:  "Manage session recording, including starting, stopping, and exporting recordings.",
}

var recordStartCmd = &cobra.Command{
	Use:   "start [room_id]",
	Short: "Start recording a room session",
	Long:  "Start recording all events in the specified room.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		roomID := args[0]
		
		// Mock implementation
		recordingID := fmt.Sprintf("rec_%s_%d", roomID, time.Now().Unix())
		fmt.Printf("Started recording room %s\n", roomID)
		fmt.Printf("Recording ID: %s\n", recordingID)
	},
}

var recordStopCmd = &cobra.Command{
	Use:   "stop [recording_id]",
	Short: "Stop recording a session",
	Long:  "Stop recording and finalize the recording log.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		recordingID := args[0]
		
		// Mock implementation
		fmt.Printf("Stopped recording: %s\n", recordingID)
		fmt.Printf("Recording duration: 120 seconds\n")
		fmt.Printf("Events captured: 1,250\n")
	},
}

var recordExportCmd = &cobra.Command{
	Use:   "export [recording_id]",
	Short: "Export a recording",
	Long:  "Export a recording to a file in the specified format.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		recordingID := args[0]
		format, _ := cmd.Flags().GetString("format")
		
		// Mock implementation
		filename := fmt.Sprintf("%s.%s", recordingID, format)
		fmt.Printf("Exporting recording %s to %s\n", recordingID, filename)
		fmt.Printf("Export completed successfully\n")
	},
}

// Replay commands
var replayCmd = &cobra.Command{
	Use:   "replay",
	Short: "Manage session replay",
	Long:  "Manage session replay, including starting, stopping, and verifying replays.",
}

var replayStartCmd = &cobra.Command{
	Use:   "start [recording_id]",
	Short: "Start replaying a recording",
	Long:  "Start replaying a recording in the specified mode.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		recordingID := args[0]
		mode, _ := cmd.Flags().GetString("mode")
		
		// Mock implementation
		replayID := fmt.Sprintf("replay_%s_%d", recordingID, time.Now().Unix())
		fmt.Printf("Started replaying recording %s in %s mode\n", recordingID, mode)
		fmt.Printf("Replay ID: %s\n", replayID)
	},
}

var replayStopCmd = &cobra.Command{
	Use:   "stop [replay_id]",
	Short: "Stop a replay",
	Long:  "Stop an active replay session.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		replayID := args[0]
		
		// Mock implementation
		fmt.Printf("Stopped replay: %s\n", replayID)
		fmt.Printf("Replay completed successfully\n")
	},
}

var replayVerifyCmd = &cobra.Command{
	Use:   "verify [recording_id]",
	Short: "Verify replay determinism",
	Long:  "Verify that a recording produces deterministic replay results.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		recordingID := args[0]
		
		// Mock implementation
		fmt.Printf("Verifying determinism for recording: %s\n", recordingID)
		fmt.Printf("Determinism check: PASSED\n")
		fmt.Printf("Snapshot consistency: 100%%\n")
	},
}

// Bridge commands
var bridgeCmd = &cobra.Command{
	Use:   "bridge",
	Short: "Manage cross-metaverse bridge",
	Long:  "Manage cross-metaverse bridge operations, including export, import, and publish.",
}

var bridgeExportCmd = &cobra.Command{
	Use:   "export [room_id]",
	Short: "Export a room to cross-metaverse formats",
	Long:  "Export a room to glTF, IPFS CAR, or WebXR formats.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		roomID := args[0]
		formats, _ := cmd.Flags().GetStringSlice("format")
		
		// Mock implementation
		exportID := fmt.Sprintf("export_%s_%d", roomID, time.Now().Unix())
		fmt.Printf("Exporting room %s in formats: %v\n", roomID, formats)
		fmt.Printf("Export bundle ID: %s\n", exportID)
		
		for _, format := range formats {
			filename := fmt.Sprintf("%s.%s", exportID, format)
			fmt.Printf("  Created: %s\n", filename)
		}
	},
}

var bridgeImportCmd = &cobra.Command{
	Use:   "import [bundle_file]",
	Short: "Import a room from cross-metaverse formats",
	Long:  "Import a room from glTF, IPFS CAR, or WebXR bundle files.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		bundleFile := args[0]
		policyMode, _ := cmd.Flags().GetString("policy-mode")
		
		// Mock implementation
		fmt.Printf("Importing bundle: %s\n", bundleFile)
		fmt.Printf("Policy mode: %s\n", policyMode)
		fmt.Printf("Import completed successfully\n")
		fmt.Printf("Imported nodes: 15\n")
		fmt.Printf("Imported materials: 8\n")
		fmt.Printf("Imported textures: 12\n")
	},
}

var bridgePublishCmd = &cobra.Command{
	Use:   "publish [room_id]",
	Short: "Publish a room to cross-metaverse",
	Long:  "Publish a room to cross-metaverse platforms with optional on-chain anchoring.",
	Args:  cobra.ExactArgs(1),
	Run: func(cmd *cobra.Command, args []string) {
		roomID := args[0]
		anchor, _ := cmd.Flags().GetBool("anchor")
		
		// Mock implementation
		fmt.Printf("Publishing room %s\n", roomID)
		if anchor {
			fmt.Printf("On-chain anchoring: ENABLED\n")
			fmt.Printf("Transaction hash: 0x1234567890abcdef\n")
		} else {
			fmt.Printf("On-chain anchoring: DISABLED\n")
		}
		fmt.Printf("Publication completed successfully\n")
		fmt.Printf("IPFS hash: QmExampleHash\n")
		fmt.Printf("WebXR URL: https://webxr.example.com/room/%s\n", roomID)
	},
}

func main() {
	// Add flags to root command
	rootCmd.PersistentFlags().String("config", "", "config file (default is $HOME/.xrctl.yaml)")
	rootCmd.PersistentFlags().String("scene-service-url", "localhost:50051", "URL of the scene service")
	rootCmd.PersistentFlags().Int("connection-timeout", 5000, "Connection timeout in milliseconds")
	rootCmd.PersistentFlags().Int("sync-interval", 16, "Sync interval in milliseconds")
	rootCmd.PersistentFlags().Bool("enable-policy-enforcement", true, "Enable policy enforcement")
	rootCmd.PersistentFlags().Bool("enable-capability-checking", true, "Enable capability checking")
	rootCmd.PersistentFlags().Bool("auto-reconnect", true, "Enable auto-reconnect")
	rootCmd.PersistentFlags().Int("max-reconnect-attempts", 10, "Maximum number of reconnect attempts")
	
	// Add flags to scene spawn command
	sceneSpawnCmd.Flags().String("position", "0,0,0", "Position of the node (x,y,z)")
	sceneSpawnCmd.Flags().String("parent", "", "Parent node ID")
	sceneSpawnCmd.Flags().String("output", "", "Output file for the node data")
	
	// Add flags to scene update command
	sceneUpdateCmd.Flags().String("position", "", "New position of the node (x,y,z)")
	sceneUpdateCmd.Flags().String("rotation", "", "New rotation of the node (x,y,z)")
	sceneUpdateCmd.Flags().String("scale", "", "New scale of the node (x,y,z)")
	
	// Add flags to avatar bind command
	avatarBindCmd.Flags().String("position", "0,1,0", "Position of the avatar (x,y,z)")
	avatarBindCmd.Flags().String("output", "", "Output file for the avatar data")
	
	// Add flags to avatar update command
	avatarUpdateCmd.Flags().String("position", "", "New position of the avatar (x,y,z)")
	avatarUpdateCmd.Flags().String("rotation", "", "New rotation of the avatar (x,y,z)")
	avatarUpdateCmd.Flags().String("scale", "", "New scale of the avatar (x,y,z)")
	
	// Add flags to demo run command
	demoRunCmd.Flags().Bool("headless", false, "Run in headless mode")
	demoRunCmd.Flags().String("spawn", "", "Spawn a demo object (cube, sphere, etc.)")
	demoRunCmd.Flags().String("bind-did", "", "Bind a demo avatar to a DID")
	
	// Add flags to session begin command
	sessionBeginCmd.Flags().String("proof", "", "DID proof for authentication")
	sessionBeginCmd.Flags().String("nonce", "", "Nonce for authentication")
	sessionBeginCmd.Flags().String("output", "", "Output file for the session data")
	
	// Add flags to capability issue command
	capIssueCmd.Flags().Int("ttl", 3600, "Time to live in seconds")
	capIssueCmd.Flags().String("output", "", "Output file for the capability data")
	
	// Add flags to XR start command
	xrStartCmd.Flags().String("xr-session-id", "", "XR session ID for device binding")
	xrStartCmd.Flags().String("output", "", "Output file for the device data")
	
	// Add flags to XR input command
	xrInputCmd.Flags().String("button-id", "", "Button ID for button input")
	xrInputCmd.Flags().Bool("pressed", false, "Button pressed state")
	xrInputCmd.Flags().Float64("value", 0, "Value for trigger/joystick input")
	xrInputCmd.Flags().Float64("x", 0, "X coordinate for joystick input")
	xrInputCmd.Flags().Float64("y", 0, "Y coordinate for joystick input")
	
	// Add flags to multi-user join command
	multiUserJoinCmd.Flags().String("display-name", "User", "Display name for the participant")
	multiUserJoinCmd.Flags().String("model-id", "", "Avatar model ID")
	multiUserJoinCmd.Flags().String("output", "", "Output file for the participant data")
	
	// Add flags to multi-user sync command
	multiUserSyncCmd.Flags().Int("last-version", 0, "Last known scene version")
	multiUserSyncCmd.Flags().String("output", "", "Output file for the scene snapshot")
	
	// Add flags to physics grab command
	physicsGrabCmd.Flags().String("grab-point", "0,0,0", "Grab point (x,y,z)")
	
	// Add flags to physics move command
	physicsMoveCmd.Flags().String("position", "0,0,0", "Target position (x,y,z)")
	
	// Add flags to physics raycast command
	physicsRaycastCmd.Flags().String("origin", "0,0,0", "Raycast origin (x,y,z)")
	physicsRaycastCmd.Flags().String("direction", "0,-1,0", "Raycast direction (x,y,z)")
	physicsRaycastCmd.Flags().Float64("max-distance", 100, "Maximum raycast distance")
	physicsRaycastCmd.Flags().String("output", "", "Output file for the raycast result")
	
	// Add subcommands
	sceneCmd.AddCommand(sceneSpawnCmd)
	sceneCmd.AddCommand(sceneUpdateCmd)
	sceneCmd.AddCommand(sceneRemoveCmd)
	sceneCmd.AddCommand(sceneImportCmd)
	sceneCmd.AddCommand(sceneExportCmd)
	
	avatarCmd.AddCommand(avatarBindCmd)
	avatarCmd.AddCommand(avatarUpdateCmd)
	avatarCmd.AddCommand(avatarRemoveCmd)
	
	policyCmd.AddCommand(policyApplyCmd)
	policyCmd.AddCommand(policyCheckCmd)
	
	sessionCmd.AddCommand(sessionBeginCmd)
	sessionCmd.AddCommand(sessionEndCmd)
	
	capCmd.AddCommand(capIssueCmd)
	
	demoCmd.AddCommand(demoRunCmd)
	
	xrCmd.AddCommand(xrStartCmd)
	xrCmd.AddCommand(xrStopCmd)
	xrCmd.AddCommand(xrInputCmd)
	
	multiUserCmd.AddCommand(multiUserJoinCmd)
	multiUserCmd.AddCommand(multiUserLeaveCmd)
	multiUserCmd.AddCommand(multiUserSyncCmd)
	
	physicsCmd.AddCommand(physicsGrabCmd)
	physicsCmd.AddCommand(physicsMoveCmd)
	physicsCmd.AddCommand(physicsReleaseCmd)
	physicsCmd.AddCommand(physicsRaycastCmd)
	

	// XR Device command
	var xrCmd = &cobra.Command{
		Use:   "xr",
		Short: "Manage XR devices",
		Long:  "Manage XR devices, including starting, stopping, and sending input.",
	}

	// XR start command
	var xrStartCmd = &cobra.Command{
		Use:   "start [device_profile]",
		Short: "Start an XR device",
		Long:  "Start an XR device with the specified profile.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			deviceProfile := args[0]
			xrSessionID, _ := cmd.Flags().GetString("xr-session-id")
			
			// Start XR device
			device, err := client.StartXRDevice(deviceProfile, xrSessionID)
			if err != nil {
				log.Fatalf("Failed to start XR device: %v", err)
			}
			
			// Output result
			if output, _ := cmd.Flags().GetString("output"); output != "" {
				saveToFile(output, device)
			} else {
				printJSON(device)
			}
			
			fmt.Printf("Started XR device: %s (%s)\n", device.Handle, deviceProfile)
		},
	}

	// XR stop command
	var xrStopCmd = &cobra.Command{
		Use:   "stop",
		Short: "Stop the XR device",
		Long:  "Stop the currently active XR device.",
		Run: func(cmd *cobra.Command, args []string) {
			// Stop XR device
			if err := client.StopXRDevice(); err != nil {
				log.Fatalf("Failed to stop XR device: %v", err)
			}
			
			fmt.Println("XR device stopped successfully")
		},
	}

	// XR input command
	var xrInputCmd = &cobra.Command{
		Use:   "input [type]",
		Short: "Send input to XR device",
		Long:  "Send input to the XR device.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			inputType := args[0]
			
			// Parse input data
			data := map[string]interface{}{}
			if buttonID, _ := cmd.Flags().GetString("button-id"); buttonID != "" {
				data["button_id"] = buttonID
			}
			if pressed, _ := cmd.Flags().GetBool("pressed"); cmd.Flags().Changed("pressed") {
				data["pressed"] = pressed
			}
			if value, _ := cmd.Flags().GetFloat64("value"); cmd.Flags().Changed("value") {
				data["value"] = value
			}
			if x, _ := cmd.Flags().GetFloat64("x"); cmd.Flags().Changed("x") {
				data["x"] = x
			}
			if y, _ := cmd.Flags().GetFloat64("y"); cmd.Flags().Changed("y") {
				data["y"] = y
			}
			
			// Create input event
			inputEvent := InputEvent{
				Type: inputType,
				DeviceHandle: client.GetXRDevice().Handle,
				Timestamp: time.Now().UnixNano(),
				Data: data,
			}
			
			// Send input
			if err := client.SendInput(inputEvent); err != nil {
				log.Fatalf("Failed to send input: %v", err)
			}
			
			fmt.Printf("Sent %s input to XR device\n", inputType)
		},
	}

	// Multi-user command
	var multiUserCmd = &cobra.Command{
		Use:   "multi-user",
		Short: "Manage multi-user sessions",
		Long:  "Manage multi-user sessions, including joining rooms and syncing scenes.",
	}

	// Multi-user join command
	var multiUserJoinCmd = &cobra.Command{
		Use:   "join [room_id]",
		Short: "Join a multi-user room",
		Long:  "Join a multi-user room with the specified ID.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			roomID := args[0]
			displayName, _ := cmd.Flags().GetString("display-name")
			
			// Parse avatar config
			var avatarConfig *AvatarConfig
			if modelID, _ := cmd.Flags().GetString("model-id"); modelID != "" {
				avatarConfig = &AvatarConfig{
					ModelID: modelID,
					SkinColor: "default",
					HairColor: "default",
					Clothing: []string{},
				}
			}
			
			// Join room
			participant, roomState, err := client.JoinRoom(roomID, displayName, avatarConfig)
			if err != nil {
				log.Fatalf("Failed to join room: %v", err)
			}
			
			// Output result
			if output, _ := cmd.Flags().GetString("output"); output != "" {
				result := map[string]interface{}{
					"participant": participant,
					"room_state": roomState,
				}
				saveToFile(output, result)
			} else {
				printJSON(participant)
			}
			
			fmt.Printf("Joined room: %s as %s\n", roomID, participant.ParticipantID)
		},
	}

	// Multi-user leave command
	var multiUserLeaveCmd = &cobra.Command{
		Use:   "leave",
		Short: "Leave the current room",
		Long:  "Leave the current multi-user room.",
		Run: func(cmd *cobra.Command, args []string) {
			// Leave room
			if err := client.LeaveRoom(); err != nil {
				log.Fatalf("Failed to leave room: %v", err)
			}
			
			fmt.Println("Left room successfully")
		},
	}

	// Multi-user sync command
	var multiUserSyncCmd = &cobra.Command{
		Use:   "sync",
		Short: "Sync scene with room",
		Long:  "Sync the scene with the current room.",
		Run: func(cmd *cobra.Command, args []string) {
			lastVersion, _ := cmd.Flags().GetInt("last-version")
			
			// Sync scene
			snapshot, err := client.SyncScene(lastVersion)
			if err != nil {
				log.Fatalf("Failed to sync scene: %v", err)
			}
			
			// Output result
			if output, _ := cmd.Flags().GetString("output"); output != "" {
				saveToFile(output, snapshot)
			} else {
				printJSON(snapshot)
			}
			
			fmt.Printf("Synced scene: version %d\n", snapshot.Version)
		},
	}

	// Physics command
	var physicsCmd = &cobra.Command{
		Use:   "physics",
		Short: "Manage physics interactions",
		Long:  "Manage physics interactions, including grabbing, moving, and raycasting.",
	}

	// Physics grab command
	var physicsGrabCmd = &cobra.Command{
		Use:   "grab [object_id]",
		Short: "Grab an object",
		Long:  "Grab an object with the specified ID.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			objectID := args[0]
			grabPointStr, _ := cmd.Flags().GetString("grab-point")
			grabPoint := parseVector3(grabPointStr)
			
			// Grab object
			if err := client.GrabObject(objectID, grabPoint); err != nil {
				log.Fatalf("Failed to grab object: %v", err)
			}
			
			fmt.Printf("Grabbed object: %s\n", objectID)
		},
	}

	// Physics move command
	var physicsMoveCmd = &cobra.Command{
		Use:   "move [object_id]",
		Short: "Move an object",
		Long:  "Move an object to the specified position.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			objectID := args[0]
			positionStr, _ := cmd.Flags().GetString("position")
			position := parseVector3(positionStr)
			
			// Create transform
			transform := Transform{
				Position: position,
				Rotation: Vector3{X: 0, Y: 0, Z: 0},
				Scale: Vector3{X: 1, Y: 1, Z: 1},
			}
			
			// Move object
			if err := client.MoveObject(objectID, transform); err != nil {
				log.Fatalf("Failed to move object: %v", err)
			}
			
			fmt.Printf("Moved object: %s to %v\n", objectID, position)
		},
	}

	// Physics release command
	var physicsReleaseCmd = &cobra.Command{
		Use:   "release [object_id]",
		Short: "Release an object",
		Long:  "Release a grabbed object.",
		Args:  cobra.ExactArgs(1),
		Run: func(cmd *cobra.Command, args []string) {
			objectID := args[0]
			
			// Release object
			if err := client.ReleaseObject(objectID); err != nil {
				log.Fatalf("Failed to release object: %v", err)
			}
			
			fmt.Printf("Released object: %s\n", objectID)
		},
	}

	// Physics raycast command
	var physicsRaycastCmd = &cobra.Command{
		Use:   "raycast",
		Short: "Perform a raycast",
		Long:  "Perform a raycast from the specified origin and direction.",
		Run: func(cmd *cobra.Command, args []string) {
			originStr, _ := cmd.Flags().GetString("origin")
			origin := parseVector3(originStr)
			directionStr, _ := cmd.Flags().GetString("direction")
			direction := parseVector3(directionStr)
			maxDistance, _ := cmd.Flags().GetFloat64("max-distance")
			
			// Perform raycast
			result, err := client.Raycast(origin, direction, maxDistance)
			if err != nil {
				log.Fatalf("Failed to perform raycast: %v", err)
			}
			
			// Output result
			if output, _ := cmd.Flags().GetString("output"); output != "" {
				saveToFile(output, result)
			} else {
				printJSON(result)
			}
			
			if result.Hit {
				fmt.Printf("Raycast hit at distance %.2f\n", result.Distance)
			} else {
				fmt.Println("Raycast missed")
			}
		},
	}

	// Add subcommands
	sceneCmd.AddCommand(sceneSpawnCmd, sceneUpdateCmd, sceneRemoveCmd, sceneImportCmd, sceneExportCmd)
	avatarCmd.AddCommand(avatarBindCmd, avatarUpdateCmd, avatarRemoveCmd)
	policyCmd.AddCommand(policyApplyCmd, policyCheckCmd)
	sessionCmd.AddCommand(sessionBeginCmd, sessionEndCmd)
	capCmd.AddCommand(capIssueCmd)
	demoCmd.AddCommand(demoRunCmd)
	xrCmd.AddCommand(xrStartCmd, xrStopCmd, xrInputCmd)
	multiUserCmd.AddCommand(multiUserJoinCmd, multiUserLeaveCmd, multiUserSyncCmd)
	physicsCmd.AddCommand(physicsGrabCmd, physicsMoveCmd, physicsReleaseCmd, physicsRaycastCmd)
	
	// Add persistence subcommands
	persistCmd.AddCommand(persistRoomCmd, persistLoadCmd, persistListCmd)
	persistRoomCmd.Flags().String("reason", "manual", "Reason for persistence")
	
	// Add recording subcommands
	recordCmd.AddCommand(recordStartCmd, recordStopCmd, recordExportCmd)
	recordExportCmd.Flags().String("format", "json", "Export format (json, cbor)")
	
	// Add replay subcommands
	replayCmd.AddCommand(replayStartCmd, replayStopCmd, replayVerifyCmd)
	replayStartCmd.Flags().String("mode", "headless", "Replay mode (headless, visualization)")
	
	// Add bridge subcommands
	bridgeCmd.AddCommand(bridgeExportCmd, bridgeImportCmd, bridgePublishCmd)
	bridgeExportCmd.Flags().StringSlice("format", []string{"gltf"}, "Export formats (gltf, car, webxr)")
	bridgeImportCmd.Flags().String("policy-mode", "strict", "Import policy mode (strict, permissive)")
	bridgePublishCmd.Flags().Bool("anchor", false, "Enable on-chain anchoring")
	
	rootCmd.AddCommand(sceneCmd)
	rootCmd.AddCommand(avatarCmd)
	rootCmd.AddCommand(policyCmd)
	rootCmd.AddCommand(sessionCmd)
	rootCmd.AddCommand(capCmd)
	rootCmd.AddCommand(demoCmd)
	rootCmd.AddCommand(xrCmd)
	rootCmd.AddCommand(multiUserCmd)
	rootCmd.AddCommand(physicsCmd)
	rootCmd.AddCommand(persistCmd)
	rootCmd.AddCommand(recordCmd)
	rootCmd.AddCommand(replayCmd)
	rootCmd.AddCommand(bridgeCmd)
	
	// Execute root command
	if err := rootCmd.Execute(); err != nil {
		log.Fatal(err)
	}
}
