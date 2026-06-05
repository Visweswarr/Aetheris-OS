#ifndef AETHERIS_AVATAR_HPP
#define AETHERIS_AVATAR_HPP

#include <godot_cpp/classes/character_body3d.hpp>
#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/string.hpp>
#include <godot_cpp/variant/vector3.hpp>
#include <godot_cpp/variant/dictionary.hpp>

using namespace godot;

/// AetherisAvatar - C++ GDExtension class for avatar management
class AetherisAvatar : public CharacterBody3D {
    GDCLASS(AetherisAvatar, CharacterBody3D)

private:
    // Avatar properties
    String avatar_id;
    String did;
    String name;
    String description;
    
    // Avatar state
    bool is_online = false;
    bool is_active = false;
    Vector3 last_position;
    Vector3 last_rotation;
    
    // Avatar profile
    Dictionary profile;
    Dictionary metadata;
    
    // Movement settings
    float move_speed = 5.0f;
    float jump_velocity = 4.5f;
    float gravity = 9.8f;
    
    // Animation state
    String current_animation = "";
    float animation_speed = 1.0f;
    
    // Networking
    bool sync_enabled = true;
    double sync_timer = 0.0;
    double sync_interval = 1.0 / 30.0; // 30 Hz sync

protected:
    static void _bind_methods();

public:
    AetherisAvatar();
    ~AetherisAvatar();

    // Avatar identity
    void set_avatar_id(const String& id);
    String get_avatar_id() const;
    void set_did(const String& did);
    String get_did() const;
    void set_name(const String& name);
    String get_name() const;
    void set_description(const String& description);
    String get_description() const;

    // Avatar state
    void set_online(bool online);
    bool is_avatar_online() const;
    void set_active(bool active);
    bool is_avatar_active() const;
    void set_last_position(const Vector3& position);
    Vector3 get_last_position() const;
    void set_last_rotation(const Vector3& rotation);
    Vector3 get_last_rotation() const;

    // Avatar profile
    void set_profile(const Dictionary& profile);
    Dictionary get_profile() const;
    void set_metadata(const Dictionary& metadata);
    Dictionary get_metadata() const;

    // Movement
    void set_move_speed(float speed);
    float get_move_speed() const;
    void set_jump_velocity(float velocity);
    float get_jump_velocity() const;
    void set_gravity(float gravity);
    float get_gravity() const;
    void move_to(const Vector3& position);
    void rotate_to(const Vector3& rotation);

    // Animation
    void set_current_animation(const String& animation);
    String get_current_animation() const;
    void set_animation_speed(float speed);
    float get_animation_speed() const;
    void play_animation(const String& animation, float speed = 1.0f);
    void stop_animation();

    // Networking
    void set_sync_enabled(bool enabled);
    bool is_sync_enabled() const;
    void set_sync_interval(double interval);
    double get_sync_interval() const;
    void sync_with_service();
    void push_local_changes();

    // Event handling
    void _ready() override;
    void _process(double delta) override;
    void _physics_process(double delta) override;

    // Signals
    static constexpr const char* SIGNAL_AVATAR_READY = "avatar_ready";
    static constexpr const char* SIGNAL_AVATAR_MOVED = "avatar_moved";
    static constexpr const char* SIGNAL_AVATAR_ROTATED = "avatar_rotated";
    static constexpr const char* SIGNAL_AVATAR_ANIMATION_CHANGED = "avatar_animation_changed";
    static constexpr const char* SIGNAL_AVATAR_STATE_CHANGED = "avatar_state_changed";
    static constexpr const char* SIGNAL_AVATAR_SYNCED = "avatar_synced";

private:
    // Internal methods
    void _setup_avatar();
    void _update_movement(double delta);
    void _handle_input();
    void _apply_gravity(double delta);
    void _check_collisions();
    void _update_animation();
    void _sync_with_service();
    void _push_local_changes();
    
    // Event emission
    void _emit_avatar_ready();
    void _emit_avatar_moved(const Vector3& position);
    void _emit_avatar_rotated(const Vector3& rotation);
    void _emit_avatar_animation_changed(const String& animation);
    void _emit_avatar_state_changed();
    void _emit_avatar_synced();
};

#endif // AETHERIS_AVATAR_HPP
