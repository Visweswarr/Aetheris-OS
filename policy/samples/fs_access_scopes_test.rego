# Unit Tests for Filesystem Access Scopes Policy
#
# Tests various scenarios for filesystem access controls including:
# - Path-based access restrictions
# - Operation-specific permissions
# - User/role-based scoping
# - Quota enforcement
# - Sensitive file protection

package polymera.filesystem.access_scopes

import rego.v1

# Test data setup
test_users := {
    "user123": {
        "role": "developer",
        "tier": "premium",
        "groups": ["developers", "beta_testers"],
        "additional_scopes": [
            {"type": "directory", "path": "/home/user123"},
            {"type": "pattern", "pattern": "/tmp/user123_*"}
        ],
        "access_restrictions": {
            "restricted_hours": [0, 1, 2, 3, 4, 5]  # Midnight to 5 AM
        }
    },
    "user456": {
        "role": "user",
        "tier": "basic",
        "groups": ["users"],
        "additional_scopes": [
            {"type": "directory", "path": "/home/user456"}
        ],
        "access_restrictions": {
            "restricted_hours": [22, 23, 0, 1, 2, 3, 4, 5, 6]  # 10 PM to 6 AM
        }
    },
    "user789": {
        "role": "admin",
        "tier": "enterprise", 
        "groups": ["admins", "system_operators"],
        "additional_scopes": [
            {"type": "explicit", "paths": ["/etc", "/var/log", "/opt"]}
        ],
        "access_restrictions": {}
    }
}

test_policy_config := {
    "role_scopes": {
        "developer": [
            {"type": "directory", "path": "/workspace"},
            {"type": "directory", "path": "/opt/dev"},
            {"type": "pattern", "pattern": "/tmp/build_*"}
        ],
        "user": [
            {"type": "directory", "path": "/home"},
            {"type": "directory", "path": "/tmp"}
        ],
        "admin": [
            {"type": "directory", "path": "/"},
            {"type": "pattern", "pattern": "*"}
        ],
        "default": [
            {"type": "directory", "path": "/home/public"}
        ]
    },
    "role_permissions": {
        "developer": {
            "allowed_operations": ["read", "write", "execute", "list", "stat"]
        },
        "user": {
            "allowed_operations": ["read", "write", "list", "stat"]
        },
        "admin": {
            "allowed_operations": ["read", "write", "execute", "delete", "list", "stat"]
        },
        "default": {
            "allowed_operations": ["read", "list", "stat"]
        }
    },
    "path_permissions": [
        {
            "pattern": "/etc/*",
            "allowed_groups": ["admins", "system_operators"],
            "permissions": {"allowed_operations": ["read", "list", "stat"]}
        },
        {
            "pattern": "/var/log/*",
            "allowed_groups": ["admins", "developers"],
            "permissions": {"allowed_operations": ["read", "list", "stat"]}
        },
        {
            "pattern": "/home/shared/*",
            "allowed_groups": ["developers", "users"],
            "permissions": {"allowed_operations": ["read", "write", "list", "stat"]}
        }
    ],
    "default_path_permissions": {
        "allowed_operations": ["read", "list", "stat"]
    },
    "user_limits": {
        "premium": {
            "max_file_size": 1073741824,  # 1 GB
            "disk_quota": 107374182400    # 100 GB
        },
        "basic": {
            "max_file_size": 104857600,   # 100 MB
            "disk_quota": 10737418240     # 10 GB
        },
        "enterprise": {
            "max_file_size": 5368709120,  # 5 GB
            "disk_quota": 1073741824000   # 1 TB
        },
        "default": {
            "max_file_size": 10485760,    # 10 MB
            "disk_quota": 1073741824      # 1 GB
        }
    },
    "sensitive_file_patterns": [
        "/etc/passwd",
        "/etc/shadow",
        "/etc/ssh/*",
        "*.key",
        "*.pem", 
        "/var/secrets/*",
        "*/config/database.yml"
    ],
    "system_directories": [
        "/bin", "/sbin", "/usr/bin", "/usr/sbin", "/boot", "/sys", "/proc"
    ],
    "shared_directories": [
        "/home/shared", "/opt/shared", "/var/shared"
    ],
    "elevated_access_groups": ["admins", "system_operators"],
    "trusted_monitoring_services": ["monitoring_service", "audit_service"]
}

test_file_usage := {
    "user123": [
        {"path": "/home/user123/project1", "size": 1048576},      # 1 MB
        {"path": "/home/user123/project2", "size": 2097152},      # 2 MB
        {"path": "/workspace/app1", "size": 5242880}              # 5 MB
    ],
    "user456": [
        {"path": "/home/user456/documents", "size": 524288},      # 512 KB
        {"path": "/home/user456/photos", "size": 10485760}       # 10 MB
    ]
}

test_temporary_access := {
    "user456": [
        {
            "path": "/opt/temp_project",
            "expires_at": "2024-01-15T18:00:00Z"  # Future expiration
        }
    ]
}

# Mock data for testing
data := {
    "users": test_users,
    "policy_config": test_policy_config,
    "file_usage": test_file_usage,
    "temporary_access": test_temporary_access
}

# Test cases for ALLOW scenarios

test_allow_user_home_directory_read if {
    allow with input as {
        "operation": "read",
        "user_id": "user123",
        "path": "/home/user123/document.txt",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_allow_developer_workspace_write if {
    allow with input as {
        "operation": "write",
        "user_id": "user123",
        "path": "/workspace/project/src/main.rs",
        "file_size": 1024,
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_allow_admin_system_file_read if {
    allow with input as {
        "operation": "read",
        "user_id": "user789",
        "path": "/etc/hosts",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_allow_shared_resource_access if {
    allow with input as {
        "operation": "write",
        "user_id": "user123",
        "path": "/home/shared/collaboration.txt",
        "file_size": 2048,
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_allow_temporary_access if {
    allow with input as {
        "operation": "read",
        "user_id": "user456",
        "path": "/opt/temp_project",
        "timestamp": "2024-01-15T16:00:00Z"  # Before expiration
    }
    with data as data
}

test_allow_system_backup if {
    allow with input as {
        "operation": "system_backup",
        "system_role": "backup_service",
        "path": "/var/backups/daily",
        "system_token": "fs_sys_backup_12345",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_allow_monitoring_audit if {
    allow with input as {
        "operation": "audit_read",
        "service_role": "monitoring",
        "source_service": "monitoring_service",
        "path": "/var/log/audit.log",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_allow_pattern_based_access if {
    allow with input as {
        "operation": "read",
        "user_id": "user123",
        "path": "/tmp/user123_build_001",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_allow_log_file_developer if {
    allow with input as {
        "operation": "read",
        "user_id": "user123",
        "path": "/var/log/application.log",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

# Test cases for DENY scenarios

test_deny_invalid_input_missing_user if {
    not allow with input as {
        "operation": "read",
        "path": "/home/user123/document.txt",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_relative_path if {
    not allow with input as {
        "operation": "read",
        "user_id": "user123",
        "path": "document.txt",  # Not absolute path
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_path_outside_scope if {
    not allow with input as {
        "operation": "read",
        "user_id": "user456",
        "path": "/workspace/secret_project/code.rs",  # Outside user's scope
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_operation_not_permitted if {
    not allow with input as {
        "operation": "execute",
        "user_id": "user456",  # User role doesn't allow execute
        "path": "/home/user456/script.sh",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_file_size_limit_exceeded if {
    not allow with input as {
        "operation": "write",
        "user_id": "user456",  # Basic tier: 100MB limit
        "path": "/home/user456/large_file.dat",
        "file_size": 209715200,  # 200 MB - exceeds limit
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_quota_limit_exceeded if {
    enhanced_data := json.patch(data, [{
        "op": "replace",
        "path": "/file_usage/user456",
        "value": [
            {"path": "/home/user456/large1", "size": 5368709120},  # 5 GB
            {"path": "/home/user456/large2", "size": 5368709120}   # 5 GB = 10 GB total
        ]
    }])
    
    not allow with input as {
        "operation": "write",
        "user_id": "user456",  # Basic tier: 10GB quota, already at limit
        "path": "/home/user456/new_file.txt",
        "file_size": 1048576,  # 1 MB - would exceed quota
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as enhanced_data
}

test_deny_sensitive_file_access if {
    not allow with input as {
        "operation": "read",
        "user_id": "user123",
        "path": "/etc/shadow",  # Sensitive file
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_ssh_key_access if {
    not allow with input as {
        "operation": "read",
        "user_id": "user456",
        "path": "/home/user456/.ssh/id_rsa",  # SSH private key
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_time_restriction_violation if {
    not allow with input as {
        "operation": "write",
        "user_id": "user123",
        "path": "/home/user123/document.txt",
        "file_size": 1024,
        "timestamp": "2024-01-15T03:00:00Z"  # 3 AM - restricted time
    }
    with data as data
}

test_deny_delete_operation_basic_user if {
    not allow with input as {
        "operation": "delete",
        "user_id": "user456",  # Basic user can't delete
        "path": "/home/user456/old_file.txt",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_admin_area_regular_user if {
    not allow with input as {
        "operation": "read",
        "user_id": "user456",
        "path": "/etc/passwd",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

test_deny_expired_temporary_access if {
    expired_data := json.patch(data, [{
        "op": "replace",
        "path": "/temporary_access/user456/0/expires_at",
        "value": "2024-01-15T12:00:00Z"  # Past expiration
    }])
    
    not allow with input as {
        "operation": "read",
        "user_id": "user456",
        "path": "/opt/temp_project",
        "timestamp": "2024-01-15T16:00:00Z"  # After expiration
    }
    with data as expired_data
}

test_deny_path_traversal_attempt if {
    not allow with input as {
        "operation": "read",
        "user_id": "user456",
        "path": "/home/user456/../user123/secret.txt",  # Path traversal
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
}

# Test rationale generation

test_rationale_for_approval if {
    reasons := rationale with input as {
        "operation": "read",
        "user_id": "user123",
        "path": "/home/user123/document.txt",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
    
    count(reasons) > 0
    "filesystem access approved" in reasons
}

test_rationale_for_denial_path_access if {
    reasons := rationale with input as {
        "operation": "read",
        "user_id": "user456",
        "path": "/workspace/secret_project/code.rs",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
    
    count(reasons) > 0
    some reason in reasons
    contains(reason, "path not accessible")
}

test_rationale_for_denial_operation if {
    reasons := rationale with input as {
        "operation": "execute",
        "user_id": "user456",
        "path": "/home/user456/script.sh",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
    
    count(reasons) > 0
    some reason in reasons
    contains(reason, "operation not permitted")
}

test_rationale_for_denial_size_limit if {
    reasons := rationale with input as {
        "operation": "write",
        "user_id": "user456",
        "path": "/home/user456/large_file.dat",
        "file_size": 209715200,
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
    
    count(reasons) > 0
    some reason in reasons
    contains(reason, "file size limit exceeded")
}

test_rationale_for_denial_sensitive_file if {
    reasons := rationale with input as {
        "operation": "read",
        "user_id": "user123",
        "path": "/etc/shadow",
        "timestamp": "2024-01-15T14:00:00Z"
    }
    with data as data
    
    count(reasons) > 0
    some reason in reasons
    contains(reason, "sensitive file")
}

test_rationale_for_denial_time_restriction if {
    reasons := rationale with input as {
        "operation": "write",
        "user_id": "user123",
        "path": "/home/user123/document.txt",
        "file_size": 1024,
        "timestamp": "2024-01-15T03:00:00Z"
    }
    with data as data
    
    count(reasons) > 0
    some reason in reasons
    contains(reason, "time restriction")
}

# Test helper functions

test_normalize_path_simple if {
    normalized := normalize_path("/home/user123/document.txt")
    normalized == "/home/user123/document.txt"
}

test_normalize_path_with_dots if {
    normalized := normalize_path("/home/user123/../user456/document.txt")
    normalized == "/home/user456/document.txt"
}

test_normalize_path_with_double_slash if {
    normalized := normalize_path("/home//user123/./document.txt")
    normalized == "/home/user123/document.txt"
}

test_is_absolute_path_true if {
    is_absolute_path("/home/user123/document.txt")
}

test_is_absolute_path_false if {
    not is_absolute_path("document.txt")
}

test_get_user_scopes_developer if {
    scopes := get_user_scopes("user123") with data as data
    count(scopes) > 2  # Role scopes + additional scopes
}

test_path_within_scope_directory if {
    scope := {"type": "directory", "path": "/home/user123"}
    path_within_scope("/home/user123/document.txt", scope)
}

test_path_within_scope_pattern if {
    scope := {"type": "pattern", "pattern": "/tmp/user123_*"}
    path_within_scope("/tmp/user123_build_001", scope)
}

test_path_within_scope_explicit if {
    scope := {"type": "explicit", "paths": ["/etc", "/var/log"]}
    path_within_scope("/etc", scope)
}

test_get_user_permissions_developer if {
    permissions := get_user_permissions("user123") with data as data
    "execute" in permissions.allowed_operations
}

test_get_user_permissions_user if {
    permissions := get_user_permissions("user456") with data as data
    not ("execute" in permissions.allowed_operations)
}

test_get_user_limits_premium if {
    limits := get_user_limits("user123") with data as data
    limits.max_file_size == 1073741824
}

test_get_user_limits_basic if {
    limits := get_user_limits("user456") with data as data
    limits.max_file_size == 104857600
}

test_calculate_user_usage if {
    usage := calculate_user_usage("user123") with data as data
    expected := 1048576 + 2097152 + 5242880  # Sum of file sizes
    usage == expected
}

test_matches_file_pattern_wildcard_suffix if {
    matches_file_pattern("/home/user123/document.txt", "*.txt")
}

test_matches_file_pattern_wildcard_prefix if {
    matches_file_pattern("/tmp/user123_build", "/tmp/user123_*")
}

test_matches_file_pattern_exact if {
    matches_file_pattern("/etc/passwd", "/etc/passwd")
}

test_has_elevated_access_admin if {
    has_elevated_access("user789") with data as data
}

test_has_elevated_access_developer if {
    not has_elevated_access("user123") with data as data  # Developer group not in elevated_access_groups
}

test_is_system_file if {
    is_system_file("/bin/bash") with data as data
}

test_is_shared_resource if {
    is_shared_resource("/home/shared/document.txt") with data as data
}

test_temporary_access_active if {
    temporary_access_active("user456", "/opt/temp_project") with data as data
}

test_temporary_access_expired if {
    expired_data := json.patch(data, [{
        "op": "replace",
        "path": "/temporary_access/user456/0/expires_at",
        "value": "2024-01-15T12:00:00Z"
    }])
    
    not temporary_access_active("user456", "/opt/temp_project") with data as expired_data
}

# Emergency access tests

test_allow_emergency_access if {
    allow with input as {
        "operation": "read",
        "user_id": "user456",
        "path": "/etc/passwd",  # Normally denied
        "timestamp": "2024-01-15T14:00:00Z",
        "emergency_token": "fs_emergency_12345",
        "admin_user_id": "admin123",
        "justification": "Security incident investigation"
    }
    with data as {
        "users": test_users,
        "policy_config": test_policy_config,
        "file_usage": test_file_usage,
        "temporary_access": test_temporary_access,
        "admins": {
            "admin123": {"emergency_fs_access": true}
        }
    }
}

test_deny_invalid_emergency_access if {
    not allow with input as {
        "operation": "read",
        "user_id": "user456",
        "path": "/etc/passwd",
        "timestamp": "2024-01-15T14:00:00Z",
        "emergency_token": "invalid_token",
        "admin_user_id": "admin123",
        "justification": "Security incident investigation"
    }
    with data as {
        "users": test_users,
        "policy_config": test_policy_config,
        "file_usage": test_file_usage,
        "temporary_access": test_temporary_access,
        "admins": {
            "admin123": {"emergency_fs_access": false}
        }
    }
}
