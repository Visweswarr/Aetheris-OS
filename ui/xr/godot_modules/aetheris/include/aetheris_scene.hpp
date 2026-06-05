#ifndef AETHERIS_SCENE_HPP
#define AETHERIS_SCENE_HPP

#include <godot_cpp/classes/node3d.hpp>
#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/string.hpp>
#include <godot_cpp/variant/vector3.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/variant/array.hpp>

using namespace godot;

/// AetherisScene - C++ GDExtension class for scene graph integration
class AetherisScene : public Node3D {
    GDCLASS(AetherisScene, Node3D)

private:
    // Scene service connection
    bool scene_service_connected = false;
    String scene_service_url = "localhost:50051";
    
    // Local scene state
    Dictionary local_nodes;
    Dictionary local_avatars;
    
    // Sync settings
    double sync_timer = 0.0;
    double sync_interval = 1.0 / 60.0; // 60 Hz sync
    
    // Scene statistics
    int node_count = 0;
    int avatar_count = 0;
    int online_avatar_count = 0;

protected:
    static void _bind_methods();

public:
    AetherisScene();
    ~AetherisScene();

    // Scene service connection
    void connect_to_service();
    void disconnect_from_service();
    bool is_service_connected() const;
    void set_service_url(const String& url);
    String get_service_url() const;

    // Node operations
    String spawn_node(const String& node_type, const Vector3& position, const String& parent_id = "");
    void update_node(const String& node_id, const Vector3& position, const Vector3& rotation = Vector3(), const Vector3& scale = Vector3(1, 1, 1));
    void remove_node(const String& node_id);
    Array get_nodes() const;
    int get_node_count() const;

    // Avatar operations
    String spawn_avatar(const String& did, const Vector3& position);
    void update_avatar(const String& avatar_id, const Vector3& position, const Vector3& rotation = Vector3());
    void remove_avatar(const String& avatar_id);
    Array get_avatars() const;
    int get_avatar_count() const;
    int get_online_avatar_count() const;

    // Snapshot operations
    String save_snapshot(const String& label);
    void load_snapshot(const String& snapshot_id);
    Array get_snapshots() const;

    // Scene management
    void clear_scene();
    void reset_scene();
    Dictionary get_scene_state() const;

    // Sync operations
    void sync_from_service();
    void push_local_changes();
    void set_sync_interval(double interval);
    double get_sync_interval() const;

    // Event handling
    void _ready() override;
    void _process(double delta) override;

    // Signals
    static constexpr const char* SIGNAL_SCENE_READY = "scene_ready";
    static constexpr const char* SIGNAL_SCENE_ERROR = "scene_error";
    static constexpr const char* SIGNAL_NODE_SPAWNED = "node_spawned";
    static constexpr const char* SIGNAL_NODE_UPDATED = "node_updated";
    static constexpr const char* SIGNAL_NODE_REMOVED = "node_removed";
    static constexpr const char* SIGNAL_AVATAR_SPAWNED = "avatar_spawned";
    static constexpr const char* SIGNAL_AVATAR_UPDATED = "avatar_updated";
    static constexpr const char* SIGNAL_AVATAR_REMOVED = "avatar_removed";
    static constexpr const char* SIGNAL_POLICY_DENIED = "policy_denied";
    static constexpr const char* SIGNAL_SNAPSHOT_SAVED = "snapshot_saved";
    static constexpr const char* SIGNAL_SNAPSHOT_LOADED = "snapshot_loaded";

private:
    // Internal methods
    void _connect_to_scene_service();
    void _sync_with_service();
    void _create_local_node(const String& node_id, const String& node_type, const Vector3& position, const String& parent_id);
    void _create_local_avatar(const String& avatar_id, const String& did, const Vector3& position);
    String _generate_node_id();
    String _generate_avatar_id();
    String _generate_snapshot_id();
    
    // Scene service communication
    Dictionary _send_scene_request(const String& request_type, const Dictionary& data);
    void _handle_scene_response(const String& request_type, const Dictionary& response);
    
    // Event emission
    void _emit_scene_ready();
    void _emit_scene_error(const String& error_message);
    void _emit_node_spawned(const String& node_id, const Vector3& position);
    void _emit_node_updated(const String& node_id, const Vector3& position);
    void _emit_node_removed(const String& node_id);
    void _emit_avatar_spawned(const String& avatar_id, const String& did, const Vector3& position);
    void _emit_avatar_updated(const String& avatar_id, const Vector3& position);
    void _emit_avatar_removed(const String& avatar_id);
    void _emit_policy_denied(const String& action, const String& reason);
    void _emit_snapshot_saved(const String& snapshot_id);
    void _emit_snapshot_loaded(const String& snapshot_id);
};

#endif // AETHERIS_SCENE_HPP
