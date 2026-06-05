/**
 * XR Bridge - TypeScript SDK for Aetheris XR Scene Graph Service
 * 
 * This module provides a high-level, event-driven API for interacting with
 * the Aetheris XR scene graph service, including scene management, avatar
 * binding, capability checking, and policy enforcement.
 */

// Mock implementations for browser compatibility
class EventEmitter {
  private listeners: Map<string, Function[]> = new Map();
  
  on(event: string, listener: Function): this {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(listener);
    return this;
  }
  
  emit(event: string, ...args: any[]): boolean {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      eventListeners.forEach(listener => listener(...args));
      return true;
    }
    return false;
  }
  
  removeListener(event: string, listener: Function): this {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      const index = eventListeners.indexOf(listener);
      if (index > -1) {
        eventListeners.splice(index, 1);
      }
    }
    return this;
  }
}

// Mock UUID implementation
function uuidv4(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

// Type definitions
export interface Vector3 {
  x: number;
  y: number;
  z: number;
}

export interface Transform {
  position: Vector3;
  rotation: Vector3;
  scale: Vector3;
}

export interface Component {
  type: string;
  data: Record<string, any>;
}

export interface SceneNode {
  id: string;
  name: string;
  parent_id?: string;
  transform: Transform;
  components: Component[];
  metadata: Record<string, any>;
}

export interface AvatarProfile {
  name: string;
  description?: string;
  appearance: Record<string, any>;
  preferences: Record<string, any>;
}

export interface Avatar {
  id: string;
  did: string;
  profile: AvatarProfile;
  transform: Transform;
  is_online: boolean;
  metadata: Record<string, any>;
}

export interface CapToken {
  id: string;
  name: string;
  permissions: string[];
  scope: string;
  expires_at?: Date;
  created_at: Date;
  created_by: string;
  session_id?: string;
  granted_scopes: string[];
}

export interface PolicyContext {
  user_did: string;
  avatar_id?: string;
  node_id?: string;
  action: string;
  scope: string;
  session_id?: string;
  metadata: Record<string, any>;
}

export interface PolicyResult {
  allowed: boolean;
  reason?: string;
  conditions?: string[];
  rule_id?: string;
}

export interface Snapshot {
  id: string;
  label: string;
  description?: string;
  nodes: SceneNode[];
  avatars: Avatar[];
  metadata: Record<string, any>;
  created_at: Date;
  created_by: string;
  version: number;
}

export interface XRConfig {
  scene_service_url: string;
  connection_timeout: number;
  sync_interval: number;
  enable_policy_enforcement: boolean;
  enable_capability_checking: boolean;
  auto_reconnect: boolean;
  max_reconnect_attempts: number;
}

export interface XRStats {
  connection_status: 'connected' | 'disconnected' | 'connecting' | 'error';
  node_count: number;
  avatar_count: number;
  online_avatar_count: number;
  snapshot_count: number;
  total_requests: number;
  successful_requests: number;
  failed_requests: number;
  average_response_time: number;
  // P4-06-A4: Persistence and recording stats
  xr_device_count: number;
  room_participant_count: number;
  physics_body_count: number;
  recording_count: number;
  replay_count: number;
  export_bundle_count: number;
  import_count: number;
  publish_count: number;
}

// XR Device types
export interface XRDevice {
  handle: string;
  profile: string;
  capabilities: string[];
  is_active: boolean;
}

export interface InputEvent {
  type: 'button' | 'trigger' | 'joystick' | 'hand_tracking' | 'eye_tracking' | 'voice_command';
  device_handle: string;
  timestamp: number;
  data: any;
}

export interface HandJoint {
  joint_id: string;
  position: Vector3;
  rotation: Vector3;
  confidence: number;
}

// Multi-user types
export interface Participant {
  participant_id: string;
  session_id: string;
  display_name: string;
  avatar_config?: AvatarConfig;
  position: Vector3;
  rotation: Vector3;
  is_active: boolean;
  joined_at: number;
}

export interface RoomState {
  room_id: string;
  participants: Participant[];
  scene_snapshot?: string;
  last_updated: number;
}

export interface AvatarConfig {
  model_id: string;
  skin_color: string;
  hair_color: string;
  clothing: string[];
}

export interface SceneSnapshot {
  version: number;
  nodes: SceneNode[];
  participants: Participant[];
  physics_state?: PhysicsState;
  timestamp: number;
}

// Physics types
export interface PhysicsState {
  bodies: PhysicsBody[];
  constraints: PhysicsConstraint[];
  timestamp: number;
}

export interface PhysicsBody {
  body_id: string;
  node_id: string;
  body_type: string;
  transform: Transform;
  velocity: Vector3;
  angular_velocity: Vector3;
}

export interface PhysicsConstraint {
  constraint_id: string;
  body_a: string;
  body_b: string;
  constraint_type: string;
  parameters: Record<string, any>;
}

export interface PhysicsOperation {
  type: 'grab' | 'move' | 'release' | 'apply_force' | 'apply_impulse' | 'set_velocity' | 'teleport' | 'raycast';
  object_id?: string;
  grab_point?: Vector3;
  target_transform?: Transform;
  force?: Vector3;
  impulse?: Vector3;
  velocity?: Vector3;
  point?: Vector3;
  origin?: Vector3;
  direction?: Vector3;
  max_distance?: number;
}

export interface RaycastResult {
  hit: boolean;
  body_id?: string;
  node_id?: string;
  hit_point: Vector3;
  hit_normal: Vector3;
  distance: number;
}

// P4-06-A4: Persistence interfaces
export interface SnapshotMetadata {
  id: string;
  room_id: string;
  version: number;
  created_at: number;
  created_by: string;
  reason: string;
  dao_proposal_id?: string;
  deterministic_hash: string;
  ngfs_cid?: string;
}

export interface PersistentRoomState {
  room: RoomState;
  nodes: SceneNode[];
  avatars: Avatar[];
  physics_state: PhysicsState;
  policies: any[];
  capabilities: any[];
}

// P4-06-A4: Recording interfaces
export interface RecordingSession {
  id: string;
  room_id: string;
  started_at: number;
  started_by: string;
  session_id: string;
  is_active: boolean;
  event_count: number;
  deterministic_seed: number;
  physics_tick_rate: number;
}

export interface RecordingSummary {
  recording_id: string;
  room_id: string;
  duration_seconds: number;
  event_count: number;
  started_at: number;
  stopped_at: number;
  deterministic_hash: string;
  file_size_bytes: number;
  participants: string[];
  devices_used: string[];
}

// P4-06-A4: Replay interfaces
export enum ReplayMode {
  Headless = 'headless',
  Visualization = 'visualization',
  Debug = 'debug'
}

export interface ReplayResult {
  replay_id: string;
  recording_id: string;
  final_snapshot_id: string;
  deterministic_hash: string;
  events_processed: number;
  duration_seconds: number;
  success: boolean;
  error_message?: string;
  byte_stability_verified: boolean;
}

// P4-06-A4: Bridge interfaces
export enum ExportFormat {
  Gltf = 'gltf',
  Car = 'car',
  Json = 'json',
  All = 'all'
}

export enum ImportPolicyMode {
  Strict = 'strict',
  Permissive = 'permissive',
  Sandbox = 'sandbox'
}

export interface ExportBundle {
  id: string;
  room_id: string;
  formats: ExportFormat[];
  created_at: number;
  created_by: string;
  file_paths: Record<string, string>;
  manifest: any;
  ipfs_cids: Record<string, string>;
  total_size_bytes: number;
}

export interface ImportResult {
  import_id: string;
  room_id: string;
  success: boolean;
  error_message?: string;
  imported_nodes: number;
  imported_avatars: number;
  imported_assets: number;
  policy_violations: string[];
  warnings: string[];
}

export interface PublishResult {
  publish_id: string;
  room_id: string;
  success: boolean;
  error_message?: string;
  ipfs_cids: Record<string, string>;
  on_chain_tx_hash?: string;
  dao_proposal_id?: string;
  public_url?: string;
}

// Event types
export interface XREvents {
  'scene_ready': () => void;
  'scene_error': (error: string) => void;
  'node_spawned': (node: SceneNode) => void;
  'node_updated': (node: SceneNode) => void;
  'node_removed': (node_id: string) => void;
  'avatar_spawned': (avatar: Avatar) => void;
  'avatar_updated': (avatar: Avatar) => void;
  'avatar_removed': (avatar_id: string) => void;
  'policy_denied': (action: string, reason: string) => void;
  'snapshot_saved': (snapshot: Snapshot) => void;
  'capability_granted': (capability: string, token: CapToken) => void;
  'capability_denied': (capability: string, reason: string) => void;
  'connection_lost': () => void;
  'connection_restored': () => void;
  'session_started': (session_id: string, did: string) => void;
  'session_ended': (session_id: string) => void;
  'cap_issued': (cap_id: string, scopes: string[]) => void;
  'cap_expired': (cap_id: string) => void;
  // XR Device events
  'xr_device_started': (device: XRDevice) => void;
  'xr_device_stopped': (device_handle: string) => void;
  'input_received': (event: InputEvent) => void;
  // Multi-user events
  'room_joined': (room_id: string, participant_id: string) => void;
  'room_left': (room_id: string) => void;
  'participant_joined': (participant: Participant) => void;
  'participant_left': (participant_id: string) => void;
  'scene_synchronized': (snapshot: SceneSnapshot) => void;
  // Physics events
  'object_grabbed': (object_id: string, grab_point: Vector3) => void;
  'object_moved': (object_id: string, transform: Transform) => void;
  'object_released': (object_id: string) => void;
  'physics_interaction': (operation: string, object_id: string) => void;
  'raycast_hit': (result: RaycastResult) => void;
  // P4-06-A4: Persistence events
  'room_persisted': (room_id: string, snapshot_id: string, ngfs_cid: string) => void;
  'snapshot_loaded': (snapshot_id: string, room_id: string) => void;
  // P4-06-A4: Recording events
  'recording_started': (recording_id: string, room_id: string) => void;
  'recording_stopped': (recording_id: string, room_id: string, duration: number, event_count: number) => void;
  // P4-06-A4: Replay events
  'replay_started': (replay_id: string, recording_id: string, mode: string) => void;
  'replay_stopped': (replay_id: string) => void;
  'replay_completed': (replay_id: string, deterministic_hash: string, events_processed: number) => void;
  // P4-06-A4: Bridge events
  'scene_exported': (bundle_id: string, room_id: string, formats: string[]) => void;
  'scene_imported': (import_id: string, room_id: string, imported_nodes: number, imported_avatars: number) => void;
  'scene_published': (publish_id: string, room_id: string, anchored: boolean, public_url?: string) => void;
}

/**
 * XR Bridge - Main class for interacting with the Aetheris XR scene service
 */
export class XRBridge extends EventEmitter {
  private config: XRConfig;
  private connected: boolean = false;
  private reconnectAttempts: number = 0;
  private reconnectTimer?: number;
  private syncTimer?: number;
  private sceneServiceClient?: any; // gRPC client
  private didSession?: string;
  private currentSessionId?: string;
  private currentDid?: string;
  private capTokens: Map<string, CapToken> = new Map();
  private localNodes: Map<string, SceneNode> = new Map();
  private localAvatars: Map<string, Avatar> = new Map();
  private xrDevice?: XRDevice;
  private currentRoom?: RoomState;
  private physicsState?: PhysicsState;
  private stats: XRStats;
  // P4-06-A4: Persistence and recording state
  private currentRecordingId?: string;
  private currentReplayId?: string;
  private snapshots: Map<string, SnapshotMetadata> = new Map();
  private recordings: Map<string, RecordingSummary> = new Map();
  private replays: Map<string, ReplayResult> = new Map();
  private exportBundles: Map<string, ExportBundle> = new Map();

  constructor(config: Partial<XRConfig> = {}) {
    super();
    
    this.config = {
      scene_service_url: 'localhost:50051',
      connection_timeout: 5000,
      sync_interval: 1000 / 60, // 60 Hz
      enable_policy_enforcement: true,
      enable_capability_checking: true,
      auto_reconnect: true,
      max_reconnect_attempts: 10,
      ...config
    };

    this.stats = {
      connection_status: 'disconnected',
      node_count: 0,
      avatar_count: 0,
      online_avatar_count: 0,
      snapshot_count: 0,
      total_requests: 0,
      successful_requests: 0,
      failed_requests: 0,
      average_response_time: 0,
      // P4-06-A4: Persistence and recording stats
      xr_device_count: 0,
      room_participant_count: 0,
      physics_body_count: 0,
      recording_count: 0,
      replay_count: 0,
      export_bundle_count: 0,
      import_count: 0,
      publish_count: 0
    };

    this.setupEventHandlers();
  }

  /**
   * Connect to the scene service
   */
  async connect(): Promise<void> {
    if (this.connected) {
      return;
    }

    this.stats.connection_status = 'connecting';
    this.emit('connection_attempt');

    try {
      // Initialize gRPC client
      await this.initializeGRPCClient();
      
      // Establish connection
      await this.establishConnection();
      
      this.connected = true;
      this.reconnectAttempts = 0;
      this.stats.connection_status = 'connected';
      
      // Start sync loop
      this.startSyncLoop();
      
      this.emit('scene_ready');
      this.emit('connection_restored');
      
    } catch (error) {
      this.stats.connection_status = 'error';
      this.emit('scene_error', `Connection failed: ${error}`);
      
      if (this.config.auto_reconnect) {
        this.scheduleReconnect();
      }
      
      throw error;
    }
  }

  /**
   * Disconnect from the scene service
   */
  async disconnect(): Promise<void> {
    if (!this.connected) {
      return;
    }

    this.connected = false;
    this.stats.connection_status = 'disconnected';
    
    // Stop sync loop
    this.stopSyncLoop();
    
    // Clear reconnect timer
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = undefined;
    }
    
    // Close gRPC client
    if (this.sceneServiceClient) {
      await this.sceneServiceClient.close();
      this.sceneServiceClient = undefined;
    }
    
    this.emit('connection_lost');
  }

  /**
   * Set the DID session for authentication
   */
  setDIDSession(did: string, sessionToken: string): void {
    this.didSession = `${did}:${sessionToken}`;
  }

  /**
   * Begin a new DID session
   */
  async beginSession(did: string, proof: string, nonce: string): Promise<string> {
    try {
      const response = await this.sendRequest('begin_session', {
        did,
        proof,
        nonce
      });

      if (response.success) {
        this.currentSessionId = response.session_id;
        this.currentDid = did;
        this.emit('session_started', response.session_id, did);
        return response.session_id;
      } else {
        throw new Error(response.error || 'Failed to begin session');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * End the current DID session
   */
  async endSession(): Promise<void> {
    if (!this.currentSessionId) {
      throw new Error('No active session to end');
    }

    try {
      const response = await this.sendRequest('end_session', {
        session_id: this.currentSessionId
      });

      if (response.success) {
        this.emit('session_ended', this.currentSessionId);
        this.currentSessionId = undefined;
        this.currentDid = undefined;
        this.capTokens.clear();
      } else {
        throw new Error(response.error || 'Failed to end session');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Issue a capability token for the current session
   */
  async issueCapability(scopes: string[], ttlSeconds: number): Promise<CapToken> {
    if (!this.currentSessionId) {
      throw new Error('No active session for capability issuance');
    }

    try {
      const response = await this.sendRequest('issue_cap', {
        session_id: this.currentSessionId,
        scopes,
        ttl: ttlSeconds
      });

      if (response.success) {
        const capToken: CapToken = {
          id: response.cap_id,
          name: `Cap-${response.cap_id}`,
          permissions: response.permissions || [],
          scope: 'session',
          expires_at: response.expires_at ? new Date(response.expires_at) : undefined,
          created_at: new Date(),
          created_by: this.currentDid || '',
          session_id: this.currentSessionId,
          granted_scopes: scopes
        };

        this.capTokens.set(capToken.id, capToken);
        this.emit('cap_issued', capToken.id, scopes);
        return capToken;
      } else {
        throw new Error(response.error || 'Failed to issue capability');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Spawn a new node in the scene
   */
  async spawnNode(options: {
    type: string;
    position: Vector3;
    rotation?: Vector3;
    scale?: Vector3;
    parent_id?: string;
    components?: Component[];
    metadata?: Record<string, any>;
  }): Promise<SceneNode> {
    // Check authentication
    if (!this.currentSessionId) {
      throw new Error('No active session - cannot spawn node');
    }

    // Check for spawn capability
    const hasSpawnCap = this.hasCapability(['scene:spawn', 'node:spawn']);
    if (!hasSpawnCap) {
      this.emit('policy_denied', 'spawn_node', 'Insufficient capabilities');
      throw new Error('Insufficient capabilities - need scene:spawn or node:spawn');
    }

    const nodeId = this.generateNodeId();
    const node: SceneNode = {
      id: nodeId,
      name: options.type,
      parent_id: options.parent_id,
      transform: {
        position: options.position,
        rotation: options.rotation || { x: 0, y: 0, z: 0 },
        scale: options.scale || { x: 1, y: 1, z: 1 }
      },
      components: options.components || [],
      metadata: options.metadata || {}
    };

    // Check policies
    if (this.config.enable_policy_enforcement) {
      const policyResult = await this.checkPolicy('spawn', {
        user_did: this.currentDid || '',
        node_id: nodeId,
        action: 'spawn',
        scope: 'scene',
        session_id: this.currentSessionId,
        metadata: { node_type: options.type }
      });
      
      if (!policyResult.allowed) {
        this.emit('policy_denied', 'spawn', policyResult.reason || 'Policy denied');
        throw new Error(`Policy denied: ${policyResult.reason}`);
      }
    }

    try {
      // Send to scene service with authentication
      const response = await this.sendRequest('spawn_node', {
        session_id: this.currentSessionId,
        node_id: nodeId,
        node_type: options.type,
        transform: node.transform,
        components: node.components,
        metadata: node.metadata
      });

      if (response.success) {
        this.localNodes.set(nodeId, node);
        this.stats.node_count++;
        this.emit('node_spawned', node);
        return node;
      } else {
        throw new Error(response.error || 'Failed to spawn node');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Update an existing node
   */
  async updateNode(nodeId: string, updates: {
    position?: Vector3;
    rotation?: Vector3;
    scale?: Vector3;
    components?: Component[];
    metadata?: Record<string, any>;
  }): Promise<void> {
    const node = this.localNodes.get(nodeId);
    if (!node) {
      throw new Error(`Node not found: ${nodeId}`);
    }

    // Check capabilities
    if (this.config.enable_capability_checking) {
      const hasCapability = await this.checkCapability('modify_node', {
        node_id: nodeId,
        updates
      });
      
      if (!hasCapability) {
        throw new Error('Insufficient capabilities to modify node');
      }
    }

    // Check policies
    if (this.config.enable_policy_enforcement) {
      const policyResult = await this.checkPolicy('modify', {
        user_did: this.didSession?.split(':')[0] || '',
        node_id: nodeId,
        action: 'modify',
        scope: 'node',
        metadata: updates
      });
      
      if (!policyResult.allowed) {
        this.emit('policy_denied', 'modify', policyResult.reason || 'Policy denied');
        throw new Error(`Policy denied: ${policyResult.reason}`);
      }
    }

    try {
      // Update local node
      if (updates.position) node.transform.position = updates.position;
      if (updates.rotation) node.transform.rotation = updates.rotation;
      if (updates.scale) node.transform.scale = updates.scale;
      if (updates.components) node.components = updates.components;
      if (updates.metadata) node.metadata = { ...node.metadata, ...updates.metadata };

      // Send to scene service
      const response = await this.sendRequest('update_node', {
        node_id: nodeId,
        transform: node.transform,
        components: node.components,
        metadata: node.metadata
      });

      if (response.success) {
        this.localNodes.set(nodeId, node);
        this.emit('node_updated', node);
      } else {
        throw new Error(response.error || 'Failed to update node');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Remove a node from the scene
   */
  async removeNode(nodeId: string): Promise<void> {
    const node = this.localNodes.get(nodeId);
    if (!node) {
      throw new Error(`Node not found: ${nodeId}`);
    }

    // Check capabilities
    if (this.config.enable_capability_checking) {
      const hasCapability = await this.checkCapability('remove_node', {
        node_id: nodeId
      });
      
      if (!hasCapability) {
        throw new Error('Insufficient capabilities to remove node');
      }
    }

    // Check policies
    if (this.config.enable_policy_enforcement) {
      const policyResult = await this.checkPolicy('remove', {
        user_did: this.didSession?.split(':')[0] || '',
        node_id: nodeId,
        action: 'remove',
        scope: 'node',
        metadata: {}
      });
      
      if (!policyResult.allowed) {
        this.emit('policy_denied', 'remove', policyResult.reason || 'Policy denied');
        throw new Error(`Policy denied: ${policyResult.reason}`);
      }
    }

    try {
      // Send to scene service
      const response = await this.sendRequest('remove_node', {
        node_id: nodeId
      });

      if (response.success) {
        this.localNodes.delete(nodeId);
        this.stats.node_count--;
        this.emit('node_removed', nodeId);
      } else {
        throw new Error(response.error || 'Failed to remove node');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Spawn an avatar
   */
  async spawnAvatar(options: {
    did: string;
    profile: AvatarProfile;
    position: Vector3;
    rotation?: Vector3;
    metadata?: Record<string, any>;
  }): Promise<Avatar> {
    const avatarId = this.generateAvatarId();
    const avatar: Avatar = {
      id: avatarId,
      did: options.did,
      profile: options.profile,
      transform: {
        position: options.position,
        rotation: options.rotation || { x: 0, y: 0, z: 0 },
        scale: { x: 1, y: 1, z: 1 }
      },
      is_online: true,
      metadata: options.metadata || {}
    };

    // Check capabilities
    if (this.config.enable_capability_checking) {
      const hasCapability = await this.checkCapability('spawn_avatar', {
        did: options.did,
        position: options.position
      });
      
      if (!hasCapability) {
        throw new Error('Insufficient capabilities to spawn avatar');
      }
    }

    // Check policies
    if (this.config.enable_policy_enforcement) {
      const policyResult = await this.checkPolicy('bind_avatar', {
        user_did: options.did,
        avatar_id: avatarId,
        action: 'bind_avatar',
        scope: 'avatar',
        metadata: { profile: options.profile }
      });
      
      if (!policyResult.allowed) {
        this.emit('policy_denied', 'bind_avatar', policyResult.reason || 'Policy denied');
        throw new Error(`Policy denied: ${policyResult.reason}`);
      }
    }

    try {
      // Send to scene service
      const response = await this.sendRequest('spawn_avatar', {
        avatar_id: avatarId,
        did: options.did,
        profile: options.profile,
        transform: avatar.transform,
        metadata: avatar.metadata
      });

      if (response.success) {
        this.localAvatars.set(avatarId, avatar);
        this.stats.avatar_count++;
        this.stats.online_avatar_count++;
        this.emit('avatar_spawned', avatar);
        return avatar;
      } else {
        throw new Error(response.error || 'Failed to spawn avatar');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Update an avatar
   */
  async updateAvatar(avatarId: string, updates: {
    position?: Vector3;
    rotation?: Vector3;
    profile?: AvatarProfile;
    metadata?: Record<string, any>;
  }): Promise<void> {
    const avatar = this.localAvatars.get(avatarId);
    if (!avatar) {
      throw new Error(`Avatar not found: ${avatarId}`);
    }

    // Check capabilities
    if (this.config.enable_capability_checking) {
      const hasCapability = await this.checkCapability('modify_avatar', {
        avatar_id: avatarId,
        updates
      });
      
      if (!hasCapability) {
        throw new Error('Insufficient capabilities to modify avatar');
      }
    }

    try {
      // Update local avatar
      if (updates.position) avatar.transform.position = updates.position;
      if (updates.rotation) avatar.transform.rotation = updates.rotation;
      if (updates.profile) avatar.profile = updates.profile;
      if (updates.metadata) avatar.metadata = { ...avatar.metadata, ...updates.metadata };

      // Send to scene service
      const response = await this.sendRequest('update_avatar', {
        avatar_id: avatarId,
        transform: avatar.transform,
        profile: avatar.profile,
        metadata: avatar.metadata
      });

      if (response.success) {
        this.localAvatars.set(avatarId, avatar);
        this.emit('avatar_updated', avatar);
      } else {
        throw new Error(response.error || 'Failed to update avatar');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Remove an avatar
   */
  async removeAvatar(avatarId: string): Promise<void> {
    const avatar = this.localAvatars.get(avatarId);
    if (!avatar) {
      throw new Error(`Avatar not found: ${avatarId}`);
    }

    // Check capabilities
    if (this.config.enable_capability_checking) {
      const hasCapability = await this.checkCapability('remove_avatar', {
        avatar_id: avatarId
      });
      
      if (!hasCapability) {
        throw new Error('Insufficient capabilities to remove avatar');
      }
    }

    try {
      // Send to scene service
      const response = await this.sendRequest('remove_avatar', {
        avatar_id: avatarId
      });

      if (response.success) {
        this.localAvatars.delete(avatarId);
        this.stats.avatar_count--;
        this.stats.online_avatar_count--;
        this.emit('avatar_removed', avatarId);
      } else {
        throw new Error(response.error || 'Failed to remove avatar');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Save a snapshot
   */
  async saveSnapshot(label: string, description?: string): Promise<Snapshot> {
    // Check capabilities
    if (this.config.enable_capability_checking) {
      const hasCapability = await this.checkCapability('create_snapshot', {
        label,
        description
      });
      
      if (!hasCapability) {
        throw new Error('Insufficient capabilities to create snapshot');
      }
    }

    try {
      const response = await this.sendRequest('save_snapshot', {
        label,
        description,
        nodes: Array.from(this.localNodes.values()),
        avatars: Array.from(this.localAvatars.values())
      });

      if (response.success) {
        const snapshot: Snapshot = {
          id: response.snapshot_id,
          label,
          description,
          nodes: Array.from(this.localNodes.values()),
          avatars: Array.from(this.localAvatars.values()),
          metadata: {},
          created_at: new Date(),
          created_by: this.didSession?.split(':')[0] || '',
          version: 1
        };

        this.stats.snapshot_count++;
        this.emit('snapshot_saved', snapshot);
        return snapshot;
      } else {
        throw new Error(response.error || 'Failed to save snapshot');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Load a snapshot
   */
  async loadSnapshot(snapshotId: string): Promise<Snapshot> {
    // Check capabilities
    if (this.config.enable_capability_checking) {
      const hasCapability = await this.checkCapability('load_snapshot', {
        snapshot_id: snapshotId
      });
      
      if (!hasCapability) {
        throw new Error('Insufficient capabilities to load snapshot');
      }
    }

    try {
      const response = await this.sendRequest('load_snapshot', {
        snapshot_id: snapshotId
      });

      if (response.success) {
        const snapshot: Snapshot = response.snapshot;
        
        // Update local state
        this.localNodes.clear();
        this.localAvatars.clear();
        
        snapshot.nodes.forEach(node => {
          this.localNodes.set(node.id, node);
        });
        
        snapshot.avatars.forEach(avatar => {
          this.localAvatars.set(avatar.id, avatar);
        });
        
        this.stats.node_count = snapshot.nodes.length;
        this.stats.avatar_count = snapshot.avatars.length;
        this.stats.online_avatar_count = snapshot.avatars.filter(a => a.is_online).length;
        
        this.emit('snapshot_loaded', snapshot);
        return snapshot;
      } else {
        throw new Error(response.error || 'Failed to load snapshot');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Check if a capability is granted
   */
  async checkCapability(capability: string, context: Record<string, any>): Promise<boolean> {
    try {
      const response = await this.sendRequest('check_capability', {
        capability,
        context,
        did_session: this.didSession
      });

      return response.allowed || false;
    } catch (error) {
      return false;
    }
  }

  /**
   * Check if the current session has any of the specified capabilities
   */
  private hasCapability(requiredScopes: string[]): boolean {
    if (!this.currentSessionId) {
      return false;
    }

    for (const capToken of this.capTokens.values()) {
      if (capToken.session_id === this.currentSessionId) {
        for (const scope of requiredScopes) {
          if (capToken.granted_scopes.includes(scope)) {
            return true;
          }
        }
      }
    }
    return false;
  }

  private getCapToken(operation: string): string {
    // In real implementation, this would return the appropriate capability token
    // For now, return a mock token
    return `cap_${operation}_${Date.now()}`;
  }

  /**
   * Attach a policy to the scene
   */
  async attachPolicy(scope: string, regoBundle: string): Promise<string> {
    if (!this.currentSessionId) {
      throw new Error('No active session - cannot attach policy');
    }

    // Check for policy capability
    const hasPolicyCap = this.hasCapability(['policy:attach', 'scene:admin']);
    if (!hasPolicyCap) {
      this.emit('policy_denied', 'attach_policy', 'Insufficient capabilities');
      throw new Error('Insufficient capabilities - need policy:attach or scene:admin');
    }

    try {
      const response = await this.sendRequest('attach_policy', {
        session_id: this.currentSessionId,
        scope,
        rego_bundle_cbor: regoBundle
      });

      if (response.success) {
        return response.policy_id;
      } else {
        throw new Error(response.error || 'Failed to attach policy');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Simulate operations to check policy decisions
   */
  async simulateOperations(operations: Array<{
    action: string;
    node_id?: string;
    avatar_id?: string;
    transform?: Transform;
    metadata?: Record<string, any>;
  }>): Promise<{
    results: Array<{
      operation: any;
      allowed: boolean;
      reason?: string;
      policy_ref?: string;
    }>;
  }> {
    if (!this.currentSessionId) {
      throw new Error('No active session - cannot simulate operations');
    }

    try {
      const response = await this.sendRequest('simulate', {
        session_id: this.currentSessionId,
        operations
      });

      if (response.success) {
        return response;
      } else {
        throw new Error(response.error || 'Failed to simulate operations');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Check policy for an action
   */
  async checkPolicy(action: string, context: PolicyContext): Promise<PolicyResult> {
    try {
      const response = await this.sendRequest('check_policy', {
        action,
        context,
        did_session: this.didSession
      });

      return {
        allowed: response.allowed || false,
        reason: response.reason,
        conditions: response.conditions,
        rule_id: response.rule_id
      };
    } catch (error) {
      return {
        allowed: false,
        reason: `Policy check failed: ${error}`
      };
    }
  }

  /**
   * Get current statistics
   */
  getStats(): XRStats {
    return { ...this.stats };
  }

  /**
   * Get all nodes
   */
  getNodes(): SceneNode[] {
    return Array.from(this.localNodes.values());
  }

  /**
   * Get all avatars
   */
  getAvatars(): Avatar[] {
    return Array.from(this.localAvatars.values());
  }

  /**
   * Get a specific node
   */
  getNode(nodeId: string): SceneNode | undefined {
    return this.localNodes.get(nodeId);
  }

  /**
   * Get a specific avatar
   */
  getAvatar(avatarId: string): Avatar | undefined {
    return this.localAvatars.get(avatarId);
  }

  /**
   * Clear the scene
   */
  async clearScene(): Promise<void> {
    // Remove all nodes
    for (const nodeId of this.localNodes.keys()) {
      await this.removeNode(nodeId);
    }

    // Remove all avatars
    for (const avatarId of this.localAvatars.keys()) {
      await this.removeAvatar(avatarId);
    }
  }

  // XR Device Management

  /**
   * Start an XR device
   */
  async startXRDevice(deviceProfile: string, xrSessionId: string): Promise<XRDevice> {
    if (!this.currentSessionId) {
      throw new Error('No active session - cannot start XR device');
    }

    try {
      const response = await this.sendRequest('start_xr_device', {
        session_id: this.currentSessionId,
        device_profile: deviceProfile,
        xr_session_id: xrSessionId
      });

      if (response.success) {
        const device: XRDevice = {
          handle: response.device_handle,
          profile: deviceProfile,
          capabilities: response.capabilities,
          is_active: true
        };

        this.xrDevice = device;
        this.emit('xr_device_started', device);
        return device;
      } else {
        throw new Error(response.error || 'Failed to start XR device');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Stop the XR device
   */
  async stopXRDevice(): Promise<void> {
    if (!this.xrDevice) {
      throw new Error('No XR device to stop');
    }

    try {
      const response = await this.sendRequest('stop_xr_device', {
        device_handle: this.xrDevice.handle
      });

      if (response.success) {
        const deviceHandle = this.xrDevice.handle;
        this.xrDevice = undefined;
        this.emit('xr_device_stopped', deviceHandle);
      } else {
        throw new Error(response.error || 'Failed to stop XR device');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Send input to XR device
   */
  async sendInput(inputEvent: InputEvent): Promise<void> {
    if (!this.xrDevice) {
      throw new Error('No XR device active');
    }

    try {
      const response = await this.sendRequest('send_input', {
        session_id: this.currentSessionId,
        device_handle: this.xrDevice.handle,
        input_event: inputEvent
      });

      if (response.success) {
        this.emit('input_received', inputEvent);
      } else {
        throw new Error(response.error || 'Failed to send input');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Get current XR device
   */
  getXRDevice(): XRDevice | undefined {
    return this.xrDevice;
  }

  // Multi-user Session Management

  /**
   * Join a multi-user room
   */
  async joinRoom(roomId: string, displayName: string, avatarConfig?: AvatarConfig): Promise<{ participantId: string; roomState: RoomState }> {
    if (!this.currentSessionId) {
      throw new Error('No active session - cannot join room');
    }

    try {
      const response = await this.sendRequest('join_multi_user', {
        session_id: this.currentSessionId,
        room_id: roomId,
        display_name: displayName,
        avatar_config: avatarConfig
      });

      if (response.success) {
        this.currentRoom = response.room_state;
        this.emit('room_joined', roomId, response.participant_id);
        return {
          participantId: response.participant_id,
          roomState: response.room_state
        };
      } else {
        throw new Error(response.error || 'Failed to join room');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Leave the current room
   */
  async leaveRoom(): Promise<void> {
    if (!this.currentRoom) {
      throw new Error('Not in a room');
    }

    try {
      const response = await this.sendRequest('leave_room', {
        session_id: this.currentSessionId,
        room_id: this.currentRoom.room_id
      });

      if (response.success) {
        const roomId = this.currentRoom.room_id;
        this.currentRoom = undefined;
        this.emit('room_left', roomId);
      } else {
        throw new Error(response.error || 'Failed to leave room');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Sync scene with the room
   */
  async syncScene(lastVersion?: number): Promise<SceneSnapshot> {
    if (!this.currentRoom) {
      throw new Error('Not in a room');
    }

    try {
      const response = await this.sendRequest('sync_scene', {
        session_id: this.currentSessionId,
        room_id: this.currentRoom.room_id,
        last_version: lastVersion
      });

      if (response.success) {
        const snapshot = response.scene_snapshot;
        this.emit('scene_synchronized', snapshot);
        return snapshot;
      } else {
        throw new Error(response.error || 'Failed to sync scene');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Get current room state
   */
  getCurrentRoom(): RoomState | undefined {
    return this.currentRoom;
  }

  // Physics Interaction

  /**
   * Perform physics interaction
   */
  async physicsInteract(operation: PhysicsOperation): Promise<{ success: boolean; result?: any }> {
    if (!this.currentSessionId) {
      throw new Error('No active session - cannot perform physics interaction');
    }

    // Check for physics capability
    const hasPhysicsCap = this.hasCapability(['physics:interact', 'scene:interact']);
    if (!hasPhysicsCap) {
      this.emit('policy_denied', 'physics_interact', 'Insufficient capabilities');
      throw new Error('Insufficient capabilities - need physics:interact or scene:interact');
    }

    try {
      const response = await this.sendRequest('physics_interact', {
        session_id: this.currentSessionId,
        operation
      });

      if (response.success) {
        this.emit('physics_interaction', operation.type, operation.object_id || 'unknown');
        return {
          success: true,
          result: response.result
        };
      } else {
        throw new Error(response.error || 'Failed to perform physics interaction');
      }
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  /**
   * Grab an object
   */
  async grabObject(objectId: string, grabPoint: Vector3): Promise<void> {
    await this.physicsInteract({
      type: 'grab',
      object_id: objectId,
      grab_point: grabPoint
    });
    this.emit('object_grabbed', objectId, grabPoint);
  }

  /**
   * Move an object
   */
  async moveObject(objectId: string, targetTransform: Transform): Promise<void> {
    await this.physicsInteract({
      type: 'move',
      object_id: objectId,
      target_transform: targetTransform
    });
    this.emit('object_moved', objectId, targetTransform);
  }

  /**
   * Release an object
   */
  async releaseObject(objectId: string): Promise<void> {
    await this.physicsInteract({
      type: 'release',
      object_id: objectId
    });
    this.emit('object_released', objectId);
  }

  /**
   * Apply force to an object
   */
  async applyForce(objectId: string, force: Vector3, point: Vector3): Promise<void> {
    await this.physicsInteract({
      type: 'apply_force',
      object_id: objectId,
      force,
      point
    });
  }

  /**
   * Apply impulse to an object
   */
  async applyImpulse(objectId: string, impulse: Vector3, point: Vector3): Promise<void> {
    await this.physicsInteract({
      type: 'apply_impulse',
      object_id: objectId,
      impulse,
      point
    });
  }

  /**
   * Set object velocity
   */
  async setVelocity(objectId: string, velocity: Vector3): Promise<void> {
    await this.physicsInteract({
      type: 'set_velocity',
      object_id: objectId,
      velocity
    });
  }

  /**
   * Teleport an object
   */
  async teleportObject(objectId: string, transform: Transform): Promise<void> {
    await this.physicsInteract({
      type: 'teleport',
      object_id: objectId,
      target_transform: transform
    });
  }

  /**
   * Perform raycast
   */
  async raycast(origin: Vector3, direction: Vector3, maxDistance: number = 100): Promise<RaycastResult> {
    const result = await this.physicsInteract({
      type: 'raycast',
      origin,
      direction,
      max_distance: maxDistance
    });

    if (result.result) {
      this.emit('raycast_hit', result.result);
      return result.result;
    }

    return {
      hit: false,
      hit_point: { x: 0, y: 0, z: 0 },
      hit_normal: { x: 0, y: 1, z: 0 },
      distance: 0
    };
  }

  /**
   * Get current physics state
   */
  getPhysicsState(): PhysicsState | undefined {
    return this.physicsState;
  }

  // P4-06-A4: Persistence Methods

  /**
   * Persist room to NGFS
   */
  async persistRoom(roomId: string, reason: string, daoProposalId?: string): Promise<{ snapshotId: string; ngfsCid: string; deterministicHash: string }> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    const request = {
      id: uuidv4(),
      payload: {
        session_id: this.currentSessionId,
        room_id: roomId,
        reason,
        cap_token: this.getCapToken('persist_room'),
        dao_proposal_id: daoProposalId
      },
      timestamp: Date.now()
    };

    // In real implementation, this would call the Rust RPC service
    // For now, return mock data
    const snapshotId = `snapshot_${Date.now()}`;
    const ngfsCid = `QmMockCid${Date.now()}`;
    const deterministicHash = `hash_${Date.now()}`;

    const snapshotMetadata: SnapshotMetadata = {
      id: snapshotId,
      room_id: roomId,
      version: 1,
      created_at: Date.now(),
      created_by: this.currentSessionId,
      reason,
      dao_proposal_id: daoProposalId,
      deterministic_hash: deterministicHash,
      ngfs_cid: ngfsCid
    };

    this.snapshots.set(snapshotId, snapshotMetadata);
    this.stats.snapshot_count++;
    this.emit('room_persisted', roomId, snapshotId, ngfsCid);

    return { snapshotId, ngfsCid, deterministicHash };
  }

  /**
   * Load persistent room state from NGFS
   */
  async loadPersistentRoomState(snapshotId: string): Promise<PersistentRoomState> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    // In real implementation, this would call the Rust RPC service
    // For now, return mock data
    const mockState: PersistentRoomState = {
      room: this.currentRoom || { room_id: 'loaded_room', participants: [], last_updated: Date.now() },
      nodes: Array.from(this.localNodes.values()),
      avatars: Array.from(this.localAvatars.values()),
      physics_state: this.physicsState || { bodies: [], constraints: [], timestamp: Date.now() },
      policies: [],
      capabilities: []
    };

    this.emit('snapshot_loaded', snapshotId, mockState.room.room_id);
    return mockState;
  }

  /**
   * Get snapshot metadata
   */
  getSnapshotMetadata(snapshotId: string): SnapshotMetadata | undefined {
    return this.snapshots.get(snapshotId);
  }

  /**
   * List snapshots for a room
   */
  listSnapshots(roomId: string): SnapshotMetadata[] {
    return Array.from(this.snapshots.values()).filter(s => s.room_id === roomId);
  }

  /**
   * Verify snapshot determinism
   */
  async verifyDeterminism(snapshotId: string): Promise<boolean> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    // In real implementation, this would call the Rust RPC service
    // For now, return mock result
    return true;
  }

  // P4-06-A4: Recording Methods

  /**
   * Start recording a room
   */
  async startRecording(roomId: string, deterministicSeed?: number, physicsTickRate: number = 60.0): Promise<string> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    if (this.currentRecordingId) {
      throw new Error('Already recording');
    }

    const request = {
      id: uuidv4(),
      payload: {
        session_id: this.currentSessionId,
        room_id: roomId,
        cap_token: this.getCapToken('start_recording'),
        deterministic_seed: deterministicSeed,
        physics_tick_rate: physicsTickRate
      },
      timestamp: Date.now()
    };

    // In real implementation, this would call the Rust RPC service
    // For now, return mock data
    const recordingId = `recording_${Date.now()}`;
    this.currentRecordingId = recordingId;

    this.emit('recording_started', recordingId, roomId);
    return recordingId;
  }

  /**
   * Stop current recording
   */
  async stopRecording(): Promise<RecordingSummary> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    if (!this.currentRecordingId) {
      throw new Error('Not currently recording');
    }

    const request = {
      id: uuidv4(),
      payload: {
        session_id: this.currentSessionId,
        recording_id: this.currentRecordingId,
        cap_token: this.getCapToken('stop_recording')
      },
      timestamp: Date.now()
    };

    // In real implementation, this would call the Rust RPC service
    // For now, return mock data
    const summary: RecordingSummary = {
      recording_id: this.currentRecordingId,
      room_id: 'test_room',
      duration_seconds: 10.0,
      event_count: 100,
      started_at: Date.now() - 10000,
      stopped_at: Date.now(),
      deterministic_hash: `hash_${Date.now()}`,
      file_size_bytes: 1024,
      participants: [],
      devices_used: []
    };

    this.recordings.set(this.currentRecordingId, summary);
    this.stats.recording_count++;
    this.emit('recording_stopped', this.currentRecordingId, summary.room_id, summary.duration_seconds, summary.event_count);

    this.currentRecordingId = undefined;
    return summary;
  }

  /**
   * Get recording summary
   */
  getRecordingSummary(recordingId: string): RecordingSummary | undefined {
    return this.recordings.get(recordingId);
  }

  /**
   * List recordings for a room
   */
  listRecordings(roomId: string): RecordingSummary[] {
    return Array.from(this.recordings.values()).filter(r => r.room_id === roomId);
  }

  /**
   * Export recording to file
   */
  async exportRecording(recordingId: string, filePath: string, format: string = 'cbor'): Promise<boolean> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    // In real implementation, this would call the Rust RPC service and write to file
    // For now, return mock result
    console.log(`Exporting recording ${recordingId} to ${filePath} in ${format} format`);
    return true;
  }

  // P4-06-A4: Replay Methods

  /**
   * Replay a recording
   */
  async replayRecording(recordingId: string, mode: ReplayMode = ReplayMode.Headless, startEvent?: number, endEvent?: number, verifyDeterminism: boolean = true): Promise<string> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    if (this.currentReplayId) {
      throw new Error('Already replaying');
    }

    const request = {
      id: uuidv4(),
      payload: {
        session_id: this.currentSessionId,
        recording_id: recordingId,
        cap_token: this.getCapToken('replay_recording'),
        mode,
        start_event: startEvent,
        end_event: endEvent,
        verify_determinism: verifyDeterminism
      },
      timestamp: Date.now()
    };

    // In real implementation, this would call the Rust RPC service
    // For now, return mock data
    const replayId = `replay_${Date.now()}`;
    this.currentReplayId = replayId;

    this.emit('replay_started', replayId, recordingId, mode);
    return replayId;
  }

  /**
   * Stop current replay
   */
  async stopReplay(): Promise<boolean> {
    if (!this.currentReplayId) {
      return false;
    }

    // In real implementation, this would call the Rust RPC service
    this.emit('replay_stopped', this.currentReplayId);
    this.currentReplayId = undefined;
    return true;
  }

  /**
   * Get replay result
   */
  getReplayResult(replayId: string): ReplayResult | undefined {
    return this.replays.get(replayId);
  }

  /**
   * Verify replay determinism
   */
  async verifyReplayDeterminism(replayId: string): Promise<boolean> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    // In real implementation, this would call the Rust RPC service
    // For now, return mock result
    return true;
  }

  /**
   * Export replay snapshot
   */
  async exportReplaySnapshot(replayId: string): Promise<PersistentRoomState> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    // In real implementation, this would call the Rust RPC service
    // For now, return mock data
    const mockState: PersistentRoomState = {
      room: this.currentRoom || { room_id: 'replay_room', participants: [], last_updated: Date.now() },
      nodes: Array.from(this.localNodes.values()),
      avatars: Array.from(this.localAvatars.values()),
      physics_state: this.physicsState || { bodies: [], constraints: [], timestamp: Date.now() },
      policies: [],
      capabilities: []
    };

    return mockState;
  }

  // P4-06-A4: Bridge Methods

  /**
   * Export scene to various formats
   */
  async exportScene(roomId: string, formats: ExportFormat[], includeAvatars: boolean = true, includePhysics: boolean = true, compressionLevel: number = 6): Promise<string> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    const request = {
      id: uuidv4(),
      payload: {
        session_id: this.currentSessionId,
        room_id: roomId,
        cap_token: this.getCapToken('export_scene'),
        formats,
        include_avatars: includeAvatars,
        include_physics: includePhysics,
        compression_level: compressionLevel
      },
      timestamp: Date.now()
    };

    // In real implementation, this would call the Rust RPC service
    // For now, return mock data
    const bundleId = `bundle_${Date.now()}`;
    const bundle: ExportBundle = {
      id: bundleId,
      room_id: roomId,
      formats,
      created_at: Date.now(),
      created_by: this.currentSessionId,
      file_paths: {},
      manifest: {},
      ipfs_cids: {},
      total_size_bytes: 1024
    };

    this.exportBundles.set(bundleId, bundle);
    this.stats.export_bundle_count++;
    this.emit('scene_exported', bundleId, roomId, formats);

    return bundleId;
  }

  /**
   * Import scene from bundle
   */
  async importScene(bundleData: Uint8Array, policyMode: ImportPolicyMode = ImportPolicyMode.Sandbox, targetRoomId?: string, mergeMode: boolean = false): Promise<ImportResult> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    const request = {
      id: uuidv4(),
      payload: {
        session_id: this.currentSessionId,
        cap_token: this.getCapToken('import_scene'),
        bundle_data: Array.from(bundleData),
        policy_mode: policyMode,
        target_room_id: targetRoomId,
        merge_mode: mergeMode
      },
      timestamp: Date.now()
    };

    // In real implementation, this would call the Rust RPC service
    // For now, return mock data
    const importId = `import_${Date.now()}`;
    const result: ImportResult = {
      import_id: importId,
      room_id: targetRoomId || 'imported_room',
      success: true,
      imported_nodes: 5,
      imported_avatars: 2,
      imported_assets: 10,
      policy_violations: [],
      warnings: []
    };

    this.stats.import_count++;
    this.emit('scene_imported', importId, result.room_id, result.imported_nodes, result.imported_avatars);
    return result;
  }

  /**
   * Publish scene to public registry
   */
  async publishScene(roomId: string, anchor: boolean = false, daoProposalId?: string, publicAccess: boolean = true, license: string = 'MIT'): Promise<PublishResult> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    const request = {
      id: uuidv4(),
      payload: {
        session_id: this.currentSessionId,
        room_id: roomId,
        cap_token: this.getCapToken('publish_scene'),
        anchor,
        dao_proposal_id: daoProposalId,
        public_access: publicAccess,
        license
      },
      timestamp: Date.now()
    };

    // In real implementation, this would call the Rust RPC service
    // For now, return mock data
    const publishId = `publish_${Date.now()}`;
    const result: PublishResult = {
      publish_id: publishId,
      room_id: roomId,
      success: true,
      ipfs_cids: {},
      on_chain_tx_hash: anchor ? `0x${Date.now().toString(16)}` : undefined,
      dao_proposal_id: daoProposalId,
      public_url: `https://ipfs.io/ip/mock_cid_${publishId}`
    };

    this.stats.publish_count++;
    this.emit('scene_published', publishId, roomId, anchor, result.public_url);
    return result;
  }

  /**
   * Get export bundle
   */
  getExportBundle(bundleId: string): ExportBundle | undefined {
    return this.exportBundles.get(bundleId);
  }

  /**
   * List exports for a room
   */
  listExports(roomId: string): ExportBundle[] {
    return Array.from(this.exportBundles.values()).filter(b => b.room_id === roomId);
  }

  /**
   * Verify export integrity
   */
  async verifyExportIntegrity(bundleId: string): Promise<boolean> {
    if (!this.currentSessionId) {
      throw new Error('No active session');
    }

    // In real implementation, this would call the Rust RPC service
    // For now, return mock result
    return true;
  }

  // Private methods

  private setupEventHandlers(): void {
    // Setup any internal event handlers
  }

  private async initializeGRPCClient(): Promise<void> {
    // Initialize gRPC client
    // This would use the actual gRPC client library
    // For now, we'll simulate it
    this.sceneServiceClient = {
      close: async () => {}
    };
  }

  private async establishConnection(): Promise<void> {
    // Establish connection to scene service
    // This would use the actual gRPC client
    // For now, we'll simulate it
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  private startSyncLoop(): void {
    this.syncTimer = setInterval(() => {
      this.syncWithService();
    }, this.config.sync_interval);
  }

  private stopSyncLoop(): void {
    if (this.syncTimer) {
      clearInterval(this.syncTimer);
      this.syncTimer = undefined;
    }
  }

  private async syncWithService(): Promise<void> {
    if (!this.connected) {
      return;
    }

    try {
      // Sync local changes to service
      await this.pushLocalChanges();
      
      // Sync service changes to local
      await this.pullServiceChanges();
    } catch (error) {
      this.emit('scene_error', `Sync failed: ${error}`);
    }
  }

  private async pushLocalChanges(): Promise<void> {
    // Push local changes to the service
    // This would send any pending local changes
  }

  private async pullServiceChanges(): Promise<void> {
    // Pull changes from the service
    // This would receive updates from the service
  }

  private async sendRequest(method: string, data: any): Promise<any> {
    this.stats.total_requests++;
    const startTime = Date.now();

    try {
      // This would send the actual gRPC request
      // For now, we'll simulate a successful response
      const response = {
        success: true,
        method,
        ...data
      };

      this.stats.successful_requests++;
      this.stats.average_response_time = 
        (this.stats.average_response_time + (Date.now() - startTime)) / 2;

      return response;
    } catch (error) {
      this.stats.failed_requests++;
      throw error;
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.config.max_reconnect_attempts) {
      this.emit('scene_error', 'Max reconnect attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);

    this.reconnectTimer = setTimeout(() => {
      this.connect().catch(error => {
        this.emit('scene_error', `Reconnect failed: ${error}`);
      });
    }, delay);
  }

  private generateNodeId(): string {
    return `node_${uuidv4()}`;
  }

  private generateAvatarId(): string {
    return `avatar_${uuidv4()}`;
  }
}

// Export default instance
export const xrBridge = new XRBridge();
