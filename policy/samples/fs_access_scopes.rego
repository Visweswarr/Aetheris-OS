# Filesystem Access Scopes Policy
#
# Enforces filesystem access controls including:
# - Path-based access restrictions
# - Operation-specific permissions (read/write/execute/delete)
# - User/role-based directory scoping
# - Sensitive file protection
# - Quota and size limits

package polymera.filesystem.access_scopes

import rego.v1

# Default deny - all filesystem operations must be explicitly allowed
default allow := false

# Main authorization decision for filesystem operations
allow if {
    input.operation in ["read", "write", "execute", "delete", "list", "stat"]
    valid_fs_input
    path_accessible
    operation_permitted
    within_size_limits
    within_quota_limits
    not accessing_sensitive_file
    not violating_time_restrictions
}

# Allow system operations with proper authorization
allow if {
    input.operation in ["system_backup", "system_restore", "maintenance"]
    input.system_role in ["backup_service", "maintenance_service"]
    valid_system_credentials
}

# Allow monitoring and audit operations
allow if {
    input.operation in ["audit_read", "log_collection", "health_check"]
    input.service_role == "monitoring"
    from_trusted_monitor
}

# Input validation for filesystem operations
valid_fs_input if {
    input.user_id
    input.path
    input.operation
    is_absolute_path(input.path)
    normalized_path := normalize_path(input.path)
    input.timestamp
    time.parse_rfc3339_ns(input.timestamp)
}

valid_system_credentials if {
    input.system_token
    # In production, verify system token cryptographically
    startswith(input.system_token, "fs_sys_")
}

from_trusted_monitor if {
    input.source_service in data.policy_config.trusted_monitoring_services
}

# Path accessibility based on user scopes
path_accessible if {
    user_scopes := get_user_scopes(input.user_id)
    normalized_path := normalize_path(input.path)
    
    some scope in user_scopes
    path_within_scope(normalized_path, scope)
}

# Operation-specific permissions
operation_permitted if {
    user_permissions := get_user_permissions(input.user_id)
    path_permissions := get_path_permissions(input.path, input.user_id)
    
    operation_allowed_by_user(input.operation, user_permissions)
    operation_allowed_by_path(input.operation, path_permissions)
}

# Size limits for write operations
within_size_limits if {
    input.operation != "write"
}

within_size_limits if {
    input.operation == "write"
    input.file_size
    
    user_limits := get_user_limits(input.user_id)
    max_file_size := user_limits.max_file_size
    
    input.file_size <= max_file_size
}

# Quota enforcement
within_quota_limits if {
    input.operation in ["read", "list", "stat", "execute"]
}

within_quota_limits if {
    input.operation in ["write", "delete"]
    
    current_usage := calculate_user_usage(input.user_id)
    user_limits := get_user_limits(input.user_id)
    quota_limit := user_limits.disk_quota
    
    input.operation == "delete"
}

within_quota_limits if {
    input.operation == "write"
    input.file_size
    
    current_usage := calculate_user_usage(input.user_id)
    user_limits := get_user_limits(input.user_id)
    quota_limit := user_limits.disk_quota
    
    (current_usage + input.file_size) <= quota_limit
}

# Sensitive file protection
accessing_sensitive_file if {
    sensitive_patterns := data.policy_config.sensitive_file_patterns
    normalized_path := normalize_path(input.path)
    
    some pattern in sensitive_patterns
    matches_file_pattern(normalized_path, pattern)
}

# Time-based access restrictions
violating_time_restrictions if {
    current_hour := time.hour(time.now_ns())
    user_restrictions := get_user_restrictions(input.user_id)
    restricted_hours := user_restrictions.restricted_hours
    
    input.operation in ["write", "delete", "execute"]
    current_hour in restricted_hours
}

# Helper functions
normalize_path(path) := normalized if {
    # Remove double slashes, resolve .., etc.
    # This is a simplified version - production should use proper path normalization
    parts := split(path, "/")
    filtered_parts := [part | 
        part := parts[_]
        part != ""
        part != "."
    ]
    
    # Resolve .. references
    resolved_parts := resolve_parent_refs(filtered_parts, [])
    normalized := sprintf("/%s", [concat("/", resolved_parts)])
}

resolve_parent_refs([], acc) := acc

resolve_parent_refs(parts, acc) if {
    count(parts) > 0
    head := parts[0]
    tail := array.slice(parts, 1, count(parts))
    
    head == ".."
    count(acc) > 0
    new_acc := array.slice(acc, 0, count(acc) - 1)
    resolve_parent_refs(tail, new_acc)
}

resolve_parent_refs(parts, acc) if {
    count(parts) > 0
    head := parts[0]
    tail := array.slice(parts, 1, count(parts))
    
    head != ".."
    new_acc := array.concat(acc, [head])
    resolve_parent_refs(tail, new_acc)
}

is_absolute_path(path) if {
    startswith(path, "/")
}

get_user_scopes(user_id) := scopes if {
    user_role := data.users[user_id].role
    role_scopes := data.policy_config.role_scopes[user_role]
    user_specific_scopes := data.users[user_id].additional_scopes
    
    # Combine role-based and user-specific scopes
    scopes := array.concat(role_scopes, user_specific_scopes)
}

get_user_scopes(user_id) := data.policy_config.role_scopes.default if {
    not data.users[user_id].role
}

path_within_scope(path, scope) if {
    scope_type := scope.type
    scope_path := scope.path
    
    scope_type == "directory"
    startswith(path, scope_path)
}

path_within_scope(path, scope) if {
    scope_type := scope.type
    scope_pattern := scope.pattern
    
    scope_type == "pattern"
    matches_file_pattern(path, scope_pattern)
}

path_within_scope(path, scope) if {
    scope_type := scope.type
    allowed_paths := scope.paths
    
    scope_type == "explicit"
    path in allowed_paths
}

get_user_permissions(user_id) := permissions if {
    user_role := data.users[user_id].role
    permissions := data.policy_config.role_permissions[user_role]
}

get_user_permissions(user_id) := data.policy_config.role_permissions.default if {
    not data.users[user_id].role
}

get_path_permissions(path, user_id) := permissions if {
    # Check for path-specific permissions
    path_configs := data.policy_config.path_permissions
    
    some config in path_configs
    matches_file_pattern(path, config.pattern)
    user_in_allowed_groups(user_id, config.allowed_groups)
    permissions := config.permissions
}

get_path_permissions(path, user_id) := data.policy_config.default_path_permissions if {
    # No specific path permissions found
    not get_path_permissions(path, user_id)
}

user_in_allowed_groups(user_id, allowed_groups) if {
    user_groups := data.users[user_id].groups
    some group in user_groups
    group in allowed_groups
}

operation_allowed_by_user(operation, user_permissions) if {
    operation in user_permissions.allowed_operations
}

operation_allowed_by_path(operation, path_permissions) if {
    operation in path_permissions.allowed_operations
}

get_user_limits(user_id) := limits if {
    user_tier := data.users[user_id].tier
    limits := data.policy_config.user_limits[user_tier]
}

get_user_limits(user_id) := data.policy_config.user_limits.default if {
    not data.users[user_id].tier
}

get_user_restrictions(user_id) := restrictions if {
    user_profile := data.users[user_id]
    restrictions := user_profile.access_restrictions
}

get_user_restrictions(user_id) := {} if {
    not data.users[user_id].access_restrictions
}

calculate_user_usage(user_id) := usage if {
    user_files := data.file_usage[user_id]
    total_size := sum([file.size | file := user_files[_]])
    usage := total_size
}

calculate_user_usage(user_id) := 0 if {
    not data.file_usage[user_id]
}

matches_file_pattern(path, pattern) if {
    # Simple wildcard matching
    contains(pattern, "*")
    
    # Handle suffix wildcard (e.g., "*.txt")
    endswith(pattern, "*")
    prefix := trim_suffix(pattern, "*")
    startswith(path, prefix)
}

matches_file_pattern(path, pattern) if {
    contains(pattern, "*")
    
    # Handle prefix wildcard (e.g., "*.txt")
    startswith(pattern, "*")
    suffix := trim_prefix(pattern, "*")
    endswith(path, suffix)
}

matches_file_pattern(path, pattern) if {
    contains(pattern, "*")
    
    # Handle middle wildcard (e.g., "/home/*/documents")
    parts := split(pattern, "*")
    count(parts) == 2
    prefix := parts[0]
    suffix := parts[1]
    startswith(path, prefix)
    endswith(path, suffix)
}

matches_file_pattern(path, pattern) if {
    # Exact match
    not contains(pattern, "*")
    path == pattern
}

# Advanced permission checks
has_elevated_access(user_id) if {
    user_groups := data.users[user_id].groups
    elevated_groups := data.policy_config.elevated_access_groups
    
    some group in user_groups
    group in elevated_groups
}

is_system_file(path) if {
    system_directories := data.policy_config.system_directories
    
    some sys_dir in system_directories
    startswith(path, sys_dir)
}

is_shared_resource(path) if {
    shared_directories := data.policy_config.shared_directories
    
    some shared_dir in shared_directories
    startswith(path, shared_dir)
}

# Special access rules for system files
allow if {
    is_system_file(input.path)
    has_elevated_access(input.user_id)
    input.operation in ["read", "execute"]
    valid_fs_input
}

# Special access rules for shared resources
allow if {
    is_shared_resource(input.path)
    valid_fs_input
    shared_resource_access_allowed
}

shared_resource_access_allowed if {
    user_permissions := get_user_permissions(input.user_id)
    "shared_resource_access" in user_permissions.capabilities
}

# Temporary access grants
temporary_access_active(user_id, path) if {
    temp_grants := data.temporary_access[user_id]
    current_time := time.now_ns()
    
    some grant in temp_grants
    grant.path == path
    time.parse_rfc3339_ns(grant.expires_at) > current_time
}

allow if {
    temporary_access_active(input.user_id, input.path)
    valid_fs_input
    input.operation in ["read", "write"]
}

# Decision rationale for audit trails
rationale := reasons if {
    allow
    not temporary_access_active(input.user_id, input.path)
    not is_system_file(input.path)
    not is_shared_resource(input.path)
    reasons := [
        "filesystem access approved",
        sprintf("operation: %v on %v", [input.operation, input.path]),
        sprintf("user scopes: %v", [count(get_user_scopes(input.user_id))]),
        sprintf("permissions: %v", [get_user_permissions(input.user_id).allowed_operations]),
        sprintf("quota usage: %v/%v bytes", [
            calculate_user_usage(input.user_id),
            get_user_limits(input.user_id).disk_quota
        ])
    ]
}

rationale := reasons if {
    allow
    temporary_access_active(input.user_id, input.path)
    reasons := [
        "filesystem access approved via temporary grant",
        sprintf("operation: %v on %v", [input.operation, input.path])
    ]
}

rationale := reasons if {
    allow
    is_system_file(input.path)
    reasons := [
        "system file access approved",
        sprintf("elevated access: %v", [has_elevated_access(input.user_id)]),
        sprintf("operation: %v", [input.operation])
    ]
}

rationale := reasons if {
    allow
    is_shared_resource(input.path)
    reasons := [
        "shared resource access approved",
        sprintf("operation: %v", [input.operation])
    ]
}

rationale := reasons if {
    not allow
    reasons := denial_reasons
}

denial_reasons := ["invalid input parameters"] if {
    not valid_fs_input
}

denial_reasons := [reason] if {
    valid_fs_input
    not path_accessible
    user_scopes := get_user_scopes(input.user_id)
    reason := sprintf("path not accessible: %v (user has %v scopes)", [input.path, count(user_scopes)])
}

denial_reasons := [reason] if {
    valid_fs_input
    path_accessible
    not operation_permitted
    user_perms := get_user_permissions(input.user_id)
    reason := sprintf("operation not permitted: %v (user can: %v)", [
        input.operation, user_perms.allowed_operations
    ])
}

denial_reasons := [reason] if {
    valid_fs_input
    path_accessible
    operation_permitted
    not within_size_limits
    max_size := get_user_limits(input.user_id).max_file_size
    reason := sprintf("file size limit exceeded: %v > %v bytes", [input.file_size, max_size])
}

denial_reasons := [reason] if {
    valid_fs_input
    path_accessible
    operation_permitted
    within_size_limits
    not within_quota_limits
    usage := calculate_user_usage(input.user_id)
    quota := get_user_limits(input.user_id).disk_quota
    reason := sprintf("quota limit exceeded: %v + %v > %v bytes", [
        usage, input.file_size, quota
    ])
}

denial_reasons := [reason] if {
    accessing_sensitive_file
    reason := sprintf("access to sensitive file denied: %v", [input.path])
}

denial_reasons := [reason] if {
    violating_time_restrictions
    current_hour := time.hour(time.now_ns())
    reason := sprintf("time restriction violation: operation %v at hour %v", [
        input.operation, current_hour
    ])
}

# Emergency access override
emergency_access if {
    input.emergency_token
    input.admin_user_id
    verify_emergency_token(input.emergency_token, input.admin_user_id)
    input.justification
}

verify_emergency_token(token, admin_id) if {
    admin_data := data.admins[admin_id]
    admin_data.emergency_fs_access == true
    startswith(token, "fs_emergency_")
}

allow if {
    emergency_access
    valid_fs_input
}

rationale := [reason] if {
    emergency_access
    reason := sprintf("emergency filesystem access granted: %v", [input.justification])
}
