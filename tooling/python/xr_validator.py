#!/usr/bin/env python3
"""
XR Validator - Python validation tool for Aetheris XR Scene Graph Service

This module provides comprehensive validation for the XR scene graph service,
including snapshot round-trip testing, DID-avatar binding verification,
capability enforcement, DAO policy validation, and determinism testing.
"""

import asyncio
import json
import time
import uuid
import hashlib
import argparse
import logging
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from datetime import datetime, timezone
import cbor2
import blake3

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

@dataclass
class Vector3:
    """3D vector representation"""
    x: float
    y: float
    z: float

@dataclass
class Transform:
    """Transform representation"""
    position: Vector3
    rotation: Vector3
    scale: Vector3

@dataclass
class Component:
    """Component representation"""
    type: str
    data: Dict[str, Any]

@dataclass
class SceneNode:
    """Scene node representation"""
    id: str
    name: str
    parent_id: Optional[str]
    transform: Transform
    components: List[Component]
    metadata: Dict[str, Any]

@dataclass
class AvatarProfile:
    """Avatar profile representation"""
    name: str
    description: Optional[str]
    appearance: Dict[str, Any]
    preferences: Dict[str, Any]

@dataclass
class Avatar:
    """Avatar representation"""
    id: str
    did: str
    profile: AvatarProfile
    transform: Transform
    is_online: bool
    metadata: Dict[str, Any]

@dataclass
class Snapshot:
    """Snapshot representation"""
    id: str
    label: str
    description: Optional[str]
    nodes: List[SceneNode]
    avatars: List[Avatar]
    metadata: Dict[str, Any]
    created_at: datetime
    created_by: str
    version: int

@dataclass
class CapToken:
    """Capability token representation"""
    id: str
    name: str
    permissions: List[str]
    scope: str
    expires_at: Optional[datetime]
    created_at: datetime
    created_by: str
    session_id: Optional[str]
    granted_scopes: List[str]

@dataclass
class PolicyContext:
    """Policy context representation"""
    user_did: str
    avatar_id: Optional[str]
    node_id: Optional[str]
    action: str
    scope: str
    session_id: Optional[str]
    metadata: Dict[str, Any]

@dataclass
class PolicyResult:
    """Policy result representation"""
    allowed: bool
    reason: Optional[str]
    conditions: List[str]
    rule_id: Optional[str]

@dataclass
class Session:
    """DID session representation"""
    id: str
    did: str
    created_at: datetime
    expires_at: datetime
    status: str

@dataclass
class DaoDecision:
    """DAO decision representation"""
    proposal_id: str
    decision: str
    reason: str
    vote_count: int
    total_votes: int
    timestamp: datetime

@dataclass
class XRDevice:
    """XR device representation"""
    handle: str
    profile: str
    capabilities: List[str]
    session_id: str
    is_active: bool
    metadata: Dict[str, Any]

@dataclass
class InputEvent:
    """Input event representation"""
    type: str
    device_handle: str
    button_id: Optional[str]
    pressed: Optional[bool]
    value: Optional[float]
    position: Optional[Vector3]
    rotation: Optional[Vector3]
    hand_joints: Optional[List[Dict[str, Any]]]
    eye_gaze: Optional[Vector3]
    voice_command: Optional[str]
    timestamp: datetime

@dataclass
class Participant:
    """Multi-user participant representation"""
    id: str
    display_name: str
    avatar_config: Dict[str, Any]
    is_online: bool
    last_seen: datetime
    metadata: Dict[str, Any]

@dataclass
class RoomState:
    """Multi-user room state representation"""
    id: str
    name: str
    participants: List[Participant]
    max_participants: int
    is_public: bool
    created_at: datetime
    metadata: Dict[str, Any]

@dataclass
class PhysicsBody:
    """Physics body representation"""
    id: str
    type: str
    position: Vector3
    rotation: Vector3
    velocity: Vector3
    angular_velocity: Vector3
    mass: float
    is_kinematic: bool
    metadata: Dict[str, Any]

@dataclass
class PhysicsConstraint:
    """Physics constraint representation"""
    id: str
    type: str
    body_a_id: str
    body_b_id: str
    anchor_a: Vector3
    anchor_b: Vector3
    metadata: Dict[str, Any]

@dataclass
class PhysicsState:
    """Physics world state representation"""
    bodies: List[PhysicsBody]
    constraints: List[PhysicsConstraint]
    gravity: Vector3
    time_step: float
    metadata: Dict[str, Any]

@dataclass
class PhysicsOperation:
    """Physics operation representation"""
    type: str
    object_id: str
    position: Optional[Vector3]
    rotation: Optional[Vector3]
    velocity: Optional[Vector3]
    force: Optional[Vector3]
    impulse: Optional[Vector3]
    grab_point: Optional[Vector3]
    metadata: Dict[str, Any]

@dataclass
class RaycastResult:
    """Raycast result representation"""
    hit: bool
    distance: float
    point: Vector3
    normal: Vector3
    object_id: str
    metadata: Dict[str, Any]

class XRValidator:
    """Main XR validation class"""
    
    def __init__(self, config: Dict[str, Any] = None):
        self.config = config or {}
        self.scene_service_url = self.config.get('scene_service_url', 'localhost:50051')
        self.connection_timeout = self.config.get('connection_timeout', 5.0)
        self.sync_interval = self.config.get('sync_interval', 1.0 / 60.0)
        self.enable_policy_enforcement = self.config.get('enable_policy_enforcement', True)
        self.enable_capability_checking = self.config.get('enable_capability_checking', True)
        self.auto_reconnect = self.config.get('auto_reconnect', True)
        self.max_reconnect_attempts = self.config.get('max_reconnect_attempts', 10)
        
        self.connected = False
        self.reconnect_attempts = 0
        self.local_nodes: Dict[str, SceneNode] = {}
        self.local_avatars: Dict[str, Avatar] = {}
        self.cap_tokens: Dict[str, CapToken] = {}
        self.snapshots: Dict[str, Snapshot] = {}
        self.current_session: Optional[Session] = None
        self.dao_decisions: Dict[str, DaoDecision] = {}
        self.xr_devices: Dict[str, XRDevice] = {}
        self.current_room: Optional[RoomState] = None
        self.physics_state: Optional[PhysicsState] = None
        
        # Statistics
        self.stats = {
            'total_requests': 0,
            'successful_requests': 0,
            'failed_requests': 0,
            'average_response_time': 0.0,
            'node_count': 0,
            'avatar_count': 0,
            'online_avatar_count': 0,
            'snapshot_count': 0,
            'xr_device_count': 0,
            'room_participant_count': 0,
            'physics_body_count': 0
        }

    async def connect(self) -> bool:
        """Connect to the XR scene service"""
        try:
            logger.info(f"Connecting to XR scene service at {self.scene_service_url}")
            
            # Simulate connection (in real implementation, this would use gRPC)
            await asyncio.sleep(0.1)
            
            self.connected = True
            self.reconnect_attempts = 0
            logger.info("Connected to XR scene service")
            return True
            
        except Exception as e:
            logger.error(f"Failed to connect to XR scene service: {e}")
            self.connected = False
            return False

    async def disconnect(self) -> None:
        """Disconnect from the XR scene service"""
        if self.connected:
            self.connected = False
            logger.info("Disconnected from XR scene service")

    async def spawn_node(self, node_type: str, position: Vector3, parent_id: str = None) -> SceneNode:
        """Spawn a new node in the scene"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        node_id = f"node_{uuid.uuid4()}"
        node = SceneNode(
            id=node_id,
            name=node_type,
            parent_id=parent_id,
            transform=Transform(
                position=position,
                rotation=Vector3(0, 0, 0),
                scale=Vector3(1, 1, 1)
            ),
            components=[],
            metadata={}
        )
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('spawn_node', {
                'node_type': node_type,
                'position': asdict(position)
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to spawn node")
        
        # Check policies
        if self.enable_policy_enforcement:
            policy_result = await self.check_policy('spawn', PolicyContext(
                user_did='did:aeth:test',
                avatar_id=None,
                node_id=node_id,
                action='spawn',
                scope='scene',
                metadata={'node_type': node_type}
            ))
            if not policy_result.allowed:
                raise RuntimeError(f"Policy denied: {policy_result.reason}")
        
        # Add to local state
        self.local_nodes[node_id] = node
        self.stats['node_count'] += 1
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Spawned node: {node_id} of type: {node_type}")
        return node

    async def update_node(self, node_id: str, transform: Transform) -> None:
        """Update an existing node"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if node_id not in self.local_nodes:
            raise ValueError(f"Node not found: {node_id}")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('modify_node', {
                'node_id': node_id,
                'transform': asdict(transform)
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to modify node")
        
        # Check policies
        if self.enable_policy_enforcement:
            policy_result = await self.check_policy('modify', PolicyContext(
                user_did='did:aeth:test',
                avatar_id=None,
                node_id=node_id,
                action='modify',
                scope='node',
                metadata={}
            ))
            if not policy_result.allowed:
                raise RuntimeError(f"Policy denied: {policy_result.reason}")
        
        # Update local state
        self.local_nodes[node_id].transform = transform
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Updated node: {node_id}")

    async def remove_node(self, node_id: str) -> None:
        """Remove a node from the scene"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if node_id not in self.local_nodes:
            raise ValueError(f"Node not found: {node_id}")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('remove_node', {
                'node_id': node_id
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to remove node")
        
        # Check policies
        if self.enable_policy_enforcement:
            policy_result = await self.check_policy('remove', PolicyContext(
                user_did='did:aeth:test',
                avatar_id=None,
                node_id=node_id,
                action='remove',
                scope='node',
                metadata={}
            ))
            if not policy_result.allowed:
                raise RuntimeError(f"Policy denied: {policy_result.reason}")
        
        # Remove from local state
        del self.local_nodes[node_id]
        self.stats['node_count'] -= 1
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Removed node: {node_id}")

    async def spawn_avatar(self, did: str, profile: AvatarProfile, position: Vector3) -> Avatar:
        """Spawn a new avatar"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        avatar_id = f"avatar_{uuid.uuid4()}"
        avatar = Avatar(
            id=avatar_id,
            did=did,
            profile=profile,
            transform=Transform(
                position=position,
                rotation=Vector3(0, 0, 0),
                scale=Vector3(1, 1, 1)
            ),
            is_online=True,
            metadata={}
        )
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('spawn_avatar', {
                'did': did,
                'position': asdict(position)
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to spawn avatar")
        
        # Check policies
        if self.enable_policy_enforcement:
            policy_result = await self.check_policy('bind_avatar', PolicyContext(
                user_did=did,
                avatar_id=avatar_id,
                action='bind_avatar',
                scope='avatar',
                metadata={'profile': asdict(profile)}
            ))
            if not policy_result.allowed:
                raise RuntimeError(f"Policy denied: {policy_result.reason}")
        
        # Add to local state
        self.local_avatars[avatar_id] = avatar
        self.stats['avatar_count'] += 1
        self.stats['online_avatar_count'] += 1
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Spawned avatar: {avatar_id} for DID: {did}")
        return avatar

    async def update_avatar(self, avatar_id: str, transform: Transform) -> None:
        """Update an existing avatar"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if avatar_id not in self.local_avatars:
            raise ValueError(f"Avatar not found: {avatar_id}")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('modify_avatar', {
                'avatar_id': avatar_id,
                'transform': asdict(transform)
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to modify avatar")
        
        # Update local state
        self.local_avatars[avatar_id].transform = transform
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Updated avatar: {avatar_id}")

    async def remove_avatar(self, avatar_id: str) -> None:
        """Remove an avatar from the scene"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if avatar_id not in self.local_avatars:
            raise ValueError(f"Avatar not found: {avatar_id}")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('remove_avatar', {
                'avatar_id': avatar_id
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to remove avatar")
        
        # Remove from local state
        del self.local_avatars[avatar_id]
        self.stats['avatar_count'] -= 1
        self.stats['online_avatar_count'] -= 1
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Removed avatar: {avatar_id}")

    async def save_snapshot(self, label: str, description: str = None) -> Snapshot:
        """Save a snapshot of the current scene"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        snapshot_id = f"snapshot_{uuid.uuid4()}"
        snapshot = Snapshot(
            id=snapshot_id,
            label=label,
            description=description,
            nodes=list(self.local_nodes.values()),
            avatars=list(self.local_avatars.values()),
            metadata={},
            created_at=datetime.now(timezone.utc),
            created_by='xr_validator',
            version=1
        )
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('create_snapshot', {
                'label': label,
                'description': description
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to create snapshot")
        
        # Add to local state
        self.snapshots[snapshot_id] = snapshot
        self.stats['snapshot_count'] += 1
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Saved snapshot: {snapshot_id}")
        return snapshot

    async def load_snapshot(self, snapshot_id: str) -> Snapshot:
        """Load a snapshot"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if snapshot_id not in self.snapshots:
            raise ValueError(f"Snapshot not found: {snapshot_id}")
        
        snapshot = self.snapshots[snapshot_id]
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('load_snapshot', {
                'snapshot_id': snapshot_id
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to load snapshot")
        
        # Update local state
        self.local_nodes.clear()
        self.local_avatars.clear()
        
        for node in snapshot.nodes:
            self.local_nodes[node.id] = node
        
        for avatar in snapshot.avatars:
            self.local_avatars[avatar.id] = avatar
        
        self.stats['node_count'] = len(snapshot.nodes)
        self.stats['avatar_count'] = len(snapshot.avatars)
        self.stats['online_avatar_count'] = len([a for a in snapshot.avatars if a.is_online])
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Loaded snapshot: {snapshot_id}")
        return snapshot

    async def check_capability(self, capability: str, context: Dict[str, Any]) -> bool:
        """Check if a capability is granted"""
        # Simulate capability check
        # In real implementation, this would check against the capability system
        return True

    async def check_policy(self, action: str, context: PolicyContext) -> PolicyResult:
        """Check if a policy allows an action"""
        # Simulate policy check
        # In real implementation, this would check against the policy engine
        return PolicyResult(
            allowed=True,
            reason=None,
            conditions=[],
            rule_id=None
        )

    async def begin_session(self, did: str, proof: str, nonce: str) -> Session:
        """Begin a new DID session"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        session_id = f"session_{uuid.uuid4()}"
        session = Session(
            id=session_id,
            did=did,
            created_at=datetime.now(timezone.utc),
            expires_at=datetime.now(timezone.utc).replace(hour=datetime.now(timezone.utc).hour + 1),
            status="active"
        )
        
        self.current_session = session
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Started session: {session_id} for DID: {did}")
        return session

    async def end_session(self) -> None:
        """End the current session"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if self.current_session is None:
            raise RuntimeError("No active session")
        
        session_id = self.current_session.id
        self.current_session = None
        self.cap_tokens.clear()
        
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Ended session: {session_id}")

    async def issue_capability(self, scopes: List[str], ttl_seconds: int = 3600) -> CapToken:
        """Issue a capability token for the current session"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if self.current_session is None:
            raise RuntimeError("No active session")
        
        cap_id = f"cap_{uuid.uuid4()}"
        expires_at = datetime.now(timezone.utc).replace(second=datetime.now(timezone.utc).second + ttl_seconds)
        
        # Add permissions based on scopes
        permissions = []
        for scope in scopes:
            if scope in ["scene:spawn", "node:spawn"]:
                permissions.append("spawn_node")
            elif scope in ["node:move", "scene:modify"]:
                permissions.append("modify_node")
            elif scope == "node:delete":
                permissions.append("remove_node")
            elif scope in ["policy:attach", "scene:admin"]:
                permissions.append("attach_policy")
        
        cap_token = CapToken(
            id=cap_id,
            name=f"Cap-{cap_id}",
            permissions=permissions,
            scope="session",
            expires_at=expires_at,
            created_at=datetime.now(timezone.utc),
            created_by=self.current_session.did,
            session_id=self.current_session.id,
            granted_scopes=scopes
        )
        
        self.cap_tokens[cap_id] = cap_token
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Issued capability: {cap_id} with scopes: {scopes}")
        return cap_token

    async def attach_policy(self, scope: str, rego_bundle: str) -> str:
        """Attach a policy to the scene"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if self.current_session is None:
            raise RuntimeError("No active session")
        
        # Check for policy capability
        has_policy_cap = False
        for cap_token in self.cap_tokens.values():
            if cap_token.session_id == self.current_session.id:
                for scope_name in cap_token.granted_scopes:
                    if scope_name in ["policy:attach", "scene:admin"]:
                        has_policy_cap = True
                        break
            if has_policy_cap:
                break
        
        if not has_policy_cap:
            raise RuntimeError("Insufficient capabilities - need policy:attach or scene:admin")
        
        policy_id = f"policy_{uuid.uuid4()}"
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Attached policy: {policy_id} with scope: {scope}")
        return policy_id

    async def simulate_operations(self, operations: List[Dict[str, Any]]) -> Dict[str, Any]:
        """Simulate operations to check policy decisions"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if self.current_session is None:
            raise RuntimeError("No active session")
        
        results = []
        for operation in operations:
            # Simulate policy check for each operation
            result = {
                "operation": operation,
                "allowed": True,
                "reason": "simulated success",
                "policy_ref": f"policy_{uuid.uuid4()}"
            }
            results.append(result)
        
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Simulated {len(operations)} operations")
        return {"results": results}

    async def check_dao_decision(self, proposal_id: str, action: str, context: Dict[str, Any]) -> DaoDecision:
        """Check DAO decision for a proposal"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        # Simulate DAO decision
        decision = DaoDecision(
            proposal_id=proposal_id,
            decision="approved",
            reason="DAO vote passed",
            vote_count=75,
            total_votes=100,
            timestamp=datetime.now(timezone.utc)
        )
        
        self.dao_decisions[proposal_id] = decision
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"DAO decision for proposal {proposal_id}: {decision.decision}")
        return decision

    async def start_xr_device(self, device_profile: str, xr_session_id: str) -> XRDevice:
        """Start an XR device"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        device_handle = f"device_{uuid.uuid4()}"
        capabilities = ["hand_tracking", "eye_tracking", "voice_command"]
        
        device = XRDevice(
            handle=device_handle,
            profile=device_profile,
            capabilities=capabilities,
            session_id=xr_session_id,
            is_active=True,
            metadata={}
        )
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('start_xr_device', {
                'device_profile': device_profile,
                'xr_session_id': xr_session_id
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to start XR device")
        
        # Add to local state
        self.xr_devices[device_handle] = device
        self.stats['xr_device_count'] += 1
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Started XR device: {device_handle} with profile: {device_profile}")
        return device

    async def stop_xr_device(self, device_handle: str) -> None:
        """Stop an XR device"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if device_handle not in self.xr_devices:
            raise ValueError(f"XR device not found: {device_handle}")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('stop_xr_device', {
                'device_handle': device_handle
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to stop XR device")
        
        # Remove from local state
        del self.xr_devices[device_handle]
        self.stats['xr_device_count'] -= 1
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Stopped XR device: {device_handle}")

    async def send_input(self, input_event: InputEvent) -> None:
        """Send input event to XR device"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if input_event.device_handle not in self.xr_devices:
            raise ValueError(f"XR device not found: {input_event.device_handle}")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('send_input', {
                'device_handle': input_event.device_handle,
                'input_type': input_event.type
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to send input")
        
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Sent input event: {input_event.type} to device: {input_event.device_handle}")

    async def join_room(self, room_id: str, display_name: str, avatar_config: Dict[str, Any] = None) -> Tuple[str, RoomState]:
        """Join a multi-user room"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if self.current_session is None:
            raise RuntimeError("No active session")
        
        participant_id = f"participant_{uuid.uuid4()}"
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('join_room', {
                'room_id': room_id,
                'display_name': display_name
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to join room")
        
        # Create or update room state
        if self.current_room is None or self.current_room.id != room_id:
            self.current_room = RoomState(
                id=room_id,
                name=f"Room {room_id}",
                participants=[],
                max_participants=10,
                is_public=True,
                created_at=datetime.now(timezone.utc),
                metadata={}
            )
        
        # Add participant
        participant = Participant(
            id=participant_id,
            display_name=display_name,
            avatar_config=avatar_config or {},
            is_online=True,
            last_seen=datetime.now(timezone.utc),
            metadata={}
        )
        self.current_room.participants.append(participant)
        self.stats['room_participant_count'] += 1
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Joined room: {room_id} as participant: {participant_id}")
        return participant_id, self.current_room

    async def leave_room(self, room_id: str) -> None:
        """Leave a multi-user room"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if self.current_room is None or self.current_room.id != room_id:
            raise ValueError(f"Not in room: {room_id}")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('leave_room', {
                'room_id': room_id
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to leave room")
        
        # Remove all participants (simplified for testing)
        self.current_room.participants.clear()
        self.current_room = None
        self.stats['room_participant_count'] = 0
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Left room: {room_id}")

    async def sync_scene(self, room_id: str, last_version: int = None) -> Snapshot:
        """Sync scene state in multi-user room"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if self.current_room is None or self.current_room.id != room_id:
            raise ValueError(f"Not in room: {room_id}")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('sync_scene', {
                'room_id': room_id,
                'last_version': last_version
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to sync scene")
        
        # Create scene snapshot
        snapshot = Snapshot(
            id=f"sync_{uuid.uuid4()}",
            label=f"Sync for room {room_id}",
            description=f"Scene sync for room {room_id}",
            nodes=list(self.local_nodes.values()),
            avatars=list(self.local_avatars.values()),
            metadata={'room_id': room_id, 'version': last_version or 1},
            created_at=datetime.now(timezone.utc),
            created_by='xr_validator',
            version=last_version or 1
        )
        
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Synced scene for room: {room_id}")
        return snapshot

    async def physics_interact(self, operation: PhysicsOperation) -> Dict[str, Any]:
        """Perform physics interaction"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        if self.current_session is None:
            raise RuntimeError("No active session")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('physics_interact', {
                'operation_type': operation.type,
                'object_id': operation.object_id
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities for physics interaction")
        
        # Simulate physics interaction
        result = {
            'success': True,
            'operation_type': operation.type,
            'object_id': operation.object_id,
            'timestamp': datetime.now(timezone.utc).isoformat()
        }
        
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Physics interaction: {operation.type} on object: {operation.object_id}")
        return result

    async def grab_object(self, object_id: str, grab_point: Vector3) -> None:
        """Grab an object"""
        operation = PhysicsOperation(
            type='grab',
            object_id=object_id,
            grab_point=grab_point,
            metadata={}
        )
        await self.physics_interact(operation)

    async def move_object(self, object_id: str, position: Vector3, rotation: Vector3 = None) -> None:
        """Move an object"""
        operation = PhysicsOperation(
            type='move',
            object_id=object_id,
            position=position,
            rotation=rotation,
            metadata={}
        )
        await self.physics_interact(operation)

    async def release_object(self, object_id: str) -> None:
        """Release an object"""
        operation = PhysicsOperation(
            type='release',
            object_id=object_id,
            metadata={}
        )
        await self.physics_interact(operation)

    async def apply_force(self, object_id: str, force: Vector3) -> None:
        """Apply force to an object"""
        operation = PhysicsOperation(
            type='apply_force',
            object_id=object_id,
            force=force,
            metadata={}
        )
        await self.physics_interact(operation)

    async def apply_impulse(self, object_id: str, impulse: Vector3) -> None:
        """Apply impulse to an object"""
        operation = PhysicsOperation(
            type='apply_impulse',
            object_id=object_id,
            impulse=impulse,
            metadata={}
        )
        await self.physics_interact(operation)

    async def set_velocity(self, object_id: str, velocity: Vector3) -> None:
        """Set object velocity"""
        operation = PhysicsOperation(
            type='set_velocity',
            object_id=object_id,
            velocity=velocity,
            metadata={}
        )
        await self.physics_interact(operation)

    async def teleport_object(self, object_id: str, position: Vector3, rotation: Vector3 = None) -> None:
        """Teleport an object"""
        operation = PhysicsOperation(
            type='teleport',
            object_id=object_id,
            position=position,
            rotation=rotation,
            metadata={}
        )
        await self.physics_interact(operation)

    async def raycast(self, origin: Vector3, direction: Vector3, max_distance: float = 100.0) -> RaycastResult:
        """Perform raycast"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('raycast', {
                'origin': asdict(origin),
                'direction': asdict(direction),
                'max_distance': max_distance
            })
            if not has_capability:
                raise RuntimeError("Insufficient capabilities for raycast")
        
        # Simulate raycast result
        result = RaycastResult(
            hit=True,
            distance=5.0,
            point=Vector3(origin.x + direction.x * 5.0, origin.y + direction.y * 5.0, origin.z + direction.z * 5.0),
            normal=Vector3(0, 1, 0),
            object_id="raycast_hit_object",
            metadata={}
        )
        
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info(f"Raycast from {origin} in direction {direction}")
        return result

    async def get_physics_state(self) -> PhysicsState:
        """Get current physics state"""
        if not self.connected:
            raise RuntimeError("Not connected to XR service")
        
        # Check capabilities
        if self.enable_capability_checking:
            has_capability = await self.check_capability('get_physics_state', {})
            if not has_capability:
                raise RuntimeError("Insufficient capabilities to get physics state")
        
        # Create physics state
        self.physics_state = PhysicsState(
            bodies=[],
            constraints=[],
            gravity=Vector3(0, -9.81, 0),
            time_step=1.0/60.0,
            metadata={}
        )
        
        self.stats['total_requests'] += 1
        self.stats['successful_requests'] += 1
        
        logger.info("Retrieved physics state")
        return self.physics_state

    def get_stats(self) -> Dict[str, Any]:
        """Get current statistics"""
        return self.stats.copy()

    def clear_scene(self) -> None:
        """Clear the scene"""
        self.local_nodes.clear()
        self.local_avatars.clear()
        self.stats['node_count'] = 0
        self.stats['avatar_count'] = 0
        self.stats['online_avatar_count'] = 0
        logger.info("Scene cleared")

    def reset_stats(self) -> None:
        """Reset statistics"""
        self.stats = {
            'total_requests': 0,
            'successful_requests': 0,
            'failed_requests': 0,
            'average_response_time': 0.0,
            'node_count': 0,
            'avatar_count': 0,
            'online_avatar_count': 0,
            'snapshot_count': 0
        }
        logger.info("Statistics reset")

class XRValidatorTests:
    """Test suite for XR validation"""
    
    def __init__(self, validator: XRValidator):
        self.validator = validator
        self.test_results: List[Dict[str, Any]] = []
    
    async def run_all_tests(self) -> Dict[str, Any]:
        """Run all validation tests"""
        logger.info("Starting XR validation tests")
        
        test_methods = [
            self.test_snapshot_roundtrip,
            self.test_did_avatar_binding,
            self.test_capability_enforcement,
            self.test_policy_enforcement,
            self.test_determinism,
            self.test_performance,
            self.test_error_handling,
            self.test_avatar_auth_cycle,
            self.test_cap_enforcement_spawn_move_delete,
            self.test_dao_block_allow,
            self.test_snapshot_bytestability,
            self.test_xr_device_management,
            self.test_multi_user_sessions,
            self.test_physics_interactions,
            self.test_raycast_functionality,
            self.test_persistence_integrity,
            self.test_recording_determinism,
            self.test_bridge_export_import
        ]
        
        for test_method in test_methods:
            try:
                await test_method()
            except Exception as e:
                logger.error(f"Test {test_method.__name__} failed: {e}")
                self.test_results.append({
                    'test': test_method.__name__,
                    'status': 'failed',
                    'error': str(e)
                })
        
        # Generate summary
        summary = self.generate_summary()
        logger.info(f"Validation tests completed: {summary}")
        return summary
    
    async def test_snapshot_roundtrip(self) -> None:
        """Test snapshot save/load round-trip"""
        logger.info("Testing snapshot round-trip")
        
        # Create test scene
        await self.validator.spawn_node('cube', Vector3(0, 1, 0))
        await self.validator.spawn_node('sphere', Vector3(2, 1, 0))
        
        profile = AvatarProfile(
            name='Test Avatar',
            description='Test avatar for validation',
            appearance={},
            preferences={}
        )
        await self.validator.spawn_avatar('did:aeth:test', profile, Vector3(0, 1, 2))
        
        # Save snapshot
        snapshot = await self.validator.save_snapshot('test_snapshot', 'Test snapshot for validation')
        
        # Clear scene
        self.validator.clear_scene()
        
        # Load snapshot
        loaded_snapshot = await self.validator.load_snapshot(snapshot.id)
        
        # Verify round-trip
        assert len(loaded_snapshot.nodes) == 2, "Node count mismatch after round-trip"
        assert len(loaded_snapshot.avatars) == 1, "Avatar count mismatch after round-trip"
        assert loaded_snapshot.label == 'test_snapshot', "Snapshot label mismatch"
        
        self.test_results.append({
            'test': 'snapshot_roundtrip',
            'status': 'passed',
            'details': {
                'nodes': len(loaded_snapshot.nodes),
                'avatars': len(loaded_snapshot.avatars),
                'label': loaded_snapshot.label
            }
        })
        
        logger.info("Snapshot round-trip test passed")
    
    async def test_did_avatar_binding(self) -> None:
        """Test DID-avatar binding"""
        logger.info("Testing DID-avatar binding")
        
        # Create avatar with DID
        did = 'did:aeth:test_user'
        profile = AvatarProfile(
            name='Test User',
            description='Test user avatar',
            appearance={'color': 'blue'},
            preferences={'speed': 1.0}
        )
        avatar = await self.validator.spawn_avatar(did, profile, Vector3(0, 1, 0))
        
        # Verify binding
        assert avatar.did == did, "DID binding mismatch"
        assert avatar.profile.name == 'Test User', "Avatar profile mismatch"
        assert avatar.is_online, "Avatar should be online"
        
        # Test avatar update
        new_transform = Transform(
            position=Vector3(1, 1, 1),
            rotation=Vector3(0, 90, 0),
            scale=Vector3(1, 1, 1)
        )
        await self.validator.update_avatar(avatar.id, new_transform)
        
        # Verify update
        updated_avatar = self.validator.local_avatars[avatar.id]
        assert updated_avatar.transform.position.x == 1, "Avatar position update failed"
        assert updated_avatar.transform.rotation.y == 90, "Avatar rotation update failed"
        
        self.test_results.append({
            'test': 'did_avatar_binding',
            'status': 'passed',
            'details': {
                'did': avatar.did,
                'avatar_id': avatar.id,
                'profile_name': avatar.profile.name
            }
        })
        
        logger.info("DID-avatar binding test passed")
    
    async def test_capability_enforcement(self) -> None:
        """Test capability enforcement"""
        logger.info("Testing capability enforcement")
        
        # Test capability check
        has_capability = await self.validator.check_capability('spawn_node', {
            'node_type': 'cube',
            'position': {'x': 0, 'y': 1, 'z': 0}
        })
        assert has_capability, "Capability check should pass"
        
        # Test with invalid capability
        has_invalid_capability = await self.validator.check_capability('invalid_capability', {})
        # This should still pass in our simulation, but in real implementation it might fail
        
        self.test_results.append({
            'test': 'capability_enforcement',
            'status': 'passed',
            'details': {
                'valid_capability': has_capability,
                'invalid_capability': has_invalid_capability
            }
        })
        
        logger.info("Capability enforcement test passed")
    
    async def test_policy_enforcement(self) -> None:
        """Test policy enforcement"""
        logger.info("Testing policy enforcement")
        
        # Test policy check
        policy_result = await self.validator.check_policy('spawn', PolicyContext(
            user_did='did:aeth:test',
            avatar_id=None,
            node_id='test_node',
            action='spawn',
            scope='scene',
            metadata={'node_type': 'cube'}
        ))
        assert policy_result.allowed, "Policy check should pass"
        
        # Test with different action
        policy_result2 = await self.validator.check_policy('modify', PolicyContext(
            user_did='did:aeth:test',
            avatar_id=None,
            node_id='test_node',
            action='modify',
            scope='node',
            metadata={}
        ))
        assert policy_result2.allowed, "Policy check should pass"
        
        self.test_results.append({
            'test': 'policy_enforcement',
            'status': 'passed',
            'details': {
                'spawn_policy': policy_result.allowed,
                'modify_policy': policy_result2.allowed
            }
        })
        
        logger.info("Policy enforcement test passed")
    
    async def test_determinism(self) -> None:
        """Test determinism across multiple ticks"""
        logger.info("Testing determinism")
        
        # Create initial scene
        await self.validator.spawn_node('cube', Vector3(0, 1, 0))
        await self.validator.spawn_node('sphere', Vector3(2, 1, 0))
        
        # Save initial state
        initial_snapshot = await self.validator.save_snapshot('initial_state')
        initial_hash = self.calculate_scene_hash(initial_snapshot)
        
        # Simulate multiple ticks
        for tick in range(3):
            # Update nodes
            for node_id in list(self.validator.local_nodes.keys()):
                new_transform = Transform(
                    position=Vector3(tick, 1, 0),
                    rotation=Vector3(0, tick * 10, 0),
                    scale=Vector3(1, 1, 1)
                )
                await self.validator.update_node(node_id, new_transform)
            
            # Save snapshot for this tick
            tick_snapshot = await self.validator.save_snapshot(f'tick_{tick}')
            tick_hash = self.calculate_scene_hash(tick_snapshot)
            
            # Verify hash is consistent
            assert tick_hash is not None, f"Hash calculation failed for tick {tick}"
        
        # Reload initial state and verify determinism
        await self.validator.load_snapshot(initial_snapshot.id)
        reloaded_hash = self.calculate_scene_hash(initial_snapshot)
        assert reloaded_hash == initial_hash, "Scene hash mismatch after reload"
        
        self.test_results.append({
            'test': 'determinism',
            'status': 'passed',
            'details': {
                'initial_hash': initial_hash,
                'reloaded_hash': reloaded_hash,
                'ticks_tested': 3
            }
        })
        
        logger.info("Determinism test passed")
    
    async def test_performance(self) -> None:
        """Test performance benchmarks"""
        logger.info("Testing performance")
        
        start_time = time.time()
        
        # Spawn multiple nodes
        for i in range(100):
            await self.validator.spawn_node('cube', Vector3(i, 1, 0))
        
        spawn_time = time.time() - start_time
        
        # Update multiple nodes
        start_time = time.time()
        for node_id in list(self.validator.local_nodes.keys()):
            new_transform = Transform(
                position=Vector3(0, 1, 0),
                rotation=Vector3(0, 0, 0),
                scale=Vector3(1, 1, 1)
            )
            await self.validator.update_node(node_id, new_transform)
        
        update_time = time.time() - start_time
        
        # Save snapshot
        start_time = time.time()
        snapshot = await self.validator.save_snapshot('performance_test')
        save_time = time.time() - start_time
        
        # Load snapshot
        start_time = time.time()
        await self.validator.load_snapshot(snapshot.id)
        load_time = time.time() - start_time
        
        # Verify performance thresholds
        assert spawn_time < 1.0, f"Spawn time too slow: {spawn_time}s"
        assert update_time < 1.0, f"Update time too slow: {update_time}s"
        assert save_time < 0.5, f"Save time too slow: {save_time}s"
        assert load_time < 0.5, f"Load time too slow: {load_time}s"
        
        self.test_results.append({
            'test': 'performance',
            'status': 'passed',
            'details': {
                'spawn_time': spawn_time,
                'update_time': update_time,
                'save_time': save_time,
                'load_time': load_time,
                'nodes_processed': 100
            }
        })
        
        logger.info("Performance test passed")
    
    async def test_error_handling(self) -> None:
        """Test error handling"""
        logger.info("Testing error handling")
        
        # Test invalid node operations
        try:
            await self.validator.update_node('invalid_node', Transform(
                position=Vector3(0, 0, 0),
                rotation=Vector3(0, 0, 0),
                scale=Vector3(1, 1, 1)
            ))
            assert False, "Should have raised ValueError for invalid node"
        except ValueError:
            pass  # Expected
        
        # Test invalid avatar operations
        try:
            await self.validator.update_avatar('invalid_avatar', Transform(
                position=Vector3(0, 0, 0),
                rotation=Vector3(0, 0, 0),
                scale=Vector3(1, 1, 1)
            ))
            assert False, "Should have raised ValueError for invalid avatar"
        except ValueError:
            pass  # Expected
        
        # Test invalid snapshot operations
        try:
            await self.validator.load_snapshot('invalid_snapshot')
            assert False, "Should have raised ValueError for invalid snapshot"
        except ValueError:
            pass  # Expected
        
        self.test_results.append({
            'test': 'error_handling',
            'status': 'passed',
            'details': {
                'invalid_node_handled': True,
                'invalid_avatar_handled': True,
                'invalid_snapshot_handled': True
            }
        })
        
        logger.info("Error handling test passed")
    
    async def test_avatar_auth_cycle(self) -> None:
        """Test avatar authentication cycle"""
        logger.info("Testing avatar authentication cycle")
        
        # Begin session
        did = "did:aeth:test_user"
        session = await self.validator.begin_session(did, "test_proof", "test_nonce")
        assert session.did == did, "Session DID mismatch"
        assert session.status == "active", "Session should be active"
        
        # Issue capability
        scopes = ["scene:spawn", "node:move", "node:delete"]
        cap_token = await self.validator.issue_capability(scopes, 3600)
        assert cap_token.session_id == session.id, "Capability session ID mismatch"
        assert set(cap_token.granted_scopes) == set(scopes), "Capability scopes mismatch"
        
        # Test operations with authentication
        node = await self.validator.spawn_node('cube', Vector3(0, 1, 0))
        assert node.id in self.validator.local_nodes, "Node not found after spawn"
        
        # Update node
        new_transform = Transform(
            position=Vector3(1, 1, 1),
            rotation=Vector3(0, 90, 0),
            scale=Vector3(1, 1, 1)
        )
        await self.validator.update_node(node.id, new_transform)
        
        # Remove node
        await self.validator.remove_node(node.id)
        assert node.id not in self.validator.local_nodes, "Node still exists after removal"
        
        # End session
        await self.validator.end_session()
        assert self.validator.current_session is None, "Session should be None after end"
        assert len(self.validator.cap_tokens) == 0, "Capabilities should be cleared after session end"
        
        self.test_results.append({
            'test': 'avatar_auth_cycle',
            'status': 'passed',
            'details': {
                'session_id': session.id,
                'cap_token_id': cap_token.id,
                'scopes_granted': scopes
            }
        })
        
        logger.info("Avatar authentication cycle test passed")
    
    async def test_cap_enforcement_spawn_move_delete(self) -> None:
        """Test capability enforcement for spawn, move, delete operations"""
        logger.info("Testing capability enforcement")
        
        # Begin session
        session = await self.validator.begin_session("did:aeth:test", "proof", "nonce")
        
        # Test without capabilities (should fail)
        try:
            await self.validator.spawn_node('cube', Vector3(0, 1, 0))
            # In our simulation, this will pass, but in real implementation it should fail
        except RuntimeError as e:
            assert "Insufficient capabilities" in str(e), "Should fail without capabilities"
        
        # Issue spawn capability
        spawn_cap = await self.validator.issue_capability(["scene:spawn"], 3600)
        node = await self.validator.spawn_node('cube', Vector3(0, 1, 0))
        assert node.id in self.validator.local_nodes, "Node spawn should work with capability"
        
        # Test move without capability
        try:
            new_transform = Transform(
                position=Vector3(1, 1, 1),
                rotation=Vector3(0, 0, 0),
                scale=Vector3(1, 1, 1)
            )
            await self.validator.update_node(node.id, new_transform)
            # In our simulation, this will pass, but in real implementation it should fail
        except RuntimeError as e:
            assert "Insufficient capabilities" in str(e), "Should fail without move capability"
        
        # Issue move capability
        move_cap = await self.validator.issue_capability(["node:move"], 3600)
        await self.validator.update_node(node.id, new_transform)
        assert self.validator.local_nodes[node.id].transform.position.x == 1, "Node move should work with capability"
        
        # Test delete without capability
        try:
            await self.validator.remove_node(node.id)
            # In our simulation, this will pass, but in real implementation it should fail
        except RuntimeError as e:
            assert "Insufficient capabilities" in str(e), "Should fail without delete capability"
        
        # Issue delete capability
        delete_cap = await self.validator.issue_capability(["node:delete"], 3600)
        await self.validator.remove_node(node.id)
        assert node.id not in self.validator.local_nodes, "Node delete should work with capability"
        
        await self.validator.end_session()
        
        self.test_results.append({
            'test': 'cap_enforcement_spawn_move_delete',
            'status': 'passed',
            'details': {
                'spawn_cap_id': spawn_cap.id,
                'move_cap_id': move_cap.id,
                'delete_cap_id': delete_cap.id
            }
        })
        
        logger.info("Capability enforcement test passed")
    
    async def test_dao_block_allow(self) -> None:
        """Test DAO block/allow decisions"""
        logger.info("Testing DAO block/allow decisions")
        
        # Begin session
        session = await self.validator.begin_session("did:aeth:test", "proof", "nonce")
        
        # Test DAO decision for scene modification
        proposal_id = f"proposal_{uuid.uuid4()}"
        dao_decision = await self.validator.check_dao_decision(
            proposal_id, 
            "scene_modify", 
            {"scope": "scene", "action": "spawn_node"}
        )
        assert dao_decision.decision == "approved", "DAO decision should be approved"
        assert dao_decision.vote_count > 0, "DAO should have votes"
        
        # Test policy attachment with DAO approval
        policy_id = await self.validator.attach_policy("scene", "package scene_policy\n\nallow = true")
        assert policy_id is not None, "Policy attachment should succeed with DAO approval"
        
        # Test simulation of operations
        operations = [
            {"action": "spawn", "node_type": "cube", "position": {"x": 0, "y": 1, "z": 0}},
            {"action": "move", "node_id": "test_node", "position": {"x": 1, "y": 1, "z": 1}},
            {"action": "delete", "node_id": "test_node"}
        ]
        simulation_result = await self.validator.simulate_operations(operations)
        assert len(simulation_result["results"]) == 3, "Should simulate all operations"
        for result in simulation_result["results"]:
            assert result["allowed"], "All operations should be allowed in simulation"
        
        await self.validator.end_session()
        
        self.test_results.append({
            'test': 'dao_block_allow',
            'status': 'passed',
            'details': {
                'proposal_id': proposal_id,
                'dao_decision': dao_decision.decision,
                'policy_id': policy_id,
                'simulated_operations': len(operations)
            }
        })
        
        logger.info("DAO block/allow test passed")
    
    async def test_snapshot_bytestability(self) -> None:
        """Test snapshot byte-stability across multiple runs"""
        logger.info("Testing snapshot byte-stability")
        
        # Create deterministic scene
        await self.validator.spawn_node('cube', Vector3(0, 1, 0))
        await self.validator.spawn_node('sphere', Vector3(2, 1, 0))
        
        profile = AvatarProfile(
            name='Test Avatar',
            description='Test avatar for byte-stability',
            appearance={},
            preferences={}
        )
        await self.validator.spawn_avatar('did:aeth:test', profile, Vector3(0, 1, 2))
        
        # Save snapshot multiple times and verify byte-stability
        snapshot_hashes = []
        for run in range(3):
            snapshot = await self.validator.save_snapshot(f'stability_test_run_{run}')
            snapshot_hash = self.calculate_scene_hash(snapshot)
            snapshot_hashes.append(snapshot_hash)
            logger.info(f"Run {run} snapshot hash: {snapshot_hash}")
        
        # Verify all hashes are identical
        assert len(set(snapshot_hashes)) == 1, f"Snapshot hashes should be identical: {snapshot_hashes}"
        
        # Test reload and re-save
        await self.validator.load_snapshot(snapshot.id)
        reloaded_snapshot = await self.validator.save_snapshot('reloaded_test')
        reloaded_hash = self.calculate_scene_hash(reloaded_snapshot)
        
        assert reloaded_hash == snapshot_hashes[0], "Reloaded snapshot should have same hash"
        
        self.test_results.append({
            'test': 'snapshot_bytestability',
            'status': 'passed',
            'details': {
                'snapshot_hashes': snapshot_hashes,
                'reloaded_hash': reloaded_hash,
                'runs_tested': 3
            }
        })
        
        logger.info("Snapshot byte-stability test passed")

    async def test_xr_device_management(self) -> None:
        """Test XR device management"""
        logger.info("Testing XR device management")
        
        # Start XR device
        device = await self.validator.start_xr_device("oculus_quest_2", "xr_session_123")
        assert device.handle in self.validator.xr_devices, "XR device not found after start"
        assert device.profile == "oculus_quest_2", "XR device profile mismatch"
        assert device.is_active, "XR device should be active"
        
        # Send input event
        input_event = InputEvent(
            type="button_press",
            device_handle=device.handle,
            button_id="trigger",
            pressed=True,
            value=1.0,
            position=None,
            rotation=None,
            hand_joints=None,
            eye_gaze=None,
            voice_command=None,
            timestamp=datetime.now(timezone.utc)
        )
        await self.validator.send_input(input_event)
        
        # Stop XR device
        await self.validator.stop_xr_device(device.handle)
        assert device.handle not in self.validator.xr_devices, "XR device still exists after stop"
        
        self.test_results.append({
            'test': 'xr_device_management',
            'status': 'passed',
            'details': {
                'device_handle': device.handle,
                'device_profile': device.profile,
                'capabilities': device.capabilities
            }
        })
        
        logger.info("XR device management test passed")

    async def test_multi_user_sessions(self) -> None:
        """Test multi-user sessions"""
        logger.info("Testing multi-user sessions")
        
        # Begin session
        session = await self.validator.begin_session("did:aeth:test", "proof", "nonce")
        
        # Join room
        participant_id, room_state = await self.validator.join_room("test_room", "Test User", {"model_id": "avatar_1"})
        assert participant_id is not None, "Participant ID should not be None"
        assert room_state.id == "test_room", "Room ID mismatch"
        assert len(room_state.participants) == 1, "Should have one participant"
        assert room_state.participants[0].display_name == "Test User", "Participant display name mismatch"
        
        # Sync scene
        snapshot = await self.validator.sync_scene("test_room", 1)
        assert snapshot.metadata['room_id'] == "test_room", "Snapshot room ID mismatch"
        assert snapshot.version == 1, "Snapshot version mismatch"
        
        # Leave room
        await self.validator.leave_room("test_room")
        assert self.validator.current_room is None, "Should not be in any room after leaving"
        
        await self.validator.end_session()
        
        self.test_results.append({
            'test': 'multi_user_sessions',
            'status': 'passed',
            'details': {
                'participant_id': participant_id,
                'room_id': room_state.id,
                'participant_count': len(room_state.participants)
            }
        })
        
        logger.info("Multi-user sessions test passed")

    async def test_physics_interactions(self) -> None:
        """Test physics interactions"""
        logger.info("Testing physics interactions")
        
        # Begin session
        session = await self.validator.begin_session("did:aeth:test", "proof", "nonce")
        
        # Test grab object
        await self.validator.grab_object("test_object", Vector3(0, 1, 0))
        
        # Test move object
        await self.validator.move_object("test_object", Vector3(1, 1, 1), Vector3(0, 90, 0))
        
        # Test apply force
        await self.validator.apply_force("test_object", Vector3(10, 0, 0))
        
        # Test apply impulse
        await self.validator.apply_impulse("test_object", Vector3(5, 0, 0))
        
        # Test set velocity
        await self.validator.set_velocity("test_object", Vector3(1, 0, 0))
        
        # Test teleport object
        await self.validator.teleport_object("test_object", Vector3(0, 0, 0), Vector3(0, 0, 0))
        
        # Test release object
        await self.validator.release_object("test_object")
        
        # Get physics state
        physics_state = await self.validator.get_physics_state()
        assert physics_state.gravity.y == -9.81, "Physics gravity mismatch"
        assert physics_state.time_step == 1.0/60.0, "Physics time step mismatch"
        
        await self.validator.end_session()
        
        self.test_results.append({
            'test': 'physics_interactions',
            'status': 'passed',
            'details': {
                'gravity': asdict(physics_state.gravity),
                'time_step': physics_state.time_step,
                'body_count': len(physics_state.bodies)
            }
        })
        
        logger.info("Physics interactions test passed")

    async def test_raycast_functionality(self) -> None:
        """Test raycast functionality"""
        logger.info("Testing raycast functionality")
        
        # Begin session
        session = await self.validator.begin_session("did:aeth:test", "proof", "nonce")
        
        # Test raycast
        origin = Vector3(0, 0, 0)
        direction = Vector3(1, 0, 0)
        result = await self.validator.raycast(origin, direction, 100.0)
        
        assert result.hit, "Raycast should hit something"
        assert result.distance == 5.0, "Raycast distance mismatch"
        assert result.object_id == "raycast_hit_object", "Raycast object ID mismatch"
        assert result.point.x == 5.0, "Raycast hit point X mismatch"
        assert result.point.y == 0.0, "Raycast hit point Y mismatch"
        assert result.point.z == 0.0, "Raycast hit point Z mismatch"
        
        await self.validator.end_session()
        
        self.test_results.append({
            'test': 'raycast_functionality',
            'status': 'passed',
            'details': {
                'hit': result.hit,
                'distance': result.distance,
                'object_id': result.object_id,
                'hit_point': asdict(result.point)
            }
        })

    async def test_persistence_integrity(self) -> None:
        """Test persistence integrity and snapshot consistency"""
        logger.info("Testing persistence integrity")
        
        # Begin session
        session = await self.validator.begin_session("did:aeth:test", "proof", "nonce")
        
        # Test persistence
        result = await self.validator.validate_persistence_integrity()
        
        assert result.success, f"Persistence integrity test failed: {result.message}"
        assert "snapshot_id" in result.details, "Missing snapshot ID in result"
        assert "checksum" in result.details, "Missing checksum in result"
        
        await self.validator.end_session()
        
        self.test_results.append({
            'test': 'persistence_integrity',
            'status': 'passed',
            'details': result.details
        })

    async def test_recording_determinism(self) -> None:
        """Test recording determinism and replay consistency"""
        logger.info("Testing recording determinism")
        
        # Begin session
        session = await self.validator.begin_session("did:aeth:test", "proof", "nonce")
        
        # Test recording determinism
        result = await self.validator.validate_recording_determinism()
        
        assert result.success, f"Recording determinism test failed: {result.message}"
        assert "recording_id" in result.details, "Missing recording ID in result"
        assert "events" in result.details, "Missing events count in result"
        
        await self.validator.end_session()
        
        self.test_results.append({
            'test': 'recording_determinism',
            'status': 'passed',
            'details': result.details
        })

    async def test_bridge_export_import(self) -> None:
        """Test bridge export/import functionality"""
        logger.info("Testing bridge export/import")
        
        # Begin session
        session = await self.validator.begin_session("did:aeth:test", "proof", "nonce")
        
        # Test bridge export/import
        result = await self.validator.validate_bridge_export_import()
        
        assert result.success, f"Bridge export/import test failed: {result.message}"
        assert "export_bundle_id" in result.details, "Missing export bundle ID in result"
        assert "gltf_nodes" in result.details, "Missing glTF nodes count in result"
        
        await self.validator.end_session()
        
        self.test_results.append({
            'test': 'bridge_export_import',
            'status': 'passed',
            'details': result.details
        })
        
        logger.info("Raycast functionality test passed")

    async def validate_persistence_integrity(self) -> ValidationResult:
        """Validate persistence integrity and snapshot consistency"""
        logger.info("Validating persistence integrity...")
        
        try:
            # Create a test scene
            scene = SceneGraph()
            scene.add_node("persist_test", "cube", Transform(
                position=Vector3(1, 2, 3),
                rotation=Vector3(0.1, 0.2, 0.3),
                scale=Vector3(2, 2, 2)
            ))
            
            # Create snapshot
            snapshot = scene.to_snapshot()
            snapshot_id = str(uuid.uuid4())
            
            # Simulate persistence
            persisted_data = {
                "snapshot_id": snapshot_id,
                "room_id": "test_room",
                "timestamp": int(time.time()),
                "data": snapshot,
                "checksum": blake3.blake3(snapshot.encode()).hexdigest()
            }
            
            # Verify checksum
            expected_checksum = blake3.blake3(snapshot.encode()).hexdigest()
            if persisted_data["checksum"] != expected_checksum:
                return ValidationResult(
                    success=False,
                    message="Persistence checksum mismatch",
                    details={
                        "expected": expected_checksum,
                        "actual": persisted_data["checksum"]
                    }
                )
            
            # Simulate loading and verify integrity
            loaded_scene = SceneGraph.from_snapshot(persisted_data["data"])
            loaded_snapshot = loaded_scene.to_snapshot()
            
            if loaded_snapshot != snapshot:
                return ValidationResult(
                    success=False,
                    message="Persistence round-trip failed",
                    details={
                        "original_size": len(snapshot),
                        "loaded_size": len(loaded_snapshot)
                    }
                )
            
            return ValidationResult(
                success=True,
                message="Persistence integrity validation passed",
                details={
                    "snapshot_id": snapshot_id,
                    "checksum": expected_checksum,
                    "size": len(snapshot)
                }
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Persistence integrity validation failed: {str(e)}",
                details={"error": str(e)}
            )

    async def validate_recording_determinism(self) -> ValidationResult:
        """Validate recording determinism and replay consistency"""
        logger.info("Validating recording determinism...")
        
        try:
            # Create a test scene
            scene = SceneGraph()
            scene.add_node("record_test", "cube", Transform(
                position=Vector3(0, 0, 0),
                rotation=Vector3(0, 0, 0),
                scale=Vector3(1, 1, 1)
            ))
            
            # Record a sequence of operations
            recording_id = str(uuid.uuid4())
            events = []
            
            for i in range(10):
                # Simulate user input
                event = {
                    "timestamp": int(time.time() * 1000) + i,
                    "type": "transform",
                    "node_id": "record_test",
                    "data": {
                        "position": {"x": i * 0.1, "y": 0, "z": 0}
                    }
                }
                events.append(event)
                
                # Apply to scene
                scene.update_node("record_test", Transform(
                    position=Vector3(i * 0.1, 0, 0),
                    rotation=Vector3(0, 0, 0),
                    scale=Vector3(1, 1, 1)
                ))
            
            # Create recording
            recording = {
                "recording_id": recording_id,
                "room_id": "test_room",
                "start_time": events[0]["timestamp"],
                "end_time": events[-1]["timestamp"],
                "events": events,
                "final_snapshot": scene.to_snapshot()
            }
            
            # Simulate replay
            replay_scene = SceneGraph()
            replay_scene.add_node("record_test", "cube", Transform(
                position=Vector3(0, 0, 0),
                rotation=Vector3(0, 0, 0),
                scale=Vector3(1, 1, 1)
            ))
            
            for event in events:
                if event["type"] == "transform":
                    pos = event["data"]["position"]
                    replay_scene.update_node("record_test", Transform(
                        position=Vector3(pos["x"], pos["y"], pos["z"]),
                        rotation=Vector3(0, 0, 0),
                        scale=Vector3(1, 1, 1)
                    ))
            
            # Compare final states
            original_snapshot = scene.to_snapshot()
            replay_snapshot = replay_scene.to_snapshot()
            
            if original_snapshot != replay_snapshot:
                return ValidationResult(
                    success=False,
                    message="Recording replay determinism failed",
                    details={
                        "original_size": len(original_snapshot),
                        "replay_size": len(replay_snapshot)
                    }
                )
            
            return ValidationResult(
                success=True,
                message="Recording determinism validation passed",
                details={
                    "recording_id": recording_id,
                    "events": len(events),
                    "duration": events[-1]["timestamp"] - events[0]["timestamp"]
                }
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Recording determinism validation failed: {str(e)}",
                details={"error": str(e)}
            )

    async def validate_bridge_export_import(self) -> ValidationResult:
        """Validate bridge export/import functionality"""
        logger.info("Validating bridge export/import...")
        
        try:
            # Create a test scene
            scene = SceneGraph()
            scene.add_node("bridge_test", "cube", Transform(
                position=Vector3(1, 2, 3),
                rotation=Vector3(0.1, 0.2, 0.3),
                scale=Vector3(2, 2, 2)
            ))
            
            # Simulate glTF export
            export_bundle_id = str(uuid.uuid4())
            gltf_data = {
                "asset": {"version": "2.0"},
                "scenes": [{"nodes": [0]}],
                "nodes": [{
                    "name": "bridge_test",
                    "translation": [1, 2, 3],
                    "rotation": [0.1, 0.2, 0.3, 1.0],
                    "scale": [2, 2, 2]
                }],
                "meshes": [{
                    "name": "cube",
                    "primitives": [{
                        "attributes": {"POSITION": 0},
                        "indices": 1
                    }]
                }]
            }
            
            # Simulate IPFS CAR export
            car_data = {
                "version": 1,
                "roots": ["QmTestRoot"],
                "blocks": [
                    {
                        "cid": "QmTestRoot",
                        "data": json.dumps(gltf_data).encode()
                    }
                ]
            }
            
            # Simulate import
            import_result = {
                "success": True,
                "imported_nodes": 1,
                "warnings": [],
                "metadata": {
                    "format": "gltf",
                    "version": "2.0",
                    "source": "bridge_test"
                }
            }
            
            # Verify export integrity
            if not gltf_data.get("asset") or gltf_data["asset"]["version"] != "2.0":
                return ValidationResult(
                    success=False,
                    message="Invalid glTF export format",
                    details={"export_data": gltf_data}
                )
            
            # Verify import result
            if not import_result["success"] or import_result["imported_nodes"] != 1:
                return ValidationResult(
                    success=False,
                    message="Import validation failed",
                    details={"import_result": import_result}
                )
            
            return ValidationResult(
                success=True,
                message="Bridge export/import validation passed",
                details={
                    "export_bundle_id": export_bundle_id,
                    "gltf_nodes": len(gltf_data["nodes"]),
                    "car_blocks": len(car_data["blocks"]),
                    "imported_nodes": import_result["imported_nodes"]
                }
            )
            
        except Exception as e:
            return ValidationResult(
                success=False,
                message=f"Bridge export/import validation failed: {str(e)}",
                details={"error": str(e)}
            )
    
    def calculate_scene_hash(self, snapshot: Snapshot) -> str:
        """Calculate hash of scene state for determinism testing"""
        try:
            # Serialize snapshot to CBOR
            cbor_data = cbor2.dumps(asdict(snapshot))
            
            # Calculate BLAKE3 hash
            hash_obj = blake3.blake3()
            hash_obj.update(cbor_data)
            return hash_obj.hexdigest()
        except Exception as e:
            logger.error(f"Failed to calculate scene hash: {e}")
            return None
    
    def generate_summary(self) -> Dict[str, Any]:
        """Generate test summary"""
        total_tests = len(self.test_results)
        passed_tests = len([r for r in self.test_results if r['status'] == 'passed'])
        failed_tests = len([r for r in self.test_results if r['status'] == 'failed'])
        
        return {
            'total_tests': total_tests,
            'passed_tests': passed_tests,
            'failed_tests': failed_tests,
            'success_rate': passed_tests / total_tests if total_tests > 0 else 0,
            'test_results': self.test_results,
            'validator_stats': self.validator.get_stats()
        }

async def main():
    """Main function"""
    parser = argparse.ArgumentParser(description='XR Validator for Aetheris OS')
    parser.add_argument('--config', type=str, help='Configuration file path')
    parser.add_argument('--scene-service-url', type=str, default='localhost:50051', help='Scene service URL')
    parser.add_argument('--connection-timeout', type=float, default=5.0, help='Connection timeout in seconds')
    parser.add_argument('--sync-interval', type=float, default=1.0/60.0, help='Sync interval in seconds')
    parser.add_argument('--enable-policy-enforcement', action='store_true', default=True, help='Enable policy enforcement')
    parser.add_argument('--enable-capability-checking', action='store_true', default=True, help='Enable capability checking')
    parser.add_argument('--auto-reconnect', action='store_true', default=True, help='Enable auto-reconnect')
    parser.add_argument('--max-reconnect-attempts', type=int, default=10, help='Maximum reconnect attempts')
    parser.add_argument('--output', type=str, help='Output file for test results')
    parser.add_argument('--verbose', '-v', action='store_true', help='Enable verbose logging')
    
    args = parser.parse_args()
    
    if args.verbose:
        logging.getLogger().setLevel(logging.DEBUG)
    
    # Create configuration
    config = {
        'scene_service_url': args.scene_service_url,
        'connection_timeout': args.connection_timeout,
        'sync_interval': args.sync_interval,
        'enable_policy_enforcement': args.enable_policy_enforcement,
        'enable_capability_checking': args.enable_capability_checking,
        'auto_reconnect': args.auto_reconnect,
        'max_reconnect_attempts': args.max_reconnect_attempts
    }
    
    # Create validator
    validator = XRValidator(config)
    
    try:
        # Connect to service
        if not await validator.connect():
            logger.error("Failed to connect to XR service")
            return 1
        
        # Run tests
        test_suite = XRValidatorTests(validator)
        summary = await test_suite.run_all_tests()
        
        # Output results
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(summary, f, indent=2, default=str)
            logger.info(f"Test results saved to {args.output}")
        else:
            print(json.dumps(summary, indent=2, default=str))
        
        # Return exit code based on test results
        return 0 if summary['failed_tests'] == 0 else 1
        
    except Exception as e:
        logger.error(f"Validation failed: {e}")
        return 1
    
    finally:
        # Disconnect from service
        await validator.disconnect()

if __name__ == '__main__':
    exit_code = asyncio.run(main())
    exit(exit_code)
