extends Node

# AetherisBus - Global communication bus for Aetheris XR integration

signal service_connected
signal service_disconnected
signal service_error(error_message: String)
signal node_event(event_type: String, node_id: String, data: Dictionary)
signal avatar_event(event_type: String, avatar_id: String, data: Dictionary)
signal policy_event(event_type: String, action: String, data: Dictionary)
signal snapshot_event(event_type: String, snapshot_id: String, data: Dictionary)

var scene_service_connected = false
var scene_service_url = "localhost:50051"  # Default gRPC endpoint
var connection_timeout = 5.0
var reconnect_interval = 1.0
var max_reconnect_attempts = 10
var reconnect_attempts = 0

# Scene service client (would be the actual gRPC client)
var scene_service_client = null

func _ready():
	print("AetherisBus initializing...")
	
	# Set up connection monitoring
	_setup_connection_monitoring()

func _setup_connection_monitoring():
	# This would set up actual connection monitoring
	# For now, we'll simulate it
	pass

func connect_to_service():
	print("Connecting to Aetheris Scene Service at: ", scene_service_url)
	
	# This would establish the actual connection
	# For now, we'll simulate success
	scene_service_connected = true
	reconnect_attempts = 0
	service_connected.emit()
	print("Connected to Aetheris Scene Service")

func disconnect_from_service():
	print("Disconnecting from Aetheris Scene Service")
	
	scene_service_connected = false
	service_disconnected.emit()
	print("Disconnected from Aetheris Scene Service")

func is_connected() -> bool:
	return scene_service_connected

func set_service_url(url: String):
	scene_service_url = url
	print("Scene service URL set to: ", url)

func send_scene_request(request_type: String, data: Dictionary) -> Dictionary:
	if not scene_service_connected:
		service_error.emit("Scene service not connected")
		return {"error": "Service not connected"}
	
	# This would send the actual request to the scene service
	# For now, we'll simulate the response
	match request_type:
		"spawn_node":
			return _handle_spawn_node_request(data)
		"update_node":
			return _handle_update_node_request(data)
		"remove_node":
			return _handle_remove_node_request(data)
		"spawn_avatar":
			return _handle_spawn_avatar_request(data)
		"update_avatar":
			return _handle_update_avatar_request(data)
		"remove_avatar":
			return _handle_remove_avatar_request(data)
		"save_snapshot":
			return _handle_save_snapshot_request(data)
		"load_snapshot":
			return _handle_load_snapshot_request(data)
		"get_scene_state":
			return _handle_get_scene_state_request(data)
		_:
			return {"error": "Unknown request type: " + request_type}

func _handle_spawn_node_request(data: Dictionary) -> Dictionary:
	var node_id = "node_" + str(randi())
	var response = {
		"success": true,
		"node_id": node_id,
		"position": data.get("position", Vector3.ZERO)
	}
	
	# Emit node event
	node_event.emit("spawned", node_id, response)
	
	return response

func _handle_update_node_request(data: Dictionary) -> Dictionary:
	var node_id = data.get("node_id", "")
	if node_id.is_empty():
		return {"error": "Node ID required"}
	
	var response = {
		"success": true,
		"node_id": node_id,
		"position": data.get("position", Vector3.ZERO),
		"rotation": data.get("rotation", Vector3.ZERO),
		"scale": data.get("scale", Vector3.ONE)
	}
	
	# Emit node event
	node_event.emit("updated", node_id, response)
	
	return response

func _handle_remove_node_request(data: Dictionary) -> Dictionary:
	var node_id = data.get("node_id", "")
	if node_id.is_empty():
		return {"error": "Node ID required"}
	
	var response = {
		"success": true,
		"node_id": node_id
	}
	
	# Emit node event
	node_event.emit("removed", node_id, response)
	
	return response

func _handle_spawn_avatar_request(data: Dictionary) -> Dictionary:
	var avatar_id = "avatar_" + str(randi())
	var did = data.get("did", "")
	
	var response = {
		"success": true,
		"avatar_id": avatar_id,
		"did": did,
		"position": data.get("position", Vector3.ZERO)
	}
	
	# Emit avatar event
	avatar_event.emit("spawned", avatar_id, response)
	
	return response

func _handle_update_avatar_request(data: Dictionary) -> Dictionary:
	var avatar_id = data.get("avatar_id", "")
	if avatar_id.is_empty():
		return {"error": "Avatar ID required"}
	
	var response = {
		"success": true,
		"avatar_id": avatar_id,
		"position": data.get("position", Vector3.ZERO),
		"rotation": data.get("rotation", Vector3.ZERO)
	}
	
	# Emit avatar event
	avatar_event.emit("updated", avatar_id, response)
	
	return response

func _handle_remove_avatar_request(data: Dictionary) -> Dictionary:
	var avatar_id = data.get("avatar_id", "")
	if avatar_id.is_empty():
		return {"error": "Avatar ID required"}
	
	var response = {
		"success": true,
		"avatar_id": avatar_id
	}
	
	# Emit avatar event
	avatar_event.emit("removed", avatar_id, response)
	
	return response

func _handle_save_snapshot_request(data: Dictionary) -> Dictionary:
	var snapshot_id = "snapshot_" + str(randi())
	var label = data.get("label", "unnamed")
	
	var response = {
		"success": true,
		"snapshot_id": snapshot_id,
		"label": label
	}
	
	# Emit snapshot event
	snapshot_event.emit("saved", snapshot_id, response)
	
	return response

func _handle_load_snapshot_request(data: Dictionary) -> Dictionary:
	var snapshot_id = data.get("snapshot_id", "")
	if snapshot_id.is_empty():
		return {"error": "Snapshot ID required"}
	
	var response = {
		"success": true,
		"snapshot_id": snapshot_id
	}
	
	# Emit snapshot event
	snapshot_event.emit("loaded", snapshot_id, response)
	
	return response

func _handle_get_scene_state_request(data: Dictionary) -> Dictionary:
	# This would return the current scene state
	var response = {
		"success": true,
		"nodes": [],
		"avatars": [],
		"timestamp": Time.get_unix_time_from_system()
	}
	
	return response

func subscribe_to_events(event_types: Array):
	print("Subscribing to events: ", event_types)
	# This would set up event subscriptions with the scene service
	# For now, it's a placeholder

func unsubscribe_from_events(event_types: Array):
	print("Unsubscribing from events: ", event_types)
	# This would remove event subscriptions
	# For now, it's a placeholder

func send_policy_check(action: String, context: Dictionary) -> Dictionary:
	if not scene_service_connected:
		return {"allowed": false, "reason": "Service not connected"}
	
	# This would send a policy check request
	# For now, we'll simulate approval
	var response = {
		"allowed": true,
		"action": action,
		"context": context
	}
	
	# Emit policy event
	policy_event.emit("checked", action, response)
	
	return response

func send_capability_check(capability: String, context: Dictionary) -> Dictionary:
	if not scene_service_connected:
		return {"allowed": false, "reason": "Service not connected"}
	
	# This would send a capability check request
	# For now, we'll simulate approval
	var response = {
		"allowed": true,
		"capability": capability,
		"context": context
	}
	
	return response

func get_connection_status() -> Dictionary:
	return {
		"connected": scene_service_connected,
		"url": scene_service_url,
		"reconnect_attempts": reconnect_attempts,
		"max_reconnect_attempts": max_reconnect_attempts
	}

func _on_connection_lost():
	print("Connection to scene service lost")
	scene_service_connected = false
	service_disconnected.emit()
	
	# Attempt to reconnect
	_attempt_reconnect()

func _attempt_reconnect():
	if reconnect_attempts >= max_reconnect_attempts:
		print("Max reconnect attempts reached")
		service_error.emit("Max reconnect attempts reached")
		return
	
	reconnect_attempts += 1
	print("Attempting to reconnect (", reconnect_attempts, "/", max_reconnect_attempts, ")")
	
	# Wait before attempting reconnect
	await get_tree().create_timer(reconnect_interval).timeout
	
	# Try to reconnect
	connect_to_service()

func _on_service_error(error_message: String):
	print("Scene service error: ", error_message)
	service_error.emit(error_message)
	
	# Attempt to reconnect on error
	_attempt_reconnect()
