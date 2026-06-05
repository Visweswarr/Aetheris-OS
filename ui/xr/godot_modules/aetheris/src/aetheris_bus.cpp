#include "aetheris_bus.hpp"
#include <godot_cpp/classes/utility_functions.hpp>
#include <godot_cpp/variant/utility_functions.hpp>

void AetherisBus::_bind_methods() {
    // Connection management
    ClassDB::bind_method(D_METHOD("connect_to_service"), &AetherisBus::connect_to_service);
    ClassDB::bind_method(D_METHOD("disconnect_from_service"), &AetherisBus::disconnect_from_service);
    ClassDB::bind_method(D_METHOD("is_connected"), &AetherisBus::is_connected);
    ClassDB::bind_method(D_METHOD("set_service_url", "url"), &AetherisBus::set_service_url);
    ClassDB::bind_method(D_METHOD("get_service_url"), &AetherisBus::get_service_url);
    ClassDB::bind_method(D_METHOD("set_connection_timeout", "timeout"), &AetherisBus::set_connection_timeout);
    ClassDB::bind_method(D_METHOD("get_connection_timeout"), &AetherisBus::get_connection_timeout);
    ClassDB::bind_method(D_METHOD("set_reconnect_interval", "interval"), &AetherisBus::set_reconnect_interval);
    ClassDB::bind_method(D_METHOD("get_reconnect_interval"), &AetherisBus::get_reconnect_interval);
    ClassDB::bind_method(D_METHOD("set_max_reconnect_attempts", "attempts"), &AetherisBus::set_max_reconnect_attempts);
    ClassDB::bind_method(D_METHOD("get_max_reconnect_attempts"), &AetherisBus::get_max_reconnect_attempts);

    // Scene service communication
    ClassDB::bind_method(D_METHOD("send_scene_request", "request_type", "data"), &AetherisBus::send_scene_request);
    ClassDB::bind_method(D_METHOD("subscribe_to_events", "event_types"), &AetherisBus::subscribe_to_events);
    ClassDB::bind_method(D_METHOD("unsubscribe_from_events", "event_types"), &AetherisBus::unsubscribe_from_events);
    ClassDB::bind_method(D_METHOD("get_subscribed_events"), &AetherisBus::get_subscribed_events);

    // Policy and capability checks
    ClassDB::bind_method(D_METHOD("send_policy_check", "action", "context"), &AetherisBus::send_policy_check);
    ClassDB::bind_method(D_METHOD("send_capability_check", "capability", "context"), &AetherisBus::send_capability_check);

    // Event handling
    ClassDB::bind_method(D_METHOD("register_event_handler", "event_type", "handler"), &AetherisBus::register_event_handler);
    ClassDB::bind_method(D_METHOD("unregister_event_handler", "event_type"), &AetherisBus::unregister_event_handler);
    ClassDB::bind_method(D_METHOD("emit_event", "event_type", "data"), &AetherisBus::emit_event);

    // Statistics
    ClassDB::bind_method(D_METHOD("get_connection_status"), &AetherisBus::get_connection_status);
    ClassDB::bind_method(D_METHOD("get_statistics"), &AetherisBus::get_statistics);
    ClassDB::bind_method(D_METHOD("reset_statistics"), &AetherisBus::reset_statistics);

    // Properties
    ADD_PROPERTY(PropertyInfo(Variant::STRING, "service_url"), "set_service_url", "get_service_url");
    ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "connection_timeout"), "set_connection_timeout", "get_connection_timeout");
    ADD_PROPERTY(PropertyInfo(Variant::FLOAT, "reconnect_interval"), "set_reconnect_interval", "get_reconnect_interval");
    ADD_PROPERTY(PropertyInfo(Variant::INT, "max_reconnect_attempts"), "set_max_reconnect_attempts", "get_max_reconnect_attempts");

    // Signals
    ADD_SIGNAL(MethodInfo(SIGNAL_SERVICE_CONNECTED));
    ADD_SIGNAL(MethodInfo(SIGNAL_SERVICE_DISCONNECTED));
    ADD_SIGNAL(MethodInfo(SIGNAL_SERVICE_ERROR, PropertyInfo(Variant::STRING, "error_message")));
    ADD_SIGNAL(MethodInfo(SIGNAL_NODE_EVENT, PropertyInfo(Variant::STRING, "event_type"), PropertyInfo(Variant::STRING, "node_id"), PropertyInfo(Variant::DICTIONARY, "data")));
    ADD_SIGNAL(MethodInfo(SIGNAL_AVATAR_EVENT, PropertyInfo(Variant::STRING, "event_type"), PropertyInfo(Variant::STRING, "avatar_id"), PropertyInfo(Variant::DICTIONARY, "data")));
    ADD_SIGNAL(MethodInfo(SIGNAL_POLICY_EVENT, PropertyInfo(Variant::STRING, "event_type"), PropertyInfo(Variant::STRING, "action"), PropertyInfo(Variant::DICTIONARY, "data")));
    ADD_SIGNAL(MethodInfo(SIGNAL_SNAPSHOT_EVENT, PropertyInfo(Variant::STRING, "event_type"), PropertyInfo(Variant::STRING, "snapshot_id"), PropertyInfo(Variant::DICTIONARY, "data")));
}

AetherisBus::AetherisBus() {
    UtilityFunctions::print("AetherisBus created");
}

AetherisBus::~AetherisBus() {
    UtilityFunctions::print("AetherisBus destroyed");
}

void AetherisBus::_ready() {
    UtilityFunctions::print("AetherisBus ready");
    _setup_connection_monitoring();
}

void AetherisBus::_process(double delta) {
    // Process any pending operations
    // For now, it's a placeholder
}

void AetherisBus::connect_to_service() {
    UtilityFunctions::print("Connecting to Aetheris Scene Service at: ", scene_service_url);
    
    // This would establish the actual connection
    // For now, we'll simulate success
    scene_service_connected = true;
    reconnect_attempts = 0;
    _emit_service_connected();
    UtilityFunctions::print("Connected to Aetheris Scene Service");
}

void AetherisBus::disconnect_from_service() {
    UtilityFunctions::print("Disconnecting from Aetheris Scene Service");
    
    scene_service_connected = false;
    _emit_service_disconnected();
    UtilityFunctions::print("Disconnected from Aetheris Scene Service");
}

bool AetherisBus::is_connected() const {
    return scene_service_connected;
}

void AetherisBus::set_service_url(const String& url) {
    scene_service_url = url;
}

String AetherisBus::get_service_url() const {
    return scene_service_url;
}

void AetherisBus::set_connection_timeout(double timeout) {
    connection_timeout = timeout;
}

double AetherisBus::get_connection_timeout() const {
    return connection_timeout;
}

void AetherisBus::set_reconnect_interval(double interval) {
    reconnect_interval = interval;
}

double AetherisBus::get_reconnect_interval() const {
    return reconnect_interval;
}

void AetherisBus::set_max_reconnect_attempts(int attempts) {
    max_reconnect_attempts = attempts;
}

int AetherisBus::get_max_reconnect_attempts() const {
    return max_reconnect_attempts;
}

Dictionary AetherisBus::send_scene_request(const String& request_type, const Dictionary& data) {
    total_requests++;
    
    if (!scene_service_connected) {
        failed_requests++;
        _emit_service_error("Scene service not connected");
        return Dictionary();
    }
    
    // This would send the actual request to the scene service
    // For now, we'll simulate the response
    Dictionary response;
    response["success"] = true;
    response["request_type"] = request_type;
    response.merge(data);
    
    successful_requests++;
    _handle_scene_response(request_type, response);
    
    return response;
}

void AetherisBus::subscribe_to_events(const Array& event_types) {
    UtilityFunctions::print("Subscribing to events: ", event_types);
    
    for (int i = 0; i < event_types.size(); i++) {
        String event_type = event_types[i];
        if (subscribed_events.find(event_type) == -1) {
            subscribed_events.append(event_type);
        }
    }
}

void AetherisBus::unsubscribe_from_events(const Array& event_types) {
    UtilityFunctions::print("Unsubscribing from events: ", event_types);
    
    for (int i = 0; i < event_types.size(); i++) {
        String event_type = event_types[i];
        int index = subscribed_events.find(event_type);
        if (index != -1) {
            subscribed_events.remove_at(index);
        }
    }
}

Array AetherisBus::get_subscribed_events() const {
    return subscribed_events;
}

Dictionary AetherisBus::send_policy_check(const String& action, const Dictionary& context) {
    if (!scene_service_connected) {
        Dictionary response;
        response["allowed"] = false;
        response["reason"] = "Service not connected";
        return response;
    }
    
    // This would send a policy check request
    // For now, we'll simulate approval
    Dictionary response;
    response["allowed"] = true;
    response["action"] = action;
    response["context"] = context;
    
    _emit_policy_event("checked", action, response);
    
    return response;
}

Dictionary AetherisBus::send_capability_check(const String& capability, const Dictionary& context) {
    if (!scene_service_connected) {
        Dictionary response;
        response["allowed"] = false;
        response["reason"] = "Service not connected";
        return response;
    }
    
    // This would send a capability check request
    // For now, we'll simulate approval
    Dictionary response;
    response["allowed"] = true;
    response["capability"] = capability;
    response["context"] = context;
    
    return response;
}

void AetherisBus::register_event_handler(const String& event_type, const Callable& handler) {
    event_handlers[event_type] = handler;
    UtilityFunctions::print("Registered event handler for: ", event_type);
}

void AetherisBus::unregister_event_handler(const String& event_type) {
    event_handlers.erase(event_type);
    UtilityFunctions::print("Unregistered event handler for: ", event_type);
}

void AetherisBus::emit_event(const String& event_type, const Dictionary& data) {
    _process_event(event_type, data);
}

Dictionary AetherisBus::get_connection_status() const {
    Dictionary status;
    status["connected"] = scene_service_connected;
    status["url"] = scene_service_url;
    status["reconnect_attempts"] = reconnect_attempts;
    status["max_reconnect_attempts"] = max_reconnect_attempts;
    return status;
}

Dictionary AetherisBus::get_statistics() const {
    Dictionary stats;
    stats["total_requests"] = total_requests;
    stats["successful_requests"] = successful_requests;
    stats["failed_requests"] = failed_requests;
    stats["average_response_time"] = average_response_time;
    stats["subscribed_events"] = subscribed_events.size();
    return stats;
}

void AetherisBus::reset_statistics() {
    total_requests = 0;
    successful_requests = 0;
    failed_requests = 0;
    average_response_time = 0.0;
    UtilityFunctions::print("Statistics reset");
}

void AetherisBus::_setup_connection_monitoring() {
    // This would set up actual connection monitoring
    // For now, it's a placeholder
}

void AetherisBus::_attempt_reconnect() {
    if (reconnect_attempts >= max_reconnect_attempts) {
        UtilityFunctions::print("Max reconnect attempts reached");
        _emit_service_error("Max reconnect attempts reached");
        return;
    }
    
    reconnect_attempts++;
    UtilityFunctions::print("Attempting to reconnect (", reconnect_attempts, "/", max_reconnect_attempts, ")");
    
    // Try to reconnect
    connect_to_service();
}

void AetherisBus::_on_connection_lost() {
    UtilityFunctions::print("Connection to scene service lost");
    scene_service_connected = false;
    _emit_service_disconnected();
    
    // Attempt to reconnect
    _attempt_reconnect();
}

void AetherisBus::_on_service_error(const String& error_message) {
    UtilityFunctions::print("Scene service error: ", error_message);
    _emit_service_error(error_message);
    
    // Attempt to reconnect on error
    _attempt_reconnect();
}

void AetherisBus::_handle_scene_response(const String& request_type, const Dictionary& response) {
    // This would handle responses from the scene service
    // For now, it's a placeholder
}

void AetherisBus::_process_event(const String& event_type, const Dictionary& data) {
    // Check if we're subscribed to this event type
    if (subscribed_events.find(event_type) == -1) {
        return;
    }
    
    // Emit the appropriate signal based on event type
    if (event_type.begins_with("node_")) {
        String node_id = data.get("node_id", "");
        _emit_node_event(event_type, node_id, data);
    } else if (event_type.begins_with("avatar_")) {
        String avatar_id = data.get("avatar_id", "");
        _emit_avatar_event(event_type, avatar_id, data);
    } else if (event_type.begins_with("policy_")) {
        String action = data.get("action", "");
        _emit_policy_event(event_type, action, data);
    } else if (event_type.begins_with("snapshot_")) {
        String snapshot_id = data.get("snapshot_id", "");
        _emit_snapshot_event(event_type, snapshot_id, data);
    }
    
    // Call registered event handler if available
    if (event_handlers.has(event_type)) {
        Callable handler = event_handlers[event_type];
        handler.call(data);
    }
}

void AetherisBus::_emit_service_connected() {
    emit_signal(SIGNAL_SERVICE_CONNECTED);
}

void AetherisBus::_emit_service_disconnected() {
    emit_signal(SIGNAL_SERVICE_DISCONNECTED);
}

void AetherisBus::_emit_service_error(const String& error_message) {
    emit_signal(SIGNAL_SERVICE_ERROR, error_message);
}

void AetherisBus::_emit_node_event(const String& event_type, const String& node_id, const Dictionary& data) {
    emit_signal(SIGNAL_NODE_EVENT, event_type, node_id, data);
}

void AetherisBus::_emit_avatar_event(const String& event_type, const String& avatar_id, const Dictionary& data) {
    emit_signal(SIGNAL_AVATAR_EVENT, event_type, avatar_id, data);
}

void AetherisBus::_emit_policy_event(const String& event_type, const String& action, const Dictionary& data) {
    emit_signal(SIGNAL_POLICY_EVENT, event_type, action, data);
}

void AetherisBus::_emit_snapshot_event(const String& event_type, const String& snapshot_id, const Dictionary& data) {
    emit_signal(SIGNAL_SNAPSHOT_EVENT, event_type, snapshot_id, data);
}
