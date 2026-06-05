#include "aetheris_avatar.hpp"
#include <godot_cpp/classes/mesh_instance3d.hpp>
#include <godot_cpp/classes/capsule_mesh.hpp>
#include <godot_cpp/classes/standard_material3d.hpp>
#include <godot_cpp/classes/collision_shape3d.hpp>
#include <godot_cpp/classes/capsule_shape3d.hpp>
#include <godot_cpp/classes/input.hpp>
#include <godot_cpp/classes/utility_functions.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

void AetherisAvatar::_bind_methods() {
    // Avatar identity
    ClassDB::bind_method(D_METHOD("set_avatar_id", "id"), &AetherisAvatar::set_avatar_id);
    ClassDB::bind_method(D_METHOD("get_avatar_id"), &AetherisAvatar::get_avatar_id);
    ClassDB::bind_method(D_METHOD("set_did", "did"), &AetherisAvatar::set_did);
    ClassDB::bind_method(D_METHOD("get_did"), &AetherisAvatar::get_did);
    ClassDB::bind_method(D_METHOD("set_name", "name"), &AetherisAvatar::set_name);
    ClassDB::bind_method(D_METHOD("get_name"), &AetherisAvatar::get_name);
    ClassDB::bind_method(D_METHOD("set_description", "description"), &AetherisAvatar::set_description);
    ClassDB::bind_method(D_METHOD("get_description"), &AetherisAvatar::get_description);

    // Avatar state
    ClassDB::bind_method(D_METHOD("set_online", "online"), &AetherisAvatar::set_online);
    ClassDB::bind_method(D_METHOD("is_avatar_online"), &AetherisAvatar::is_avatar_online);
    ClassDB::bind_method(D_METHOD("set_active", "active"), &AetherisAvatar::set_active);
    ClassDB::bind_method(D_METHOD("is_avatar_active"), &AetherisAvatar::is_avatar_active);
    ClassDB::bind_method(D_METHOD("set_last_position", "position"), &AetherisAvatar::set_last_position);
    ClassDB::bind_method(D_METHOD("get_last_position"), &AetherisAvatar::get_last_position);
    ClassDB::bind_method(D_METHOD("set_last_rotation", "rotation"), &AetherisAvatar::set_last_rotation);
    ClassDB::bind_method(D_METHOD("get_last_rotation"), &AetherisAvatar::get_last_rotation);

    // Avatar profile
    ClassDB::bind_method(D_METHOD("set_profile", "profile"), &AetherisAvatar::set_profile);
    ClassDB::bind_method(D_METHOD("get_profile"), &AetherisAvatar::get_profile);
    ClassDB::bind_method(D_METHOD("set_metadata", "metadata"), &AetherisAvatar::set_metadata);
    ClassDB::bind_method(D_METHOD("get_metadata"), &AetherisAvatar::get_metadata);

    // Movement
    ClassDB::bind_method(D_METHOD("set_move_speed", "speed"), &AetherisAvatar::set_move_speed);
    ClassDB::bind_method(D_METHOD("get_move_speed"), &AetherisAvatar::get_move_speed);
    ClassDB::bind_method(D_METHOD("set_jump_velocity", "velocity"), &AetherisAvatar::set_jump_velocity);
    ClassDB::bind_method(D_METHOD("get_jump_velocity"), &AetherisAvatar::get_jump_velocity);
    ClassDB::bind_method(D_METHOD("set_gravity", "gravity"), &AetherisAvatar::set_gravity);
    ClassDB::bind_method(D_METHOD("get_gravity"), &AetherisAvatar::get_gravity);
    ClassDB::bind_method(D_METHOD("move_to", "position"), &AetherisAvatar::move_to);
    ClassDB::bind_method(D_METHOD("rotate_to", "rotation"), &AetherisAvatar::rotate_to);

    // Animation
    ClassDB::bind_method(D_METHOD("set_current_animation", "animation"), &AetherisAvatar::set_current_animation);
    ClassDB::bind_method(D_METHOD("get_current_animation"), &AetherisAvatar::get_current_animation);
    ClassDB::bind_method(D_METHOD("set_animation_speed", "speed"), &AetherisAvatar::set_animation_speed);
    ClassDB::bind_method(D_METHOD("get_animation_speed"), &AetherisAvatar::get_animation_speed);
    ClassDB::bind_method(D_METHOD("play_animation", "animation", "speed"), &AetherisAvatar::play_animation, DEFVAL(1.0f));
    ClassDB::bind_method(D_METHOD("stop_animation"), &AetherisAvatar::stop_animation);

    // Networking
    ClassDB::bind_method(D_METHOD("set_sync_enabled", "enabled"), &AetherisAvatar::set_sync_enabled);
    ClassDB::bind_method(D_METHOD("is_sync_enabled"), &AetherisAvatar::is_sync_enabled);
    ClassDB::bind_method(D_METHOD("set_sync_interval", "interval"), &AetherisAvatar::set_sync_interval);
    ClassDB::bind_method(D_METHOD("get_sync_interval"), &AetherisAvatar::get_sync_interval);
    ClassDB::bind_method(D_METHOD("sync_with_service"), &AetherisAvatar::sync_with_service);
    ClassDB::bind_method(D_METHOD("push_local_changes"), &AetherisAvatar::push_local_changes);

    // Properties
    ADD_PROPERTY(PropertyInfo(Variant::STRING, "avatar_id"), "set_avatar_id", "get_avatar_id");
    ADD_PROPERTY(PropertyInfo(Variant::STRING, "did"), "set_did", "get_did");
    ADD_PROPERTY(PropertyInfo(Variant::STRING, "name"), "set_name", "get_name");
    ADD_PROPERTY(PropertyInfo(Variant::STRING, "description"), "set_description", "get_description");
    ADD_PROPERTY(PropertyInfo(Variant::BOOL, "is_online"), "set_online", "is_avatar_online");
    ADD_PROPERTY(PropertyInfo(Variant::BOOL, "is_active"), "set_active", "is_avatar_active");
    ADD_PROPERTY(PropertyInfo(Variant::VECTOR3, "last_position"), "set_last_position", "get_last_position");
    ADD_PROPERTY(PropertyInfo(Variant::VECTOR3, "last_rotation"), "set_last_rotation", "get_last_rotation");
    ADD_PROPERTY(PropertyInfo(Variant::DICTIONARY, "profile"), "set_profile", "get_profile");
    ADD_PROPERTY(PropertyInfo(Variant::DICTIONARY, "metadata"), "set_metadata", "get_metadata");
    ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "move_speed"), "set_move_speed", "get_move_speed");
    ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "jump_velocity"), "set_jump_velocity", "get_jump_velocity");
    ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "gravity"), "set_gravity", "get_gravity");
    ADD_PROPERTY(PropertyInfo(Variant::STRING, "current_animation"), "set_current_animation", "get_current_animation");
    ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "animation_speed"), "set_animation_speed", "get_animation_speed");
    ADD_PROPERTY(PropertyInfo(Variant::BOOL, "sync_enabled"), "set_sync_enabled", "is_sync_enabled");
    ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "sync_interval"), "set_sync_interval", "get_sync_interval");

    // Signals
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_READY));
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_MOVED, PropertyInfo(Variant::VECTOR3, "position")));
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_ROTATED, PropertyInfo(Variant::VECTOR3, "rotation")));
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_ANIMATION_CHANGED, PropertyInfo(Variant::STRING, "animation")));
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_STATE_CHANGED));
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_SYNCED));
}

AetherisAvatar::AetherisAvatar() {
    UtilityFunctions::print("AetherisAvatar created");
}

AetherisAvatar::~AetherisAvatar() {
    UtilityFunctions::print("AetherisAvatar destroyed");
}

void AetherisAvatar::_ready() {
    _setup_avatar();
    _emit_avatar_ready();
}

void AetherisAvatar::_process(double delta) {
    if (sync_enabled) {
        sync_timer += delta;
        if (sync_timer >= sync_interval) {
            sync_timer = 0.0;
            _sync_with_service();
        }
    }
    
    _update_animation();
}

void AetherisAvatar::_physics_process(double delta) {
    _apply_gravity(delta);
    _update_movement(delta);
    _check_collisions();
}

void AetherisAvatar::set_avatar_id(const String& id) {
    avatar_id = id;
}

String AetherisAvatar::get_avatar_id() const {
    return avatar_id;
}

void AetherisAvatar::set_did(const String& did) {
    this->did = did;
}

String AetherisAvatar::get_did() const {
    return did;
}

void AetherisAvatar::set_name(const String& name) {
    this->name = name;
}

String AetherisAvatar::get_name() const {
    return name;
}

void AetherisAvatar::set_description(const String& description) {
    this->description = description;
}

String AetherisAvatar::get_description() const {
    return description;
}

void AetherisAvatar::set_online(bool online) {
    if (is_online != online) {
        is_online = online;
        _emit_avatar_state_changed();
    }
}

bool AetherisAvatar::is_avatar_online() const {
    return is_online;
}

void AetherisAvatar::set_active(bool active) {
    if (is_active != active) {
        is_active = active;
        _emit_avatar_state_changed();
    }
}

bool AetherisAvatar::is_avatar_active() const {
    return is_active;
}

void AetherisAvatar::set_last_position(const Vector3& position) {
    last_position = position;
}

Vector3 AetherisAvatar::get_last_position() const {
    return last_position;
}

void AetherisAvatar::set_last_rotation(const Vector3& rotation) {
    last_rotation = rotation;
}

Vector3 AetherisAvatar::get_last_rotation() const {
    return last_rotation;
}

void AetherisAvatar::set_profile(const Dictionary& profile) {
    this->profile = profile;
}

Dictionary AetherisAvatar::get_profile() const {
    return profile;
}

void AetherisAvatar::set_metadata(const Dictionary& metadata) {
    this->metadata = metadata;
}

Dictionary AetherisAvatar::get_metadata() const {
    return metadata;
}

void AetherisAvatar::set_move_speed(float speed) {
    move_speed = speed;
}

float AetherisAvatar::get_move_speed() const {
    return move_speed;
}

void AetherisAvatar::set_jump_velocity(float velocity) {
    jump_velocity = velocity;
}

float AetherisAvatar::get_jump_velocity() const {
    return jump_velocity;
}

void AetherisAvatar::set_gravity(float gravity) {
    this->gravity = gravity;
}

float AetherisAvatar::get_gravity() const {
    return gravity;
}

void AetherisAvatar::move_to(const Vector3& position) {
    set_position(position);
    last_position = position;
    _emit_avatar_moved(position);
}

void AetherisAvatar::rotate_to(const Vector3& rotation) {
    set_rotation(rotation);
    last_rotation = rotation;
    _emit_avatar_rotated(rotation);
}

void AetherisAvatar::set_current_animation(const String& animation) {
    if (current_animation != animation) {
        current_animation = animation;
        _emit_avatar_animation_changed(animation);
    }
}

String AetherisAvatar::get_current_animation() const {
    return current_animation;
}

void AetherisAvatar::set_animation_speed(float speed) {
    animation_speed = speed;
}

float AetherisAvatar::get_animation_speed() const {
    return animation_speed;
}

void AetherisAvatar::play_animation(const String& animation, float speed) {
    set_current_animation(animation);
    set_animation_speed(speed);
}

void AetherisAvatar::stop_animation() {
    set_current_animation("");
}

void AetherisAvatar::set_sync_enabled(bool enabled) {
    sync_enabled = enabled;
}

bool AetherisAvatar::is_sync_enabled() const {
    return sync_enabled;
}

void AetherisAvatar::set_sync_interval(double interval) {
    sync_interval = interval;
}

double AetherisAvatar::get_sync_interval() const {
    return sync_interval;
}

void AetherisAvatar::sync_with_service() {
    _sync_with_service();
}

void AetherisAvatar::push_local_changes() {
    _push_local_changes();
}

void AetherisAvatar::_setup_avatar() {
    // Create avatar mesh
    MeshInstance3D* mesh_instance = memnew(MeshInstance3D);
    mesh_instance->set_mesh(memnew(CapsuleMesh));
    
    Ref<StandardMaterial3D> material = memnew(StandardMaterial3D);
    material->set_albedo(Color(0.2, 0.8, 0.2, 1.0));
    mesh_instance->set_material_override(material);
    add_child(mesh_instance);
    
    // Add collision
    CollisionShape3D* collision = memnew(CollisionShape3D);
    collision->set_shape(memnew(CapsuleShape3D));
    add_child(collision);
    
    // Set initial state
    is_online = true;
    is_active = true;
    last_position = get_position();
    last_rotation = get_rotation();
    
    UtilityFunctions::print("Avatar setup complete for ID: ", avatar_id);
}

void AetherisAvatar::_update_movement(double delta) {
    if (!is_active) {
        return;
    }
    
    Vector3 velocity = get_velocity();
    
    // Handle input
    _handle_input();
    
    // Apply movement
    if (velocity.length() > 0) {
        set_velocity(velocity);
        move_and_slide();
        
        // Update last position if moved
        Vector3 current_position = get_position();
        if (current_position.distance_to(last_position) > 0.01f) {
            last_position = current_position;
            _emit_avatar_moved(current_position);
        }
    }
}

void AetherisAvatar::_handle_input() {
    if (!is_active) {
        return;
    }
    
    Vector3 velocity = get_velocity();
    
    // Handle movement input
    if (Input::get_singleton()->is_action_pressed("ui_right")) {
        velocity.x = move_speed;
    } else if (Input::get_singleton()->is_action_pressed("ui_left")) {
        velocity.x = -move_speed;
    } else {
        velocity.x = 0;
    }
    
    if (Input::get_singleton()->is_action_pressed("ui_down")) {
        velocity.z = move_speed;
    } else if (Input::get_singleton()->is_action_pressed("ui_up")) {
        velocity.z = -move_speed;
    } else {
        velocity.z = 0;
    }
    
    // Handle jump
    if (Input::get_singleton()->is_action_just_pressed("ui_accept") && is_on_floor()) {
        velocity.y = jump_velocity;
    }
    
    set_velocity(velocity);
}

void AetherisAvatar::_apply_gravity(double delta) {
    if (!is_on_floor()) {
        Vector3 velocity = get_velocity();
        velocity.y -= gravity * delta;
        set_velocity(velocity);
    }
}

void AetherisAvatar::_check_collisions() {
    // Handle collisions
    if (get_slide_collision_count() > 0) {
        // Process collisions
        for (int i = 0; i < get_slide_collision_count(); i++) {
            KinematicCollision3D* collision = get_slide_collision(i);
            if (collision) {
                // Handle collision
                UtilityFunctions::print("Avatar collision detected");
            }
        }
    }
}

void AetherisAvatar::_update_animation() {
    if (current_animation.is_empty()) {
        return;
    }
    
    // Update animation based on current state
    Vector3 velocity = get_velocity();
    if (velocity.length() > 0.1f) {
        if (current_animation != "walk") {
            play_animation("walk", 1.0f);
        }
    } else {
        if (current_animation != "idle") {
            play_animation("idle", 1.0f);
        }
    }
}

void AetherisAvatar::_sync_with_service() {
    // This would sync with the scene service
    // For now, it's a placeholder
    _emit_avatar_synced();
}

void AetherisAvatar::_push_local_changes() {
    // This would push local changes to the scene service
    // For now, it's a placeholder
}

void AetherisAvatar::_emit_avatar_ready() {
    emit_signal(SIGNAL_AVATAR_READY);
}

void AetherisAvatar::_emit_avatar_moved(const Vector3& position) {
    emit_signal(SIGNAL_AVATAR_MOVED, position);
}

void AetherisAvatar::_emit_avatar_rotated(const Vector3& rotation) {
    emit_signal(SIGNAL_AVATAR_ROTATED, rotation);
}

void AetherisAvatar::_emit_avatar_animation_changed(const String& animation) {
    emit_signal(SIGNAL_AVATAR_ANIMATION_CHANGED, animation);
}

void AetherisAvatar::_emit_avatar_state_changed() {
    emit_signal(SIGNAL_AVATAR_STATE_CHANGED);
}

void AetherisAvatar::_emit_avatar_synced() {
    emit_signal(SIGNAL_AVATAR_SYNCED);
}
