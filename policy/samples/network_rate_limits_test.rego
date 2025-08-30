# Unit Tests for Network Rate Limits Policy
#
# Tests various scenarios for network access controls including:
# - Request rate limiting
# - Bandwidth throttling  
# - Protocol restrictions
# - Geographic controls
# - DDoS protection

package polymera.network.rate_limits

import rego.v1

# Test data setup
test_users := {
    "user123": {
        "tier": "premium",
        "role": "developer",
        "average_request_size": 1024
    },
    "user456": {
        "tier": "basic", 
        "role": "user",
        "average_request_size": 512
    },
    "user789": {
        "tier": "enterprise",
        "role": "admin",
        "average_request_size": 2048
    }
}

test_policy_config := {
    "rate_limits": {
        "premium": {
            "requests_per_minute": 1000,
            "ip_requests_per_minute": 500,
            "endpoint_requests_per_minute": 200,
            "max_bandwidth_bps": 10485760  # 10 MB/s
        },
        "basic": {
            "requests_per_minute": 100,
            "ip_requests_per_minute": 50,
            "endpoint_requests_per_minute": 20,
            "max_bandwidth_bps": 1048576  # 1 MB/s
        },
        "enterprise": {
            "requests_per_minute": 5000,
            "ip_requests_per_minute": 2000,
            "endpoint_requests_per_minute": 1000,
            "max_bandwidth_bps": 52428800  # 50 MB/s
        },
        "default": {
            "requests_per_minute": 10,
            "ip_requests_per_minute": 5,
            "endpoint_requests_per_minute": 2,
            "max_bandwidth_bps": 102400  # 100 KB/s
        }
    },
    "permissions": {
        "developer": {
            "allowed_protocols": ["HTTP", "HTTPS", "QUIC", "WebSocket"],
            "allowed_endpoints": ["api.polymera.os/*", "dev.polymera.os/*"],
            "blocked_endpoints": ["admin.polymera.os/*"],
            "allowed_countries": ["US", "CA", "GB", "DE"],
            "blocked_countries": ["CN", "RU"]
        },
        "user": {
            "allowed_protocols": ["HTTP", "HTTPS"],
            "allowed_endpoints": ["api.polymera.os/public/*"],
            "blocked_endpoints": ["api.polymera.os/admin/*", "dev.polymera.os/*"],
            "allowed_countries": [],
            "blocked_countries": ["CN", "RU", "IR"]
        },
        "admin": {
            "allowed_protocols": ["HTTP", "HTTPS", "QUIC", "WebSocket", "P2P"],
            "allowed_endpoints": ["*"],
            "blocked_endpoints": [],
            "allowed_countries": [],
            "blocked_countries": []
        },
        "default": {
            "allowed_protocols": ["HTTP", "HTTPS"],
            "allowed_endpoints": ["api.polymera.os/public/*"],
            "blocked_endpoints": ["*admin*", "*dev*"],
            "allowed_countries": [],
            "blocked_countries": ["CN", "RU", "IR", "KP"]
        }
    },
    "trusted_services": ["auth_service", "monitoring_service", "backup_service"],
    "monitoring_endpoints": ["/health", "/metrics", "/status"],
    "monitoring_sources": ["192.168.1.100", "10.0.0.50"],
    "known_bot_user_agents": ["bot", "spider", "crawler", "scraper"],
    "ddos_threshold": 10000,
    "emergency_rate_limit": 5,
    "priority_users": ["user789"]
}

test_request_logs := {
    "user": {
        "user123": [
            {"timestamp": "2024-01-15T14:58:30Z", "request_size": 1024, "response_size": 2048},
            {"timestamp": "2024-01-15T14:59:00Z", "request_size": 1024, "response_size": 2048},
            {"timestamp": "2024-01-15T14:59:30Z", "request_size": 1024, "response_size": 2048}
        ],
        "user456": [
            {"timestamp": "2024-01-15T14:59:00Z", "request_size": 512, "response_size": 1024},
            {"timestamp": "2024-01-15T14:59:30Z", "request_size": 512, "response_size": 1024}
        ]
    },
    "ip": {
        "192.168.1.10": [
            {"timestamp": "2024-01-15T14:59:30Z"},
            {"timestamp": "2024-01-15T14:59:45Z"}
        ]
    },
    "endpoint": {
        "api.polymera.os/users": [
            {"timestamp": "2024-01-15T14:59:30Z"},
            {"timestamp": "2024-01-15T14:59:45Z"}
        ]
    }
}

test_geoip := {
    "192.168.1.10": {"country": "US"},
    "10.0.0.50": {"country": "US"},
    "203.0.113.1": {"country": "CN"},
    "198.51.100.1": {"country": "GB"}
}

test_domain_geo := {
    "api.polymera.os": {"country": "US"},
    "dev.polymera.os": {"country": "US"},
    "admin.polymera.os": {"country": "US"},
    "suspicious.example.com": {"country": "CN"}
}

# Mock data for testing
data := {
    "users": test_users,
    "policy_config": test_policy_config,
    "request_logs": test_request_logs,
    "geoip": test_geoip,
    "domain_geo": test_domain_geo,
    "security": {
        "blacklisted_users": ["blacklisted_user"],
        "blacklisted_ips": ["203.0.113.10"]
    }
}

# Test cases for ALLOW scenarios

test_allow_valid_network_request if {
    allow with input as {
        "operation": "network_request",
        "user_id": "user123",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 1024,
        "timestamp": "2024-01-15T15:00:00Z",
        "user_agent": "Mozilla/5.0"
    }
    with data as data
}

test_allow_internal_communication if {
    allow with input as {
        "operation": "internal_communication",
        "source_service": "auth_service",
        "destination_service": "monitoring_service",
        "service_token": "svc_internal_token_12345"
    }
    with data as data
}

test_allow_monitoring_request if {
    allow with input as {
        "operation": "health_check",
        "source_ip": "192.168.1.100",
        "endpoint": "/health",
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
}

test_allow_admin_user_unrestricted if {
    allow with input as {
        "operation": "network_request",
        "user_id": "user789",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "admin.polymera.os/dashboard",
        "protocol": "HTTPS",
        "request_size": 2048,
        "timestamp": "2024-01-15T15:00:00Z",
        "user_agent": "Admin Dashboard"
    }
    with data as data
}

test_allow_websocket_for_developer if {
    allow with input as {
        "operation": "network_request",
        "user_id": "user123",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "dev.polymera.os/websocket",
        "protocol": "WebSocket",
        "request_size": 512,
        "timestamp": "2024-01-15T15:00:00Z",
        "user_agent": "Developer Tools"
    }
    with data as data
}

# Test cases for DENY scenarios

test_deny_invalid_input_missing_user if {
    not allow with input as {
        "operation": "network_request",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 1024,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
}

test_deny_invalid_protocol if {
    not allow with input as {
        "operation": "network_request",
        "user_id": "user123",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "INVALID_PROTOCOL",
        "request_size": 1024,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
}

test_deny_rate_limit_exceeded if {
    # Simulate user with many recent requests
    enhanced_data := json.patch(data, [{
        "op": "replace",
        "path": "/request_logs/user/user456",
        "value": [
            {"timestamp": "2024-01-15T14:59:00Z", "request_size": 512},
            {"timestamp": "2024-01-15T14:59:05Z", "request_size": 512},
            {"timestamp": "2024-01-15T14:59:10Z", "request_size": 512},
            {"timestamp": "2024-01-15T14:59:15Z", "request_size": 512},
            {"timestamp": "2024-01-15T14:59:20Z", "request_size": 512}
        ] + [{"timestamp": sprintf("2024-01-15T14:59:%02dZ", [i]), "request_size": 512} | 
             i := numbers.range(25, 55)[_]]  # 30+ more requests
    }])
    
    not allow with input as {
        "operation": "network_request",
        "user_id": "user456",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 512,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as enhanced_data
}

test_deny_blocked_endpoint if {
    not allow with input as {
        "operation": "network_request",
        "user_id": "user456",  # Basic user
        "source_ip": "192.168.1.10",
        "destination_endpoint": "admin.polymera.os/dashboard",  # Admin endpoint
        "protocol": "HTTPS",
        "request_size": 512,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
}

test_deny_protocol_not_allowed if {
    not allow with input as {
        "operation": "network_request",
        "user_id": "user456",  # Basic user
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/public/data",
        "protocol": "WebSocket",  # Not allowed for basic users
        "request_size": 512,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
}

test_deny_geographic_restriction if {
    not allow with input as {
        "operation": "network_request",
        "user_id": "user456",
        "source_ip": "203.0.113.1",  # China IP
        "destination_endpoint": "api.polymera.os/public/data",
        "protocol": "HTTPS",
        "request_size": 512,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
}

test_deny_blacklisted_user if {
    not allow with input as {
        "operation": "network_request",
        "user_id": "blacklisted_user",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 1024,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
}

test_deny_blacklisted_ip if {
    not allow with input as {
        "operation": "network_request",
        "user_id": "user123",
        "source_ip": "203.0.113.10",  # Blacklisted IP
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 1024,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
}

test_deny_bandwidth_limit_exceeded if {
    # Simulate user with high bandwidth usage
    enhanced_data := json.patch(data, [{
        "op": "replace", 
        "path": "/request_logs/user/user456",
        "value": [
            {"timestamp": "2024-01-15T14:59:30Z", "request_size": 500000, "response_size": 500000},
            {"timestamp": "2024-01-15T14:59:40Z", "request_size": 500000, "response_size": 500000},
            {"timestamp": "2024-01-15T14:59:50Z", "request_size": 500000, "response_size": 500000}
        ]
    }])
    
    not allow with input as {
        "operation": "network_request",
        "user_id": "user456",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/public/data",
        "protocol": "HTTPS",
        "request_size": 1000000,  # Large request that would exceed bandwidth
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as enhanced_data
}

test_deny_suspicious_user_agent if {
    not allow with input as {
        "operation": "network_request",
        "user_id": "user123",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 1024,
        "timestamp": "2024-01-15T15:00:00Z",
        "user_agent": "bot/1.0"  # Suspicious bot user agent
    }
    with data as data
}

# Test DDoS protection

test_deny_ddos_protection_non_priority if {
    # Simulate DDoS conditions
    enhanced_data := json.patch(data, [{
        "op": "add",
        "path": "/request_logs/user/ddos_users",
        "value": {sprintf("user%d", [i]): [
            {"timestamp": "2024-01-15T14:59:59Z"}
        ] | i := numbers.range(1, 11000)[_]}  # 11000 users with recent requests
    }])
    
    not allow with input as {
        "operation": "network_request",
        "user_id": "user456",  # Non-priority user
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 1024,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as enhanced_data
}

test_allow_ddos_protection_priority_user if {
    # Simulate DDoS conditions but allow priority user
    enhanced_data := json.patch(data, [{
        "op": "add",
        "path": "/request_logs/user/ddos_users", 
        "value": {sprintf("user%d", [i]): [
            {"timestamp": "2024-01-15T14:59:59Z"}
        ] | i := numbers.range(1, 11000)[_]}
    }])
    
    allow with input as {
        "operation": "network_request",
        "user_id": "user789",  # Priority user
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 1024,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as enhanced_data
}

# Test rationale generation

test_rationale_for_approval if {
    reasons := rationale with input as {
        "operation": "network_request",
        "user_id": "user123",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 1024,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
    
    count(reasons) > 0
    "network request approved" in reasons
}

test_rationale_for_denial_rate_limit if {
    enhanced_data := json.patch(data, [{
        "op": "replace",
        "path": "/request_logs/user/user456",
        "value": [{"timestamp": sprintf("2024-01-15T14:59:%02dZ", [i]), "request_size": 512} | 
                 i := numbers.range(0, 120)[_]]  # 120 requests in last minute
    }])
    
    reasons := rationale with input as {
        "operation": "network_request", 
        "user_id": "user456",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "api.polymera.os/users",
        "protocol": "HTTPS",
        "request_size": 512,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as enhanced_data
    
    count(reasons) > 0
    some reason in reasons
    contains(reason, "rate limit exceeded")
}

test_rationale_for_denial_blocked_endpoint if {
    reasons := rationale with input as {
        "operation": "network_request",
        "user_id": "user456",
        "source_ip": "192.168.1.10", 
        "destination_endpoint": "admin.polymera.os/dashboard",
        "protocol": "HTTPS",
        "request_size": 512,
        "timestamp": "2024-01-15T15:00:00Z"
    }
    with data as data
    
    count(reasons) > 0
    some reason in reasons
    contains(reason, "endpoint not accessible")
}

# Test helper functions

test_count_recent_requests if {
    count := count_recent_requests("user123", "user", 120) with data as data  # Last 2 minutes
    count == 3  # From test data
}

test_count_recent_requests_no_data if {
    count := count_recent_requests("unknown_user", "user", 60) with data as data
    count == 0
}

test_calculate_bandwidth_usage if {
    bandwidth := calculate_bandwidth_usage("user123", 120) with data as data  # Last 2 minutes
    expected := (3 * (1024 + 2048)) / 120  # 3 requests with total 3072 bytes each
    bandwidth == expected
}

test_get_rate_limits_premium if {
    limits := get_rate_limits("user123") with data as data
    limits.requests_per_minute == 1000
    limits.max_bandwidth_bps == 10485760
}

test_get_rate_limits_basic if {
    limits := get_rate_limits("user456") with data as data
    limits.requests_per_minute == 100
    limits.max_bandwidth_bps == 1048576
}

test_get_user_permissions_developer if {
    permissions := get_user_permissions("user123") with data as data
    "WebSocket" in permissions.allowed_protocols
    "dev.polymera.os/*" in permissions.allowed_endpoints
}

test_get_user_permissions_user if {
    permissions := get_user_permissions("user456") with data as data
    not ("WebSocket" in permissions.allowed_protocols)
    "api.polymera.os/admin/*" in permissions.blocked_endpoints
}

test_matches_endpoint_pattern_wildcard if {
    matches_endpoint_pattern("api.polymera.os/users/123", "api.polymera.os/*")
}

test_matches_endpoint_pattern_exact if {
    matches_endpoint_pattern("api.polymera.os/users", "api.polymera.os/users")
}

test_get_country_from_ip if {
    country := get_country_from_ip("192.168.1.10") with data as data
    country == "US"
}

test_get_country_from_endpoint if {
    country := get_country_from_endpoint("https://api.polymera.os/users") with data as data
    country == "US"
}

# Emergency bypass tests

test_allow_emergency_bypass if {
    allow with input as {
        "operation": "emergency_communication",
        "user_id": "user123",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "emergency.polymera.os/alert",
        "protocol": "HTTPS",
        "request_size": 10000000,  # Exceeds all limits
        "timestamp": "2024-01-15T15:00:00Z",
        "emergency_token": "net_emergency_12345",
        "admin_user_id": "admin123"
    }
    with data as {
        "users": test_users,
        "policy_config": test_policy_config,
        "request_logs": test_request_logs,
        "geoip": test_geoip,
        "domain_geo": test_domain_geo,
        "admins": {
            "admin123": {"emergency_network_access": true}
        }
    }
}

test_deny_invalid_emergency_bypass if {
    not allow with input as {
        "operation": "emergency_communication",
        "user_id": "user123",
        "source_ip": "192.168.1.10",
        "destination_endpoint": "emergency.polymera.os/alert",
        "protocol": "HTTPS",
        "request_size": 10000000,
        "timestamp": "2024-01-15T15:00:00Z",
        "emergency_token": "invalid_token",
        "admin_user_id": "admin123"
    }
    with data as {
        "users": test_users,
        "policy_config": test_policy_config,
        "request_logs": test_request_logs,
        "geoip": test_geoip,
        "domain_geo": test_domain_geo,
        "admins": {
            "admin123": {"emergency_network_access": false}
        }
    }
}
