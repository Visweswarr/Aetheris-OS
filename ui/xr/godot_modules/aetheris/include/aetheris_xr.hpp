/**
 * Aetheris XR Module for Godot
 * 
 * This module provides XR device management, multi-user sessions, and physics
 * interaction capabilities for the Aetheris XR surface in Godot.
 */

#ifndef AETHERIS_XR_HPP
#define AETHERIS_XR_HPP

#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/variant/string.hpp>
#include <godot_cpp/variant/array.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/variant/vector3.hpp>
#include <godot_cpp/variant/transform3d.hpp>
#include <godot_cpp/classes/signal.hpp>

using namespace godot;

/**
 * XR Device Manager
 * 
 * Manages XR device connections, input handling, and device capabilities.
 */
class AetherisXR : public RefCounted {
    GDCLASS(AetherisXR, RefCounted);

private:
    String device_handle;
    String device_profile;
    String session_id;
    bool is_device_active;
    Array device_capabilities;
    Dictionary input_handlers;

protected:
    static void _bind_methods();

public:
    AetherisXR();
    ~AetherisXR();

    // Device Management
    bool start_device(const String& device_profile, const String& xr_session_id);
    void stop_device();
    bool is_device_connected() const;
    String get_device_handle() const;
    String get_device_profile() const;
    Array get_device_capabilities() const;

    // Input Handling
    void send_button_input(const String& button_id, bool pressed, uint64_t timestamp);
    void send_trigger_input(const String& trigger_id, float value, uint64_t timestamp);
    void send_joystick_input(const String& joystick_id, float x, float y, uint64_t timestamp);
    void send_hand_tracking_data(const String& hand_id, const Array& joints, uint64_t timestamp);
    void send_eye_tracking_data(const Vector3& gaze_origin, const Vector3& gaze_direction, uint64_t timestamp);
    void send_voice_command(const String& command, float confidence, uint64_t timestamp);

    // Session Management
    void set_session_id(const String& session_id);
    String get_session_id() const;

    // Signals
    Signal device_started;
    Signal device_stopped;
    Signal input_received;
    Signal error_occurred;
};

/**
 * Multi-User Session Manager
 * 
 * Manages multi-user XR sessions, room joining/leaving, and scene synchronization.
 */
class AetherisMultiUser : public RefCounted {
    GDCLASS(AetherisMultiUser, RefCounted);

private:
    String participant_id;
    String room_id;
    String session_id;
    bool is_in_room;
    Dictionary room_state;
    Array participants;
    uint64_t last_sync_version;

protected:
    static void _bind_methods();

public:
    AetherisMultiUser();
    ~AetherisMultiUser();

    // Room Management
    bool join_room(const String& room_id, const String& display_name, const Dictionary& avatar_config = Dictionary());
    void leave_room();
    bool is_in_room() const;
    String get_participant_id() const;
    String get_room_id() const;
    Array get_participants() const;
    Dictionary get_room_state() const;

    // Scene Synchronization
    Dictionary sync_scene(uint64_t last_version = 0);
    uint64_t get_last_sync_version() const;
    void set_last_sync_version(uint64_t version);

    // Session Management
    void set_session_id(const String& session_id);
    String get_session_id() const;

    // Signals
    Signal room_joined;
    Signal room_left;
    Signal participant_joined;
    Signal participant_left;
    Signal scene_synchronized;
    Signal error_occurred;
};

/**
 * Physics Interaction Manager
 * 
 * Manages physics interactions, object manipulation, and collision detection.
 */
class AetherisPhysics : public RefCounted {
    GDCLASS(AetherisPhysics, RefCounted);

private:
    String session_id;
    Dictionary grabbed_objects;
    Array physics_bodies;
    Dictionary physics_constraints;
    bool is_physics_enabled;

protected:
    static void _bind_methods();

public:
    AetherisPhysics();
    ~AetherisPhysics();

    // Object Manipulation
    bool grab_object(const String& object_id, const Vector3& grab_point);
    bool move_object(const String& object_id, const Transform3D& target_transform);
    bool release_object(const String& object_id);
    bool is_object_grabbed(const String& object_id) const;
    Array get_grabbed_objects() const;

    // Physics Operations
    bool apply_force(const String& object_id, const Vector3& force, const Vector3& point);
    bool apply_impulse(const String& object_id, const Vector3& impulse, const Vector3& point);
    bool set_velocity(const String& object_id, const Vector3& velocity);
    bool teleport_object(const String& object_id, const Transform3D& transform);

    // Raycasting
    Dictionary raycast(const Vector3& origin, const Vector3& direction, float max_distance = 100.0f);

    // Physics State
    Array get_physics_bodies() const;
    Dictionary get_physics_constraints() const;
    bool is_physics_enabled() const;
    void set_physics_enabled(bool enabled);

    // Session Management
    void set_session_id(const String& session_id);
    String get_session_id() const;

    // Signals
    Signal object_grabbed;
    Signal object_moved;
    Signal object_released;
    Signal physics_interaction;
    Signal raycast_hit;
    Signal error_occurred;
};

/**
 * XR Scene Manager
 * 
 * Main manager that coordinates XR devices, multi-user sessions, and physics.
 */
class AetherisXRScene : public Node {
    GDCLASS(AetherisXRScene, Node);

private:
    Ref<AetherisXR> xr_device;
    Ref<AetherisMultiUser> multi_user;
    Ref<AetherisPhysics> physics;
    String session_id;
    bool is_initialized;
    Dictionary scene_state;
    Array scene_nodes;

protected:
    static void _bind_methods();
    void _ready() override;
    void _process(double delta) override;

public:
    AetherisXRScene();
    ~AetherisXRScene();

    // Initialization
    bool initialize(const String& session_id);
    void shutdown();
    bool is_initialized() const;

    // XR Device Management
    Ref<AetherisXR> get_xr_device() const;
    bool start_xr_device(const String& device_profile);
    void stop_xr_device();

    // Multi-User Management
    Ref<AetherisMultiUser> get_multi_user() const;
    bool join_room(const String& room_id, const String& display_name, const Dictionary& avatar_config = Dictionary());
    void leave_room();

    // Physics Management
    Ref<AetherisPhysics> get_physics() const;
    void enable_physics(bool enabled);

    // Scene Management
    Dictionary get_scene_state() const;
    Array get_scene_nodes() const;
    void update_scene_node(const String& node_id, const Dictionary& node_data);

    // Session Management
    void set_session_id(const String& session_id);
    String get_session_id() const;

    // Signals
    Signal xr_initialized;
    Signal xr_shutdown;
    Signal room_joined;
    Signal room_left;
    Signal physics_enabled;
    Signal physics_disabled;
    Signal scene_updated;
    Signal error_occurred;
};

#endif // AETHERIS_XR_HPP
