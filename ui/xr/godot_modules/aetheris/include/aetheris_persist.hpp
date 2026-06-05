#ifndef AETHERIS_PERSIST_HPP
#define AETHERIS_PERSIST_HPP

#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/variant/string.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/variant/array.hpp>
#include <godot_cpp/variant/packed_byte_array.hpp>

namespace godot {

/// Aetheris Persistence Manager for XR Scene Persistence
class AetherisPersist : public RefCounted {
    GDCLASS(AetherisPersist, RefCounted);

private:
    String session_id;
    String cap_token;
    String current_room_id;

protected:
    static void _bind_methods();

public:
    AetherisPersist();
    ~AetherisPersist();

    /// Set session ID and capability token
    void set_session(const String& session_id, const String& cap_token);
    
    /// Get current session ID
    String get_session_id() const;
    
    /// Get current capability token
    String get_cap_token() const;
    
    /// Set current room ID
    void set_current_room(const String& room_id);
    
    /// Get current room ID
    String get_current_room() const;

    /// Persist room to NGFS
    Dictionary persist_room(const String& room_id, const String& reason, const String& dao_proposal_id = "");
    
    /// Load snapshot from NGFS
    Dictionary load_snapshot(const String& snapshot_id);
    
    /// Get snapshot metadata
    Dictionary get_snapshot_metadata(const String& snapshot_id);
    
    /// List snapshots for a room
    Array list_snapshots(const String& room_id);
    
    /// Verify snapshot determinism
    bool verify_determinism(const String& snapshot_id);
    
    /// Get persistence statistics
    Dictionary get_stats() const;
};

/// Aetheris Recording Manager for XR Session Recording
class AetherisRecord : public RefCounted {
    GDCLASS(AetherisRecord, RefCounted);

private:
    String session_id;
    String cap_token;
    String current_recording_id;
    bool is_recording;
    Dictionary recording_stats;

protected:
    static void _bind_methods();

public:
    AetherisRecord();
    ~AetherisRecord();

    /// Set session ID and capability token
    void set_session(const String& session_id, const String& cap_token);
    
    /// Get current session ID
    String get_session_id() const;
    
    /// Get current capability token
    String get_cap_token() const;
    
    /// Check if currently recording
    bool is_currently_recording() const;
    
    /// Get current recording ID
    String get_current_recording_id() const;

    /// Start recording a room
    Dictionary start_recording(const String& room_id, int64_t deterministic_seed = 0, float physics_tick_rate = 60.0);
    
    /// Stop current recording
    Dictionary stop_recording();
    
    /// Get recording summary
    Dictionary get_recording_summary(const String& recording_id);
    
    /// List recordings for a room
    Array list_recordings(const String& room_id);
    
    /// Export recording to file
    bool export_recording(const String& recording_id, const String& file_path, const String& format = "cbor");
    
    /// Get recording statistics
    Dictionary get_stats() const;
};

/// Aetheris Replay Manager for XR Session Replay
class AetherisReplay : public RefCounted {
    GDCLASS(AetherisReplay, RefCounted);

public:
    enum ReplayMode {
        REPLAY_HEADLESS = 0,
        REPLAY_VISUALIZATION = 1,
        REPLAY_DEBUG = 2
    };

private:
    String session_id;
    String cap_token;
    String current_replay_id;
    bool is_replaying;
    Dictionary replay_stats;

protected:
    static void _bind_methods();

public:
    AetherisReplay();
    ~AetherisReplay();

    /// Set session ID and capability token
    void set_session(const String& session_id, const String& cap_token);
    
    /// Get current session ID
    String get_session_id() const;
    
    /// Get current capability token
    String get_cap_token() const;
    
    /// Check if currently replaying
    bool is_currently_replaying() const;
    
    /// Get current replay ID
    String get_current_replay_id() const;

    /// Replay a recording
    Dictionary replay_recording(const String& recording_id, ReplayMode mode = REPLAY_HEADLESS, 
                               int64_t start_event = 0, int64_t end_event = 0, bool verify_determinism = true);
    
    /// Stop current replay
    bool stop_replay();
    
    /// Get replay result
    Dictionary get_replay_result(const String& replay_id);
    
    /// Verify replay determinism
    bool verify_determinism(const String& replay_id);
    
    /// Export replay snapshot
    Dictionary export_replay_snapshot(const String& replay_id);
    
    /// Get replay statistics
    Dictionary get_stats() const;
};

/// Aetheris Bridge Manager for Cross-Metaverse Interoperability
class AetherisBridge : public RefCounted {
    GDCLASS(AetherisBridge, RefCounted);

public:
    enum ExportFormat {
        EXPORT_GLTF = 0,
        EXPORT_CAR = 1,
        EXPORT_JSON = 2,
        EXPORT_ALL = 3
    };

    enum ImportPolicyMode {
        IMPORT_STRICT = 0,
        IMPORT_PERMISSIVE = 1,
        IMPORT_SANDBOX = 2
    };

private:
    String session_id;
    String cap_token;
    Dictionary bridge_stats;

protected:
    static void _bind_methods();

public:
    AetherisBridge();
    ~AetherisBridge();

    /// Set session ID and capability token
    void set_session(const String& session_id, const String& cap_token);
    
    /// Get current session ID
    String get_session_id() const;
    
    /// Get current capability token
    String get_cap_token() const;

    /// Export scene to various formats
    Dictionary export_scene(const String& room_id, const Array& formats, bool include_avatars = true, 
                           bool include_physics = true, int compression_level = 6);
    
    /// Import scene from bundle
    Dictionary import_scene(const PackedByteArray& bundle_data, ImportPolicyMode policy_mode = IMPORT_SANDBOX,
                           const String& target_room_id = "", bool merge_mode = false);
    
    /// Publish scene to public registry
    Dictionary publish_scene(const String& room_id, bool anchor = false, const String& dao_proposal_id = "",
                            bool public_access = true, const String& license = "MIT");
    
    /// Get export bundle
    Dictionary get_export_bundle(const String& bundle_id);
    
    /// List exports for a room
    Array list_exports(const String& room_id);
    
    /// Verify export integrity
    bool verify_export_integrity(const String& bundle_id);
    
    /// Get bridge statistics
    Dictionary get_stats() const;
};

} // namespace godot

VARIANT_ENUM_CAST(AetherisReplay::ReplayMode);
VARIANT_ENUM_CAST(AetherisBridge::ExportFormat);
VARIANT_ENUM_CAST(AetherisBridge::ImportPolicyMode);

#endif // AETHERIS_PERSIST_HPP
