#ifndef AETHERIS_BUS_HPP
#define AETHERIS_BUS_HPP

#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/core/class_db.hpp>
#include <godot_cpp/variant/string.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/variant/array.hpp>

using namespace godot;

/// AetherisBus - C++ GDExtension class for scene service communication
class AetherisBus : public Node {
    GDCLASS(AetherisBus, Node)

private:
    // Connection settings
    String scene_service_url = "localhost:50051";
    bool scene_service_connected = false;
    double connection_timeout = 5.0;
    double reconnect_interval = 1.0;
    int max_reconnect_attempts = 10;
    int reconnect_attempts = 0;
    
    // Event subscriptions
    Array subscribed_events;
    Dictionary event_handlers;
    
    // Statistics
    int total_requests = 0;
    int successful_requests = 0;
    int failed_requests = 0;
    double average_response_time = 0.0;

protected:
    static void _bind_methods();

public:
    AetherisBus();
    ~AetherisBus();

    // Connection management
    void connect_to_service();
    void disconnect_from_service();
    bool is_connected() const;
    void set_service_url(const String& url);
    String get_service_url() const;
    void set_connection_timeout(double timeout);
    double get_connection_timeout() const;
    void set_reconnect_interval(double interval);
    double get_reconnect_interval() const;
    void set_max_reconnect_attempts(int attempts);
    int get_max_reconnect_attempts() const;

    // Scene service communication
    Dictionary send_scene_request(const String& request_type, const Dictionary& data);
    void subscribe_to_events(const Array& event_types);
    void unsubscribe_from_events(const Array& event_types);
    Array get_subscribed_events() const;

    // Policy and capability checks
    Dictionary send_policy_check(const String& action, const Dictionary& context);
    Dictionary send_capability_check(const String& capability, const Dictionary& context);

    // Event handling
    void register_event_handler(const String& event_type, const Callable& handler);
    void unregister_event_handler(const String& event_type);
    void emit_event(const String& event_type, const Dictionary& data);

    // Statistics
    Dictionary get_connection_status() const;
    Dictionary get_statistics() const;
    void reset_statistics();

    // Event handling
    void _ready() override;
    void _process(double delta) override;

    // Signals
    static constexpr const char* SIGNAL_SERVICE_CONNECTED = "service_connected";
    static constexpr const char* SIGNAL_SERVICE_DISCONNECTED = "service_disconnected";
    static constexpr const char* SIGNAL_SERVICE_ERROR = "service_error";
    static constexpr const char* SIGNAL_NODE_EVENT = "node_event";
    static constexpr const char* SIGNAL_AVATAR_EVENT = "avatar_event";
    static constexpr const char* SIGNAL_POLICY_EVENT = "policy_event";
    static constexpr const char* SIGNAL_SNAPSHOT_EVENT = "snapshot_event";

private:
    // Internal methods
    void _setup_connection_monitoring();
    void _attempt_reconnect();
    void _on_connection_lost();
    void _on_service_error(const String& error_message);
    void _handle_scene_response(const String& request_type, const Dictionary& response);
    void _process_event(const String& event_type, const Dictionary& data);
    
    // Event emission
    void _emit_service_connected();
    void _emit_service_disconnected();
    void _emit_service_error(const String& error_message);
    void _emit_node_event(const String& event_type, const String& node_id, const Dictionary& data);
    void _emit_avatar_event(const String& event_type, const String& avatar_id, const Dictionary& data);
    void _emit_policy_event(const String& event_type, const String& action, const Dictionary& data);
    void _emit_snapshot_event(const String& event_type, const String& snapshot_id, const Dictionary& data);
};

#endif // AETHERIS_BUS_HPP
