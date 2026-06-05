extends Node3D
class_name AetherisScene

# Aetheris Scene Graph integration for Godot with DID authentication

signal scene_ready
signal scene_error(error_message: String)
signal node_spawned(node_id: String, position: Vector3)
signal node_updated(node_id: String, position: Vector3)
signal node_removed(node_id: String)
signal avatar_spawned(avatar_id: String, did: String, position: Vector3)
signal avatar_updated(avatar_id: String, position: Vector3)
signal avatar_removed(avatar_id: String)
signal policy_denied(action: String, reason: String)
signal snapshot_saved(snapshot_id: String)
signal snapshot_loaded(snapshot_id: String)
signal session_started(session_id: String, did: String)
signal session_ended(session_id: String)
signal cap_issued(cap_id: String, scopes: Array)
signal cap_expired(cap_id: String)

var scene_service_connected = false
var local_nodes = {}
var local_avatars = {}
var sync_timer = 0.0
var sync_interval = 1.0 / 60.0  # 60 Hz sync

# Authentication state
var current_session_id = ""
var current_did = ""
var current_cap_tokens = {}
var auth_module = null

func _ready():
	print("AetherisScene initializing...")
	
	# Initialize authentication module
	_init_auth_module()
	
	# Try to connect to the scene service
	_connect_to_scene_service()

func _init_auth_module():
	# Initialize the AetherisAuth module
	if ClassDB.class_exists("AetherisAuth"):
		auth_module = ClassDB.instantiate("AetherisAuth")
		if auth_module:
			# Connect to auth module signals
			auth_module.connect("session_started", _on_session_started)
			auth_module.connect("session_ended", _on_session_ended)
			auth_module.connect("cap_issued", _on_cap_issued)
			auth_module.connect("cap_expired", _on_cap_expired)
			print("AetherisAuth module initialized")
		else:
			print("Failed to instantiate AetherisAuth module")
	else:
		print("AetherisAuth class not found - running without authentication")

func _connect_to_scene_service():
	# This would connect to the actual scene service
	# For now, we'll simulate the connection
	scene_service_connected = true
	print("Connected to Aetheris Scene Service")
	scene_ready.emit()

func _process(delta):
	sync_timer += delta
	
	if sync_timer >= sync_interval:
		sync_timer = 0.0
		_sync_with_service()

func _sync_with_service():
	if not scene_service_connected:
		return
	
	# Sync local changes to service
	_push_local_changes()
	
	# Sync service changes to local
	_sync_from_service()

func _push_local_changes():
	# This would send local changes to the scene service
	# For now, it's a placeholder
	pass

func _sync_from_service():
	# This would receive updates from the scene service
	# For now, it's a placeholder
	pass

# Authentication callback methods
func _on_session_started(session_id: String, did: String):
	current_session_id = session_id
	current_did = did
	session_started.emit(session_id, did)
	print("Session started: ", session_id, " for DID: ", did)

func _on_session_ended(session_id: String):
	if current_session_id == session_id:
		current_session_id = ""
		current_did = ""
		current_cap_tokens.clear()
	session_ended.emit(session_id)
	print("Session ended: ", session_id)

func _on_cap_issued(cap_id: String, scopes: Array):
	current_cap_tokens[cap_id] = scopes
	cap_issued.emit(cap_id, scopes)
	print("Capability issued: ", cap_id, " with scopes: ", scopes)

func _on_cap_expired(cap_id: String):
	current_cap_tokens.erase(cap_id)
	cap_expired.emit(cap_id)
	print("Capability expired: ", cap_id)

# Authentication methods
func begin_session(did: String, proof: String, nonce: String) -> String:
	if not auth_module:
		scene_error.emit("Authentication module not available")
		return ""
	
	var session_id = auth_module.begin_session(did, proof, nonce)
	if session_id.is_empty():
		scene_error.emit("Failed to begin session")
		return ""
	
	return session_id

func end_session():
	if not auth_module or current_session_id.is_empty():
		scene_error.emit("No active session to end")
		return false
	
	var success = auth_module.end_session(current_session_id)
	if not success:
		scene_error.emit("Failed to end session")
		return false
	
	return true

func issue_capability(scopes: Array, ttl_seconds: int) -> String:
	if not auth_module or current_session_id.is_empty():
		scene_error.emit("No active session for capability issuance")
		return ""
	
	var cap_id = auth_module.issue_capability(current_session_id, scopes, ttl_seconds)
	if cap_id.is_empty():
		scene_error.emit("Failed to issue capability")
		return ""
	
	return cap_id

func spawn_node(node_type: String, position: Vector3, parent_id: String = "") -> String:
	var node_id = _generate_node_id()
	
	# Check if we have authentication and required capabilities
	if current_session_id.is_empty():
		scene_error.emit("No active session - cannot spawn node")
		return ""
	
	# Check for spawn capability
	var has_spawn_cap = false
	for cap_id in current_cap_tokens:
		var scopes = current_cap_tokens[cap_id]
		if "scene:spawn" in scopes or "node:spawn" in scopes:
			has_spawn_cap = true
			break
	
	if not has_spawn_cap:
		scene_error.emit("Insufficient capabilities - need scene:spawn or node:spawn")
		policy_denied.emit("spawn_node", "Insufficient capabilities")
		return ""
	
	if scene_service_connected:
		# Send to scene service with authentication
		_send_spawn_node_request(node_id, node_type, position, parent_id)
	else:
		# Create locally
		_create_local_node(node_id, node_type, position, parent_id)
	
	return node_id

func _send_spawn_node_request(node_id: String, node_type: String, position: Vector3, parent_id: String):
	# This would send a request to the scene service
	# For now, we'll create locally
	_create_local_node(node_id, node_type, position, parent_id)

func _create_local_node(node_id: String, node_type: String, position: Vector3, parent_id: String):
	var node = _create_node_by_type(node_type)
	if node:
		node.position = position
		node.name = node_id
		add_child(node)
		
		local_nodes[node_id] = node
		node_spawned.emit(node_id, position)
		print("Created local node: ", node_id, " of type: ", node_type)

func _create_node_by_type(node_type: String) -> Node3D:
	match node_type:
		"cube":
			var mesh_instance = MeshInstance3D.new()
			mesh_instance.mesh = BoxMesh.new()
			mesh_instance.material_override = StandardMaterial3D.new()
			mesh_instance.material_override.albedo_color = Color(randf(), randf(), randf(), 1.0)
			return mesh_instance
		"sphere":
			var mesh_instance = MeshInstance3D.new()
			mesh_instance.mesh = SphereMesh.new()
			mesh_instance.material_override = StandardMaterial3D.new()
			mesh_instance.material_override.albedo_color = Color(randf(), randf(), randf(), 1.0)
			return mesh_instance
		"cylinder":
			var mesh_instance = MeshInstance3D.new()
			mesh_instance.mesh = CylinderMesh.new()
			mesh_instance.material_override = StandardMaterial3D.new()
			mesh_instance.material_override.albedo_color = Color(randf(), randf(), randf(), 1.0)
			return mesh_instance
		"plane":
			var mesh_instance = MeshInstance3D.new()
			mesh_instance.mesh = PlaneMesh.new()
			mesh_instance.material_override = StandardMaterial3D.new()
			mesh_instance.material_override.albedo_color = Color(randf(), randf(), randf(), 1.0)
			return mesh_instance
		_:
			print("Unknown node type: ", node_type)
			return null

func update_node(node_id: String, position: Vector3, rotation: Vector3 = Vector3.ZERO, scale: Vector3 = Vector3.ONE):
	# Check authentication and capabilities for node updates
	if current_session_id.is_empty():
		scene_error.emit("No active session - cannot update node")
		return false
	
	# Check for move capability
	var has_move_cap = false
	for cap_id in current_cap_tokens:
		var scopes = current_cap_tokens[cap_id]
		if "node:move" in scopes or "scene:modify" in scopes:
			has_move_cap = true
			break
	
	if not has_move_cap:
		scene_error.emit("Insufficient capabilities - need node:move or scene:modify")
		policy_denied.emit("update_node", "Insufficient capabilities")
		return false
	
	if node_id in local_nodes:
		var node = local_nodes[node_id]
		node.position = position
		node.rotation = rotation
		node.scale = scale
		node_updated.emit(node_id, position)
		
		if scene_service_connected:
			_send_update_node_request(node_id, position, rotation, scale)
		return true
	
	return false

func _send_update_node_request(node_id: String, position: Vector3, rotation: Vector3, scale: Vector3):
	# This would send an update request to the scene service
	pass

func remove_node(node_id: String):
	# Check authentication and capabilities for node removal
	if current_session_id.is_empty():
		scene_error.emit("No active session - cannot remove node")
		return false
	
	# Check for delete capability
	var has_delete_cap = false
	for cap_id in current_cap_tokens:
		var scopes = current_cap_tokens[cap_id]
		if "node:delete" in scopes or "scene:modify" in scopes:
			has_delete_cap = true
			break
	
	if not has_delete_cap:
		scene_error.emit("Insufficient capabilities - need node:delete or scene:modify")
		policy_denied.emit("remove_node", "Insufficient capabilities")
		return false
	
	if node_id in local_nodes:
		var node = local_nodes[node_id]
		node.queue_free()
		local_nodes.erase(node_id)
		node_removed.emit(node_id)
		
		if scene_service_connected:
			_send_remove_node_request(node_id)
		return true
	
	return false

func _send_remove_node_request(node_id: String):
	# This would send a remove request to the scene service
	pass

func spawn_avatar(did: String, position: Vector3) -> String:
	var avatar_id = _generate_avatar_id()
	
	if scene_service_connected:
		_send_spawn_avatar_request(avatar_id, did, position)
	else:
		_create_local_avatar(avatar_id, did, position)
	
	return avatar_id

func _send_spawn_avatar_request(avatar_id: String, did: String, position: Vector3):
	# This would send a request to the scene service
	# For now, we'll create locally
	_create_local_avatar(avatar_id, did, position)

func _create_local_avatar(avatar_id: String, did: String, position: Vector3):
	var avatar = CharacterBody3D.new()
	avatar.position = position
	
	# Create avatar mesh
	var mesh_instance = MeshInstance3D.new()
	mesh_instance.mesh = CapsuleMesh.new()
	mesh_instance.material_override = StandardMaterial3D.new()
	mesh_instance.material_override.albedo_color = Color(0.2, 0.8, 0.2, 1.0)
	avatar.add_child(mesh_instance)
	
	# Add collision
	var collision = CollisionShape3D.new()
	collision.shape = CapsuleShape3D.new()
	avatar.add_child(collision)
	
	avatar.name = avatar_id
	add_child(avatar)
	
	local_avatars[avatar_id] = avatar
	avatar_spawned.emit(avatar_id, did, position)
	print("Created local avatar: ", avatar_id, " for DID: ", did)

func update_avatar(avatar_id: String, position: Vector3, rotation: Vector3 = Vector3.ZERO):
	if avatar_id in local_avatars:
		var avatar = local_avatars[avatar_id]
		avatar.position = position
		avatar.rotation = rotation
		avatar_updated.emit(avatar_id, position)
		
		if scene_service_connected:
			_send_update_avatar_request(avatar_id, position, rotation)

func _send_update_avatar_request(avatar_id: String, position: Vector3, rotation: Vector3):
	# This would send an update request to the scene service
	pass

func remove_avatar(avatar_id: String):
	if avatar_id in local_avatars:
		var avatar = local_avatars[avatar_id]
		avatar.queue_free()
		local_avatars.erase(avatar_id)
		avatar_removed.emit(avatar_id)
		
		if scene_service_connected:
			_send_remove_avatar_request(avatar_id)

func _send_remove_avatar_request(avatar_id: String):
	# This would send a remove request to the scene service
	pass

func save_snapshot(label: String) -> String:
	if not scene_service_connected:
		scene_error.emit("Scene service not connected")
		return ""
	
	var snapshot_id = _generate_snapshot_id()
	_send_save_snapshot_request(snapshot_id, label)
	return snapshot_id

func _send_save_snapshot_request(snapshot_id: String, label: String):
	# This would send a save request to the scene service
	# For now, we'll simulate success
	snapshot_saved.emit(snapshot_id)

func load_snapshot(snapshot_id: String):
	if not scene_service_connected:
		scene_error.emit("Scene service not connected")
		return
	
	_send_load_snapshot_request(snapshot_id)

func _send_load_snapshot_request(snapshot_id: String):
	# This would send a load request to the scene service
	# For now, we'll simulate success
	snapshot_loaded.emit(snapshot_id)

func set_avatar(did: String, avatar_id: String):
	if scene_service_connected:
		_send_set_avatar_request(did, avatar_id)

func _send_set_avatar_request(did: String, avatar_id: String):
	# This would send a set avatar request to the scene service
	pass

func _generate_node_id() -> String:
	return "node_" + str(randi())

func _generate_avatar_id() -> String:
	return "avatar_" + str(randi())

func _generate_snapshot_id() -> String:
	return "snapshot_" + str(randi())

func get_node_count() -> int:
	return local_nodes.size()

func get_avatar_count() -> int:
	return local_avatars.size()

func get_online_avatar_count() -> int:
	return local_avatars.size()  # All local avatars are considered online

func attach_policy(scope: String, rego_bundle: String) -> String:
	# Check authentication and capabilities for policy attachment
	if current_session_id.is_empty():
		scene_error.emit("No active session - cannot attach policy")
		return ""
	
	# Check for policy capability
	var has_policy_cap = false
	for cap_id in current_cap_tokens:
		var scopes = current_cap_tokens[cap_id]
		if "policy:attach" in scopes or "scene:admin" in scopes:
			has_policy_cap = true
			break
	
	if not has_policy_cap:
		scene_error.emit("Insufficient capabilities - need policy:attach or scene:admin")
		policy_denied.emit("attach_policy", "Insufficient capabilities")
		return ""
	
	if not auth_module:
		scene_error.emit("Authentication module not available")
		return ""
	
	var policy_id = auth_module.attach_policy(current_session_id, scope, rego_bundle)
	if policy_id.is_empty():
		scene_error.emit("Failed to attach policy")
		return ""
	
	print("Policy attached: ", policy_id, " with scope: ", scope)
	return policy_id

func simulate_operations(operations: Array) -> Dictionary:
	# Check authentication for simulation
	if current_session_id.is_empty():
		scene_error.emit("No active session - cannot simulate operations")
		return {}
	
	if not auth_module:
		scene_error.emit("Authentication module not available")
		return {}
	
	var result = auth_module.simulate_operations(current_session_id, operations)
	if result.is_empty():
		scene_error.emit("Failed to simulate operations")
		return {}
	
	print("Simulation completed: ", result)
	return result

func clear_scene():
	# Clear all local nodes and avatars
	for node_id in local_nodes.keys():
		remove_node(node_id)
	
	for avatar_id in local_avatars.keys():
		remove_avatar(avatar_id)
	
	print("Scene cleared")
