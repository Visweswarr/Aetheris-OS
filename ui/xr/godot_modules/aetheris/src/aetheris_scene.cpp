#include "aetheris_scene.hpp"
#include <godot_cpp/classes/mesh_instance3d.hpp>
#include <godot_cpp/classes/box_mesh.hpp>
#include <godot_cpp/classes/sphere_mesh.hpp>
#include <godot_cpp/classes/cylinder_mesh.hpp>
#include <godot_cpp/classes/plane_mesh.hpp>
#include <godot_cpp/classes/capsule_mesh.hpp>
#include <godot_cpp/classes/standard_material3d.hpp>
#include <godot_cpp/classes/character_body3d.hpp>
#include <godot_cpp/classes/collision_shape3d.hpp>
#include <godot_cpp/classes/capsule_shape3d.hpp>
#include <godot_cpp/classes/utility_functions.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

void AetherisScene::_bind_methods() {
    // Scene service connection
    ClassDB::bind_method(D_METHOD("connect_to_service"), &AetherisScene::connect_to_service);
    ClassDB::bind_method(D_METHOD("disconnect_from_service"), &AetherisScene::disconnect_from_service);
    ClassDB::bind_method(D_METHOD("is_service_connected"), &AetherisScene::is_service_connected);
    ClassDB::bind_method(D_METHOD("set_service_url", "url"), &AetherisScene::set_service_url);
    ClassDB::bind_method(D_METHOD("get_service_url"), &AetherisScene::get_service_url);

    // Node operations
    ClassDB::bind_method(D_METHOD("spawn_node", "node_type", "position", "parent_id"), &AetherisScene::spawn_node, DEFVAL(""));
    ClassDB::bind_method(D_METHOD("update_node", "node_id", "position", "rotation", "scale"), &AetherisScene::update_node, DEFVAL(Vector3()), DEFVAL(Vector3(1, 1, 1)));
    ClassDB::bind_method(D_METHOD("remove_node", "node_id"), &AetherisScene::remove_node);
    ClassDB::bind_method(D_METHOD("get_nodes"), &AetherisScene::get_nodes);
    ClassDB::bind_method(D_METHOD("get_node_count"), &AetherisScene::get_node_count);

    // Avatar operations
    ClassDB::bind_method(D_METHOD("spawn_avatar", "did", "position"), &AetherisScene::spawn_avatar);
    ClassDB::bind_method(D_METHOD("update_avatar", "avatar_id", "position", "rotation"), &AetherisScene::update_avatar, DEFVAL(Vector3()));
    ClassDB::bind_method(D_METHOD("remove_avatar", "avatar_id"), &AetherisScene::remove_avatar);
    ClassDB::bind_method(D_METHOD("get_avatars"), &AetherisScene::get_avatars);
    ClassDB::bind_method(D_METHOD("get_avatar_count"), &AetherisScene::get_avatar_count);
    ClassDB::bind_method(D_METHOD("get_online_avatar_count"), &AetherisScene::get_online_avatar_count);

    // Snapshot operations
    ClassDB::bind_method(D_METHOD("save_snapshot", "label"), &AetherisScene::save_snapshot);
    ClassDB::bind_method(D_METHOD("load_snapshot", "snapshot_id"), &AetherisScene::load_snapshot);
    ClassDB::bind_method(D_METHOD("get_snapshots"), &AetherisScene::get_snapshots);

    // Scene management
    ClassDB::bind_method(D_METHOD("clear_scene"), &AetherisScene::clear_scene);
    ClassDB::bind_method(D_METHOD("reset_scene"), &AetherisScene::reset_scene);
    ClassDB::bind_method(D_METHOD("get_scene_state"), &AetherisScene::get_scene_state);

    // Sync operations
    ClassDB::bind_method(D_METHOD("sync_from_service"), &AetherisScene::sync_from_service);
    ClassDB::bind_method(D_METHOD("push_local_changes"), &AetherisScene::push_local_changes);
    ClassDB::bind_method(D_METHOD("set_sync_interval", "interval"), &AetherisScene::set_sync_interval);
    ClassDB::bind_method(D_METHOD("get_sync_interval"), &AetherisScene::get_sync_interval);

    // Properties
    ADD_PROPERTY(PropertyInfo(Variant::STRING, "service_url"), "set_service_url", "get_service_url");
    ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "sync_interval"), "set_sync_interval", "get_sync_interval");

    // Signals
    ADD_SIGNAL(MethodInfo(SIGNAL_SCENE_READY));
    ADD_SIGNAL(MethodInfo(SIGNAL_SCENE_ERROR, PropertyInfo(Variant::STRING, "error_message")));
    ADD_SIGNAL(MethodInfo(SIGNAL_NODE_SPAWNED, PropertyInfo(Variant::STRING, "node_id"), PropertyInfo(Variant::VECTOR3, "position")));
    ADD_SIGNAL(MethodInfo(SIGNAL_NODE_UPDATED, PropertyInfo(Variant::STRING, "node_id"), PropertyInfo(Variant::VECTOR3, "position")));
    ADD_SIGNAL(MethodInfo(SIGNAL_NODE_REMOVED, PropertyInfo(Variant::STRING, "node_id")));
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_SPAWNED, PropertyInfo(Variant::STRING, "avatar_id"), PropertyInfo(Variant::STRING, "did"), PropertyInfo(Variant::VECTOR3, "position")));
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_UPDATED, PropertyInfo(Variant::STRING, "avatar_id"), PropertyInfo(Variant::VECTOR3, "position")));
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_REMOVED, PropertyInfo(Variant::STRING, "avatar_id")));
    ADD_SIGNAL(MethodInfo(SIGNAL_POLICY_DENIED, PropertyInfo(Variant::STRING, "action"), PropertyInfo(Variant::STRING, "reason")));
    ADD_SIGNAL(MethodInfo(SIGNAL_SNAPSHOT_SAVED, PropertyInfo(Variant::STRING, "snapshot_id")));
    ADD_SIGNAL(MethodInfo(SIGNAL_SNAPSHOT_LOADED, PropertyInfo(Variant::STRING, "snapshot_id")));
}

AetherisScene::AetherisScene() {
    UtilityFunctions::print("AetherisScene created");
}

AetherisScene::~AetherisScene() {
    UtilityFunctions::print("AetherisScene destroyed");
}

void AetherisScene::_ready() {
    UtilityFunctions::print("AetherisScene ready");
    _connect_to_scene_service();
}

void AetherisScene::_process(double delta) {
    sync_timer += delta;
    
    if (sync_timer >= sync_interval) {
        sync_timer = 0.0;
        _sync_with_service();
    }
}

void AetherisScene::connect_to_service() {
    UtilityFunctions::print("Connecting to Aetheris Scene Service at: ", scene_service_url);
    _connect_to_scene_service();
}

void AetherisScene::disconnect_from_service() {
    UtilityFunctions::print("Disconnecting from Aetheris Scene Service");
    scene_service_connected = false;
}

bool AetherisScene::is_service_connected() const {
    return scene_service_connected;
}

void AetherisScene::set_service_url(const String& url) {
    scene_service_url = url;
}

String AetherisScene::get_service_url() const {
    return scene_service_url;
}

String AetherisScene::spawn_node(const String& node_type, const Vector3& position, const String& parent_id) {
    String node_id = _generate_node_id();
    
    if (scene_service_connected) {
        Dictionary data;
        data["node_id"] = node_id;
        data["node_type"] = node_type;
        data["position"] = position;
        data["parent_id"] = parent_id;
        
        Dictionary response = _send_scene_request("spawn_node", data);
        if (response.has("error")) {
            _emit_scene_error(response["error"]);
            return "";
        }
    } else {
        _create_local_node(node_id, node_type, position, parent_id);
    }
    
    return node_id;
}

void AetherisScene::update_node(const String& node_id, const Vector3& position, const Vector3& rotation, const Vector3& scale) {
    if (local_nodes.has(node_id)) {
        Node3D* node = Object::cast_to<Node3D>(local_nodes[node_id]);
        if (node) {
            node->set_position(position);
            node->set_rotation(rotation);
            node->set_scale(scale);
            _emit_node_updated(node_id, position);
        }
    }
    
    if (scene_service_connected) {
        Dictionary data;
        data["node_id"] = node_id;
        data["position"] = position;
        data["rotation"] = rotation;
        data["scale"] = scale;
        
        _send_scene_request("update_node", data);
    }
}

void AetherisScene::remove_node(const String& node_id) {
    if (local_nodes.has(node_id)) {
        Node3D* node = Object::cast_to<Node3D>(local_nodes[node_id]);
        if (node) {
            node->queue_free();
        }
        local_nodes.erase(node_id);
        node_count--;
        _emit_node_removed(node_id);
    }
    
    if (scene_service_connected) {
        Dictionary data;
        data["node_id"] = node_id;
        
        _send_scene_request("remove_node", data);
    }
}

Array AetherisScene::get_nodes() const {
    Array nodes;
    for (int i = 0; i < local_nodes.size(); i++) {
        nodes.append(local_nodes.keys()[i]);
    }
    return nodes;
}

int AetherisScene::get_node_count() const {
    return node_count;
}

String AetherisScene::spawn_avatar(const String& did, const Vector3& position) {
    String avatar_id = _generate_avatar_id();
    
    if (scene_service_connected) {
        Dictionary data;
        data["avatar_id"] = avatar_id;
        data["did"] = did;
        data["position"] = position;
        
        Dictionary response = _send_scene_request("spawn_avatar", data);
        if (response.has("error")) {
            _emit_scene_error(response["error"]);
            return "";
        }
    } else {
        _create_local_avatar(avatar_id, did, position);
    }
    
    return avatar_id;
}

void AetherisScene::update_avatar(const String& avatar_id, const Vector3& position, const Vector3& rotation) {
    if (local_avatars.has(avatar_id)) {
        Node3D* avatar = Object::cast_to<Node3D>(local_avatars[avatar_id]);
        if (avatar) {
            avatar->set_position(position);
            avatar->set_rotation(rotation);
            _emit_avatar_updated(avatar_id, position);
        }
    }
    
    if (scene_service_connected) {
        Dictionary data;
        data["avatar_id"] = avatar_id;
        data["position"] = position;
        data["rotation"] = rotation;
        
        _send_scene_request("update_avatar", data);
    }
}

void AetherisScene::remove_avatar(const String& avatar_id) {
    if (local_avatars.has(avatar_id)) {
        Node3D* avatar = Object::cast_to<Node3D>(local_avatars[avatar_id]);
        if (avatar) {
            avatar->queue_free();
        }
        local_avatars.erase(avatar_id);
        avatar_count--;
        online_avatar_count--;
        _emit_avatar_removed(avatar_id);
    }
    
    if (scene_service_connected) {
        Dictionary data;
        data["avatar_id"] = avatar_id;
        
        _send_scene_request("remove_avatar", data);
    }
}

Array AetherisScene::get_avatars() const {
    Array avatars;
    for (int i = 0; i < local_avatars.size(); i++) {
        avatars.append(local_avatars.keys()[i]);
    }
    return avatars;
}

int AetherisScene::get_avatar_count() const {
    return avatar_count;
}

int AetherisScene::get_online_avatar_count() const {
    return online_avatar_count;
}

String AetherisScene::save_snapshot(const String& label) {
    if (!scene_service_connected) {
        _emit_scene_error("Scene service not connected");
        return "";
    }
    
    String snapshot_id = _generate_snapshot_id();
    Dictionary data;
    data["snapshot_id"] = snapshot_id;
    data["label"] = label;
    
    Dictionary response = _send_scene_request("save_snapshot", data);
    if (response.has("error")) {
        _emit_scene_error(response["error"]);
        return "";
    }
    
    _emit_snapshot_saved(snapshot_id);
    return snapshot_id;
}

void AetherisScene::load_snapshot(const String& snapshot_id) {
    if (!scene_service_connected) {
        _emit_scene_error("Scene service not connected");
        return;
    }
    
    Dictionary data;
    data["snapshot_id"] = snapshot_id;
    
    Dictionary response = _send_scene_request("load_snapshot", data);
    if (response.has("error")) {
        _emit_scene_error(response["error"]);
        return;
    }
    
    _emit_snapshot_loaded(snapshot_id);
}

Array AetherisScene::get_snapshots() const {
    // This would return the list of available snapshots
    // For now, return an empty array
    return Array();
}

void AetherisScene::clear_scene() {
    // Clear all local nodes and avatars
    for (int i = 0; i < local_nodes.size(); i++) {
        String node_id = local_nodes.keys()[i];
        remove_node(node_id);
    }
    
    for (int i = 0; i < local_avatars.size(); i++) {
        String avatar_id = local_avatars.keys()[i];
        remove_avatar(avatar_id);
    }
    
    UtilityFunctions::print("Scene cleared");
}

void AetherisScene::reset_scene() {
    clear_scene();
    // Reset any other scene state
    UtilityFunctions::print("Scene reset");
}

Dictionary AetherisScene::get_scene_state() const {
    Dictionary state;
    state["nodes"] = local_nodes;
    state["avatars"] = local_avatars;
    state["node_count"] = node_count;
    state["avatar_count"] = avatar_count;
    state["online_avatar_count"] = online_avatar_count;
    state["service_connected"] = scene_service_connected;
    return state;
}

void AetherisScene::sync_from_service() {
    if (!scene_service_connected) {
        return;
    }
    
    // This would sync the scene state from the service
    // For now, it's a placeholder
}

void AetherisScene::push_local_changes() {
    if (!scene_service_connected) {
        return;
    }
    
    // This would push local changes to the service
    // For now, it's a placeholder
}

void AetherisScene::set_sync_interval(double interval) {
    sync_interval = interval;
}

double AetherisScene::get_sync_interval() const {
    return sync_interval;
}

void AetherisScene::_connect_to_scene_service() {
    // This would establish the actual connection to the scene service
    // For now, we'll simulate the connection
    scene_service_connected = true;
    UtilityFunctions::print("Connected to Aetheris Scene Service");
    _emit_scene_ready();
}

void AetherisScene::_sync_with_service() {
    if (!scene_service_connected) {
        return;
    }
    
    push_local_changes();
    sync_from_service();
}

void AetherisScene::_create_local_node(const String& node_id, const String& node_type, const Vector3& position, const String& parent_id) {
    MeshInstance3D* mesh_instance = nullptr;
    
    if (node_type == "cube") {
        mesh_instance = memnew(MeshInstance3D);
        mesh_instance->set_mesh(memnew(BoxMesh));
    } else if (node_type == "sphere") {
        mesh_instance = memnew(MeshInstance3D);
        mesh_instance->set_mesh(memnew(SphereMesh));
    } else if (node_type == "cylinder") {
        mesh_instance = memnew(MeshInstance3D);
        mesh_instance->set_mesh(memnew(CylinderMesh));
    } else if (node_type == "plane") {
        mesh_instance = memnew(MeshInstance3D);
        mesh_instance->set_mesh(memnew(PlaneMesh));
    } else {
        UtilityFunctions::print("Unknown node type: ", node_type);
        return;
    }
    
    if (mesh_instance) {
        mesh_instance->set_position(position);
        mesh_instance->set_name(node_id);
        
        // Add some random color
        Ref<StandardMaterial3D> material = memnew(StandardMaterial3D);
        material->set_albedo(Color(UtilityFunctions::randf(), UtilityFunctions::randf(), UtilityFunctions::randf(), 1.0));
        mesh_instance->set_material_override(material);
        
        add_child(mesh_instance);
        local_nodes[node_id] = mesh_instance;
        node_count++;
        
        _emit_node_spawned(node_id, position);
        UtilityFunctions::print("Created local node: ", node_id, " of type: ", node_type);
    }
}

void AetherisScene::_create_local_avatar(const String& avatar_id, const String& did, const Vector3& position) {
    CharacterBody3D* avatar = memnew(CharacterBody3D);
    avatar->set_position(position);
    
    // Create avatar mesh
    MeshInstance3D* mesh_instance = memnew(MeshInstance3D);
    mesh_instance->set_mesh(memnew(CapsuleMesh));
    
    Ref<StandardMaterial3D> material = memnew(StandardMaterial3D);
    material->set_albedo(Color(0.2, 0.8, 0.2, 1.0));
    mesh_instance->set_material_override(material);
    avatar->add_child(mesh_instance);
    
    // Add collision
    CollisionShape3D* collision = memnew(CollisionShape3D);
    collision->set_shape(memnew(CapsuleShape3D));
    avatar->add_child(collision);
    
    avatar->set_name(avatar_id);
    add_child(avatar);
    
    local_avatars[avatar_id] = avatar;
    avatar_count++;
    online_avatar_count++;
    
    _emit_avatar_spawned(avatar_id, did, position);
    UtilityFunctions::print("Created local avatar: ", avatar_id, " for DID: ", did);
}

String AetherisScene::_generate_node_id() {
    return "node_" + String::num_int64(UtilityFunctions::randi());
}

String AetherisScene::_generate_avatar_id() {
    return "avatar_" + String::num_int64(UtilityFunctions::randi());
}

String AetherisScene::_generate_snapshot_id() {
    return "snapshot_" + String::num_int64(UtilityFunctions::randi());
}

Dictionary AetherisScene::_send_scene_request(const String& request_type, const Dictionary& data) {
    // This would send the actual request to the scene service
    // For now, we'll simulate a successful response
    Dictionary response;
    response["success"] = true;
    response["request_type"] = request_type;
    response.merge(data);
    return response;
}

void AetherisScene::_handle_scene_response(const String& request_type, const Dictionary& response) {
    // This would handle responses from the scene service
    // For now, it's a placeholder
}

void AetherisScene::_emit_scene_ready() {
    emit_signal(SIGNAL_SCENE_READY);
}

void AetherisScene::_emit_scene_error(const String& error_message) {
    emit_signal(SIGNAL_SCENE_ERROR, error_message);
}

void AetherisScene::_emit_node_spawned(const String& node_id, const Vector3& position) {
    emit_signal(SIGNAL_NODE_SPAWNED, node_id, position);
}

void AetherisScene::_emit_node_updated(const String& node_id, const Vector3& position) {
    emit_signal(SIGNAL_NODE_UPDATED, node_id, position);
}

void AetherisScene::_emit_node_removed(const String& node_id) {
    emit_signal(SIGNAL_NODE_REMOVED, node_id);
}

void AetherisScene::_emit_avatar_spawned(const String& avatar_id, const String& did, const Vector3& position) {
    emit_signal(SIGNAL_AVATAR_SPAWNED, avatar_id, did, position);
}

void AetherisScene::_emit_avatar_updated(const String& avatar_id, const Vector3& position) {
    emit_signal(SIGNAL_AVATAR_UPDATED, avatar_id, position);
}

void AetherisScene::_emit_avatar_removed(const String& avatar_id) {
    emit_signal(SIGNAL_AVATAR_REMOVED, avatar_id);
}

void AetherisScene::_emit_policy_denied(const String& action, const String& reason) {
    emit_signal(SIGNAL_POLICY_DENIED, action, reason);
}

void AetherisScene::_emit_snapshot_saved(const String& snapshot_id) {
    emit_signal(SIGNAL_SNAPSHOT_SAVED, snapshot_id);
}

void AetherisScene::_emit_snapshot_loaded(const String& snapshot_id) {
    emit_signal(SIGNAL_SNAPSHOT_LOADED, snapshot_id);
}
