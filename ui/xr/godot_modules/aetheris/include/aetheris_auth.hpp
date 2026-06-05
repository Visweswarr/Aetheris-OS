//! Aetheris Authentication Module for Godot
//! 
//! This module provides authentication and session management capabilities
//! for the Aetheris XR scene service, including DID session binding,
//! capability token management, and policy enforcement.

#pragma once

#include <godot_cpp/classes/ref_counted.hpp>
#include <godot_cpp/classes/node.hpp>
#include <godot_cpp/variant/string.hpp>
#include <godot_cpp/variant/array.hpp>
#include <godot_cpp/variant/dictionary.hpp>
#include <godot_cpp/core/binder_common.hpp>

using namespace godot;

/// Authentication and session management for Aetheris XR
class AetherisAuth : public RefCounted {
    GDCLASS(AetherisAuth, RefCounted)

private:
    /// Current session ID
    String session_id;
    
    /// Current capability token
    String cap_token;
    
    /// User DID
    String user_did;
    
    /// Session expiration timestamp
    uint64_t session_expires_at;
    
    /// Capability token expiration timestamp
    uint64_t cap_expires_at;
    
    /// Whether session is active
    bool is_session_active;
    
    /// Whether capability token is valid
    bool is_cap_valid;
    
    /// RPC service connection
    void* rpc_service_connection;

protected:
    static void _bind_methods();

public:
    AetherisAuth();
    ~AetherisAuth();

    /// Begin a new DID session
    /// @param did User DID
    /// @param proof Authentication proof (JWT)
    /// @param nonce Nonce for replay protection
    /// @param avatar_profile Optional avatar profile
    /// @param ttl_seconds Session TTL in seconds
    /// @return Dictionary with session_id, avatar_id, expires_at
    Dictionary begin_session(const String& did, const String& proof, const String& nonce, 
                           const Dictionary& avatar_profile = Dictionary(), int ttl_seconds = 3600);

    /// End the current session
    /// @param reason Optional reason for termination
    /// @return Dictionary with success status and timestamp
    Dictionary end_session(const String& reason = "");

    /// Issue a capability token for the current session
    /// @param scopes Array of granted scopes
    /// @param ttl_seconds Token TTL in seconds
    /// @return Dictionary with cap_token and expires_at
    Dictionary issue_cap(const Array& scopes, int ttl_seconds = 300);

    /// Refresh the current session
    /// @param ttl_seconds New TTL in seconds
    /// @return Dictionary with success status and new expires_at
    Dictionary refresh_session(int ttl_seconds = 3600);

    /// Validate the current session
    /// @return Dictionary with is_valid, session info, and error
    Dictionary validate_session();

    /// Validate the current capability token
    /// @return Dictionary with is_valid, token info, and error
    Dictionary validate_cap();

    /// Check if the current session has a specific scope
    /// @param scope Scope to check
    /// @return True if session has the scope
    bool has_scope(const String& scope);

    /// Check if the current session can perform an action
    /// @param action Action to check
    /// @param scope Scope to check
    /// @return True if action is allowed
    bool can_perform_action(const String& action, const String& scope);

    /// Get current session information
    /// @return Dictionary with session details
    Dictionary get_session_info();

    /// Get current capability token information
    /// @return Dictionary with token details
    Dictionary get_cap_info();

    /// Get user DID
    /// @return User DID string
    String get_user_did() const;

    /// Get session ID
    /// @return Session ID string
    String get_session_id() const;

    /// Check if session is active
    /// @return True if session is active
    bool is_active() const;

    /// Check if capability token is valid
    /// @return True if capability token is valid
    bool is_capability_valid() const;

    /// Get session expiration timestamp
    /// @return Expiration timestamp
    uint64_t get_session_expires_at() const;

    /// Get capability token expiration timestamp
    /// @return Expiration timestamp
    uint64_t get_cap_expires_at() const;

    /// Set RPC service connection
    /// @param connection RPC service connection pointer
    void set_rpc_connection(void* connection);

    /// Get RPC service connection
    /// @return RPC service connection pointer
    void* get_rpc_connection() const;

private:
    /// Make RPC call to the scene service
    /// @param method RPC method name
    /// @param params RPC parameters
    /// @return RPC response
    Dictionary make_rpc_call(const String& method, const Dictionary& params);

    /// Parse RPC response
    /// @param response RPC response
    /// @return Parsed response dictionary
    Dictionary parse_rpc_response(const Dictionary& response);

    /// Update session state from response
    /// @param response Session response
    void update_session_state(const Dictionary& response);

    /// Update capability token state from response
    /// @param response Capability response
    void update_cap_state(const Dictionary& response);

    /// Check if timestamp is expired
    /// @param timestamp Timestamp to check
    /// @return True if expired
    bool is_timestamp_expired(uint64_t timestamp) const;

    /// Get current timestamp
    /// @return Current timestamp
    uint64_t get_current_timestamp() const;

    /// Emit authentication event
    /// @param event_type Event type
    /// @param event_data Event data
    void emit_auth_event(const String& event_type, const Dictionary& event_data);
};
