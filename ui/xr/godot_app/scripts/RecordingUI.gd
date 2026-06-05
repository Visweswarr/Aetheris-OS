extends Control

# P4-06-A4: Recording UI for XR Persistence and Replay
# This script demonstrates the new persistence, recording, replay, and bridge functionality

@onready var persist_manager = AetherisPersist.new()
@onready var record_manager = AetherisRecord.new()
@onready var replay_manager = AetherisReplay.new()
@onready var bridge_manager = AetherisBridge.new()

@onready var session_id_label = $VBoxContainer/SessionInfo/SessionID
@onready var room_id_input = $VBoxContainer/Persistence/RoomIDInput
@onready var reason_input = $VBoxContainer/Persistence/ReasonInput
@onready var persist_button = $VBoxContainer/Persistence/PersistButton
@onready var load_snapshot_button = $VBoxContainer/Persistence/LoadSnapshotButton
@onready var snapshot_id_input = $VBoxContainer/Persistence/SnapshotIDInput

@onready var recording_room_input = $VBoxContainer/Recording/RoomIDInput
@onready var start_recording_button = $VBoxContainer/Recording/StartRecordingButton
@onready var stop_recording_button = $VBoxContainer/Recording/StopRecordingButton
@onready var recording_status_label = $VBoxContainer/Recording/StatusLabel

@onready var replay_recording_input = $VBoxContainer/Replay/RecordingIDInput
@onready var replay_mode_option = $VBoxContainer/Replay/ModeOption
@onready var start_replay_button = $VBoxContainer/Replay/StartReplayButton
@onready var stop_replay_button = $VBoxContainer/Replay/StopReplayButton
@onready var replay_status_label = $VBoxContainer/Replay/StatusLabel

@onready var export_room_input = $VBoxContainer/Bridge/ExportRoomInput
@onready var export_formats_option = $VBoxContainer/Bridge/ExportFormatsOption
@onready var export_button = $VBoxContainer/Bridge/ExportButton
@onready var import_file_input = $VBoxContainer/Bridge/ImportFileInput
@onready var import_button = $VBoxContainer/Bridge/ImportButton
@onready var publish_room_input = $VBoxContainer/Bridge/PublishRoomInput
@onready var publish_button = $VBoxContainer/Bridge/PublishButton

@onready var status_label = $VBoxContainer/StatusLabel

var current_session_id = ""
var current_cap_token = ""

func _ready():
    # Initialize with mock session
    current_session_id = "session_" + str(Time.get_unix_time_from_system())
    current_cap_token = "cap_" + str(Time.get_unix_time_from_system())
    
    # Set up managers
    persist_manager.set_session(current_session_id, current_cap_token)
    record_manager.set_session(current_session_id, current_cap_token)
    replay_manager.set_session(current_session_id, current_cap_token)
    bridge_manager.set_session(current_session_id, current_cap_token)
    
    # Update UI
    session_id_label.text = "Session: " + current_session_id
    update_status("Ready - All managers initialized")
    
    # Connect signals
    persist_button.pressed.connect(_on_persist_button_pressed)
    load_snapshot_button.pressed.connect(_on_load_snapshot_button_pressed)
    start_recording_button.pressed.connect(_on_start_recording_button_pressed)
    stop_recording_button.pressed.connect(_on_stop_recording_button_pressed)
    start_replay_button.pressed.connect(_on_start_replay_button_pressed)
    stop_replay_button.pressed.connect(_on_stop_replay_button_pressed)
    export_button.pressed.connect(_on_export_button_pressed)
    import_button.pressed.connect(_on_import_button_pressed)
    publish_button.pressed.connect(_on_publish_button_pressed)
    
    # Set up option buttons
    replay_mode_option.add_item("Headless", AetherisReplay.REPLAY_HEADLESS)
    replay_mode_option.add_item("Visualization", AetherisReplay.REPLAY_VISUALIZATION)
    replay_mode_option.add_item("Debug", AetherisReplay.REPLAY_DEBUG)
    
    export_formats_option.add_item("glTF", AetherisBridge.EXPORT_GLTF)
    export_formats_option.add_item("CAR", AetherisBridge.EXPORT_CAR)
    export_formats_option.add_item("JSON", AetherisBridge.EXPORT_JSON)
    export_formats_option.add_item("All", AetherisBridge.EXPORT_ALL)
    
    # Set default values
    room_id_input.text = "test_room"
    recording_room_input.text = "test_room"
    export_room_input.text = "test_room"
    publish_room_input.text = "test_room"
    reason_input.text = "Test persistence"
    
    # Update button states
    _update_button_states()

func _update_button_states():
    var is_recording = record_manager.is_currently_recording()
    var is_replaying = replay_manager.is_currently_replaying()
    
    start_recording_button.disabled = is_recording
    stop_recording_button.disabled = !is_recording
    start_replay_button.disabled = is_replaying
    stop_replay_button.disabled = !is_replaying
    
    recording_status_label.text = "Recording: " + ("Active" if is_recording else "Inactive")
    replay_status_label.text = "Replay: " + ("Active" if is_replaying else "Inactive")

func update_status(message: String):
    status_label.text = "Status: " + message
    print("RecordingUI: ", message)

# Persistence Functions

func _on_persist_button_pressed():
    var room_id = room_id_input.text
    var reason = reason_input.text
    
    if room_id.is_empty() or reason.is_empty():
        update_status("Error: Room ID and reason are required")
        return
    
    persist_manager.set_current_room(room_id)
    var result = persist_manager.persist_room(room_id, reason)
    
    if result.get("success", false):
        var snapshot_id = result.get("snapshot_id", "")
        var ngfs_cid = result.get("ngfs_cid", "")
        update_status("Room persisted successfully - Snapshot: " + snapshot_id + ", CID: " + ngfs_cid)
        snapshot_id_input.text = snapshot_id
    else:
        update_status("Failed to persist room: " + str(result.get("error", "Unknown error")))

func _on_load_snapshot_button_pressed():
    var snapshot_id = snapshot_id_input.text
    
    if snapshot_id.is_empty():
        update_status("Error: Snapshot ID is required")
        return
    
    var result = persist_manager.load_snapshot(snapshot_id)
    
    if result.get("success", false):
        var room_id = result.get("room_id", "")
        var nodes = result.get("nodes", [])
        var avatars = result.get("avatars", [])
        update_status("Snapshot loaded successfully - Room: " + room_id + ", Nodes: " + str(nodes.size()) + ", Avatars: " + str(avatars.size()))
    else:
        update_status("Failed to load snapshot: " + str(result.get("error", "Unknown error")))

# Recording Functions

func _on_start_recording_button_pressed():
    var room_id = recording_room_input.text
    
    if room_id.is_empty():
        update_status("Error: Room ID is required for recording")
        return
    
    record_manager.set_current_room(room_id)
    var result = record_manager.start_recording(room_id, 12345, 60.0)
    
    if result.get("success", false):
        var recording_id = result.get("recording_id", "")
        update_status("Recording started successfully - ID: " + recording_id)
    else:
        update_status("Failed to start recording: " + str(result.get("error", "Unknown error")))
    
    _update_button_states()

func _on_stop_recording_button_pressed():
    var result = record_manager.stop_recording()
    
    if result.get("success", false):
        var summary = result.get("recording_summary", {})
        var duration = summary.get("duration_seconds", 0.0)
        var event_count = summary.get("event_count", 0)
        update_status("Recording stopped successfully - Duration: " + str(duration) + "s, Events: " + str(event_count))
    else:
        update_status("Failed to stop recording: " + str(result.get("error", "Unknown error")))
    
    _update_button_states()

# Replay Functions

func _on_start_replay_button_pressed():
    var recording_id = replay_recording_input.text
    
    if recording_id.is_empty():
        update_status("Error: Recording ID is required for replay")
        return
    
    var mode = replay_mode_option.get_selected_id()
    var result = replay_manager.replay_recording(recording_id, mode, 0, 0, true)
    
    if result.get("success", false):
        var replay_id = result.get("replay_id", "")
        update_status("Replay started successfully - ID: " + replay_id)
    else:
        update_status("Failed to start replay: " + str(result.get("error", "Unknown error")))
    
    _update_button_states()

func _on_stop_replay_button_pressed():
    var success = replay_manager.stop_replay()
    
    if success:
        update_status("Replay stopped successfully")
    else:
        update_status("Failed to stop replay")
    
    _update_button_states()

# Bridge Functions

func _on_export_button_pressed():
    var room_id = export_room_input.text
    
    if room_id.is_empty():
        update_status("Error: Room ID is required for export")
        return
    
    var formats = [export_formats_option.get_selected_id()]
    var result = bridge_manager.export_scene(room_id, formats, true, true, 6)
    
    if result.get("success", false):
        var bundle_id = result.get("bundle_id", "")
        update_status("Scene exported successfully - Bundle: " + bundle_id)
    else:
        update_status("Failed to export scene: " + str(result.get("error", "Unknown error")))

func _on_import_button_pressed():
    var file_path = import_file_input.text
    
    if file_path.is_empty():
        update_status("Error: File path is required for import")
        return
    
    # In a real implementation, you would read the file and pass the data
    # For now, we'll use mock data
    var mock_data = PackedByteArray()
    mock_data.append_array("mock_bundle_data".to_utf8_buffer())
    
    var result = bridge_manager.import_scene(mock_data, AetherisBridge.IMPORT_SANDBOX, "", false)
    
    if result.get("success", false):
        var import_result = result.get("import_result", {})
        var import_id = import_result.get("import_id", "")
        var imported_nodes = import_result.get("imported_nodes", 0)
        var imported_avatars = import_result.get("imported_avatars", 0)
        update_status("Scene imported successfully - ID: " + import_id + ", Nodes: " + str(imported_nodes) + ", Avatars: " + str(imported_avatars))
    else:
        update_status("Failed to import scene: " + str(result.get("error", "Unknown error")))

func _on_publish_button_pressed():
    var room_id = publish_room_input.text
    
    if room_id.is_empty():
        update_status("Error: Room ID is required for publish")
        return
    
    var result = bridge_manager.publish_scene(room_id, true, "", true, "MIT")
    
    if result.get("success", false):
        var publish_result = result.get("publish_result", {})
        var publish_id = publish_result.get("publish_id", "")
        var public_url = publish_result.get("public_url", "")
        update_status("Scene published successfully - ID: " + publish_id + ", URL: " + str(public_url))
    else:
        update_status("Failed to publish scene: " + str(result.get("error", "Unknown error")))

# Utility Functions

func _on_timer_timeout():
    _update_button_states()
