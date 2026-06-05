extends Node3D

# Main scene controller for Aetheris XR Host

@onready var aetheris_scene = $AetherisScene
@onready var camera = $AetherisScene/Camera3D

var scene_service_connected = false
var demo_mode = false

func _ready():
	print("Aetheris XR Host starting...")
	
	# Initialize the Aetheris scene
	if aetheris_scene:
		aetheris_scene.scene_ready.connect(_on_scene_ready)
		aetheris_scene.scene_error.connect(_on_scene_error)
		aetheris_scene.node_spawned.connect(_on_node_spawned)
		aetheris_scene.node_updated.connect(_on_node_updated)
		aetheris_scene.policy_denied.connect(_on_policy_denied)
	
	# Connect to the scene service
	_connect_to_scene_service()

func _connect_to_scene_service():
	print("Connecting to Aetheris Scene Service...")
	
	# Try to connect to the scene service via AetherisBus
	if AetherisBus:
		AetherisBus.connect_to_service()
		scene_service_connected = true
		print("Connected to Aetheris Scene Service")
	else:
		print("AetherisBus not available, running in demo mode")
		demo_mode = true
		_start_demo_mode()

func _start_demo_mode():
	print("Starting demo mode...")
	
	# Create some demo objects
	_create_demo_cube(Vector3(0, 1, 0))
	_create_demo_cube(Vector3(2, 1, 0))
	_create_demo_cube(Vector3(-2, 1, 0))
	
	# Create a demo avatar
	_create_demo_avatar("did:aeth:demo", Vector3(0, 1, 2))

func _create_demo_cube(position: Vector3):
	var cube = MeshInstance3D.new()
	cube.mesh = BoxMesh.new()
	cube.position = position
	
	# Add some random color
	var material = StandardMaterial3D.new()
	material.albedo_color = Color(randf(), randf(), randf(), 1.0)
	cube.material_override = material
	
	aetheris_scene.add_child(cube)
	print("Created demo cube at ", position)

func _create_demo_avatar(did: String, position: Vector3):
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
	
	aetheris_scene.add_child(avatar)
	print("Created demo avatar for DID: ", did, " at ", position)

func _on_scene_ready():
	print("Aetheris scene is ready")
	
	# Sync with the scene service
	if scene_service_connected:
		aetheris_scene.sync_from_service()

func _on_scene_error(error_message: String):
	print("Scene error: ", error_message)

func _on_node_spawned(node_id: String, position: Vector3):
	print("Node spawned: ", node_id, " at ", position)

func _on_node_updated(node_id: String, position: Vector3):
	print("Node updated: ", node_id, " at ", position)

func _on_policy_denied(action: String, reason: String):
	print("Policy denied: ", action, " - ", reason)

func _input(event):
	if event is InputEventKey and event.pressed:
		match event.keycode:
			KEY_SPACE:
				_spawn_random_cube()
			KEY_R:
				_reset_scene()
			KEY_S:
				_save_snapshot()
			KEY_L:
				_load_snapshot()
			KEY_ESCAPE:
				get_tree().quit()

func _spawn_random_cube():
	var position = Vector3(
		randf_range(-5, 5),
		1,
		randf_range(-5, 5)
	)
	
	if scene_service_connected:
		# Spawn via scene service
		aetheris_scene.spawn_node("cube", position)
	else:
		# Spawn locally in demo mode
		_create_demo_cube(position)

func _reset_scene():
	print("Resetting scene...")
	
	# Clear all children except the ground and camera
	for child in aetheris_scene.get_children():
		if child.name != "Ground" and child.name != "Camera3D" and child.name != "DirectionalLight3D":
			child.queue_free()
	
	# Restart demo mode if not connected to service
	if not scene_service_connected:
		_start_demo_mode()

func _save_snapshot():
	if scene_service_connected:
		print("Saving snapshot...")
		aetheris_scene.save_snapshot("manual_save")
	else:
		print("Demo mode: snapshot save not available")

func _load_snapshot():
	if scene_service_connected:
		print("Loading snapshot...")
		aetheris_scene.load_snapshot("manual_save")
	else:
		print("Demo mode: snapshot load not available")

func _process(delta):
	# Update camera position for demo
	if demo_mode and camera:
		var time = Time.get_time_dict_from_system()
		var seconds = time.second + time.minute * 60.0
		camera.position.x = sin(seconds * 0.1) * 3
		camera.position.z = cos(seconds * 0.1) * 3
		camera.look_at(Vector3.ZERO, Vector3.UP)
