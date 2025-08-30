# Network Rate Limiting Policy
#
# Enforces network access controls including:
# - Request rate limiting per user/IP/endpoint
# - Bandwidth throttling based on user tier
# - Protocol-specific restrictions
# - Geographic access controls
# - DDoS protection measures

package polymera.network.rate_limits

import rego.v1

# Default deny - all network requests must be explicitly allowed
default allow := false

# Main authorization decision for network requests
allow if {
    input.operation == "network_request"
    valid_network_input
    within_request_rate_limits
    within_bandwidth_limits
    protocol_allowed
    endpoint_accessible
    geographic_access_allowed
    not suspicious_pattern
    not blacklisted_source
}

# Allow internal system communications
allow if {
    input.operation == "internal_communication"
    input.source_service in data.policy_config.trusted_services
    input.destination_service in data.policy_config.trusted_services
    valid_service_token
}

# Allow monitoring and health checks
allow if {
    input.operation in ["health_check", "metrics_collection"]
    input.endpoint in data.policy_config.monitoring_endpoints
    from_monitoring_source
}

# Input validation for network requests
valid_network_input if {
    input.user_id
    input.source_ip
    input.destination_endpoint
    input.protocol in ["HTTP", "HTTPS", "QUIC", "WebSocket", "P2P"]
    input.request_size >= 0
    input.timestamp
    time.parse_rfc3339_ns(input.timestamp)
}

valid_service_token if {
    input.service_token
    # In production, verify service token cryptographically
    startswith(input.service_token, "svc_")
    count(input.service_token) > 20
}

from_monitoring_source if {
    input.source_ip in data.policy_config.monitoring_sources
}

# Request rate limiting per user
within_request_rate_limits if {
    user_requests := count_recent_requests(input.user_id, "user", 60) # Last minute
    ip_requests := count_recent_requests(input.source_ip, "ip", 60)
    endpoint_requests := count_recent_requests(input.destination_endpoint, "endpoint", 60)
    
    user_limits := get_rate_limits(input.user_id)
    
    user_requests < user_limits.requests_per_minute
    ip_requests < user_limits.ip_requests_per_minute
    endpoint_requests < user_limits.endpoint_requests_per_minute
}

# Bandwidth limiting based on user tier and recent usage
within_bandwidth_limits if {
    current_bandwidth := calculate_bandwidth_usage(input.user_id, 60) # Last minute
    projected_bandwidth := current_bandwidth + (input.request_size / 60) # Bytes per second
    
    user_limits := get_rate_limits(input.user_id)
    max_bandwidth := user_limits.max_bandwidth_bps
    
    projected_bandwidth <= max_bandwidth
}

# Protocol-specific access controls
protocol_allowed if {
    user_permissions := get_user_permissions(input.user_id)
    allowed_protocols := user_permissions.allowed_protocols
    
    input.protocol in allowed_protocols
}

# Endpoint accessibility based on user permissions
endpoint_accessible if {
    user_permissions := get_user_permissions(input.user_id)
    
    # Check if endpoint is in allowed list
    endpoint_allowed
} else if {
    # Check if endpoint is not in blocked list
    not endpoint_blocked
}

endpoint_allowed if {
    user_permissions := get_user_permissions(input.user_id)
    allowed_endpoints := user_permissions.allowed_endpoints
    
    some endpoint in allowed_endpoints
    matches_endpoint_pattern(input.destination_endpoint, endpoint)
}

endpoint_blocked if {
    user_permissions := get_user_permissions(input.user_id)
    blocked_endpoints := user_permissions.blocked_endpoints
    
    some endpoint in blocked_endpoints
    matches_endpoint_pattern(input.destination_endpoint, endpoint)
}

# Geographic access controls
geographic_access_allowed if {
    source_country := get_country_from_ip(input.source_ip)
    destination_country := get_country_from_endpoint(input.destination_endpoint)
    
    user_permissions := get_user_permissions(input.user_id)
    allowed_countries := user_permissions.allowed_countries
    blocked_countries := user_permissions.blocked_countries
    
    # Allow if source and destination are in allowed countries
    (source_country in allowed_countries) or (count(allowed_countries) == 0)
    (destination_country in allowed_countries) or (count(allowed_countries) == 0)
    
    # Deny if either is in blocked countries
    not (source_country in blocked_countries)
    not (destination_country in blocked_countries)
}

# Suspicious pattern detection
suspicious_pattern if {
    rapid_fire_requests
} else if {
    unusual_request_size
} else if {
    suspicious_user_agent
} else if {
    bot_like_behavior
}

rapid_fire_requests if {
    recent_requests := count_recent_requests(input.user_id, "user", 10) # Last 10 seconds
    rapid_threshold := 50 # More than 50 requests in 10 seconds
    
    recent_requests > rapid_threshold
}

unusual_request_size if {
    user_profile := data.users[input.user_id]
    avg_request_size := user_profile.average_request_size
    
    # Flag if request is more than 10x typical size
    input.request_size > (avg_request_size * 10)
}

suspicious_user_agent if {
    known_bot_agents := data.policy_config.known_bot_user_agents
    input.user_agent in known_bot_agents
}

bot_like_behavior if {
    # Check for bot-like request patterns
    request_intervals := get_request_intervals(input.user_id, 300) # Last 5 minutes
    
    # Look for extremely regular intervals (bot characteristic)
    interval_variance := calculate_variance(request_intervals)
    interval_variance < 0.1 # Very low variance indicates bot behavior
}

# Blacklist checks
blacklisted_source if {
    blacklisted_users := data.security.blacklisted_users
    blacklisted_ips := data.security.blacklisted_ips
    
    (input.user_id in blacklisted_users) or (input.source_ip in blacklisted_ips)
}

# Helper functions
count_recent_requests(identifier, type, seconds_ago) := count if {
    cutoff_time := time.now_ns() - (seconds_ago * 1000000000)
    
    requests := data.request_logs[type][identifier]
    recent_requests := [req |
        req := requests[_]
        time.parse_rfc3339_ns(req.timestamp) > cutoff_time
    ]
    
    count := count(recent_requests)
}

count_recent_requests(identifier, type, seconds_ago) := 0 if {
    not data.request_logs[type][identifier]
}

calculate_bandwidth_usage(user_id, seconds_ago) := bandwidth if {
    cutoff_time := time.now_ns() - (seconds_ago * 1000000000)
    
    requests := data.request_logs.user[user_id]
    recent_requests := [req |
        req := requests[_]
        time.parse_rfc3339_ns(req.timestamp) > cutoff_time
    ]
    
    total_bytes := sum([req.request_size + req.response_size | req := recent_requests[_]])
    bandwidth := total_bytes / seconds_ago # Bytes per second
}

calculate_bandwidth_usage(user_id, seconds_ago) := 0 if {
    not data.request_logs.user[user_id]
}

get_rate_limits(user_id) := limits if {
    user_tier := data.users[user_id].tier
    limits := data.policy_config.rate_limits[user_tier]
}

get_rate_limits(user_id) := data.policy_config.rate_limits.default if {
    not data.users[user_id].tier
}

get_user_permissions(user_id) := permissions if {
    user_role := data.users[user_id].role
    permissions := data.policy_config.permissions[user_role]
}

get_user_permissions(user_id) := data.policy_config.permissions.default if {
    not data.users[user_id].role
}

matches_endpoint_pattern(endpoint, pattern) if {
    # Simple wildcard matching (in production, use proper regex)
    contains(pattern, "*")
    prefix := split(pattern, "*")[0]
    startswith(endpoint, prefix)
}

matches_endpoint_pattern(endpoint, pattern) if {
    not contains(pattern, "*")
    endpoint == pattern
}

get_country_from_ip(ip_address) := country if {
    # In production, use GeoIP database
    geo_data := data.geoip[ip_address]
    country := geo_data.country
}

get_country_from_ip(ip_address) := "UNKNOWN" if {
    not data.geoip[ip_address]
}

get_country_from_endpoint(endpoint) := country if {
    # Extract domain and look up country
    domain := extract_domain(endpoint)
    geo_data := data.domain_geo[domain]
    country := geo_data.country
}

get_country_from_endpoint(endpoint) := "UNKNOWN" if {
    domain := extract_domain(endpoint)
    not data.domain_geo[domain]
}

extract_domain(endpoint) := domain if {
    # Simple domain extraction (in production, use proper URL parsing)
    contains(endpoint, "://")
    url_parts := split(endpoint, "://")
    host_part := url_parts[1]
    domain_parts := split(host_part, "/")
    domain := domain_parts[0]
}

extract_domain(endpoint) := endpoint if {
    not contains(endpoint, "://")
}

get_request_intervals(user_id, seconds_ago) := intervals if {
    cutoff_time := time.now_ns() - (seconds_ago * 1000000000)
    
    requests := data.request_logs.user[user_id]
    recent_requests := [req |
        req := requests[_]
        time.parse_rfc3339_ns(req.timestamp) > cutoff_time
    ]
    
    # Calculate intervals between consecutive requests
    sorted_requests := sort(recent_requests, "timestamp")
    intervals := [interval |
        i := numbers.range(1, count(sorted_requests) - 1)[_]
        prev_time := time.parse_rfc3339_ns(sorted_requests[i-1].timestamp)
        curr_time := time.parse_rfc3339_ns(sorted_requests[i].timestamp)
        interval := (curr_time - prev_time) / 1000000000 # Convert to seconds
    ]
}

get_request_intervals(user_id, seconds_ago) := [] if {
    not data.request_logs.user[user_id]
}

calculate_variance(values) := variance if {
    count(values) > 1
    mean := sum(values) / count(values)
    squared_diffs := [(val - mean) * (val - mean) | val := values[_]]
    variance := sum(squared_diffs) / count(values)
}

calculate_variance(values) := 0 if {
    count(values) <= 1
}

# DDoS protection measures
ddos_protection_triggered if {
    global_request_rate := count_global_requests(60) # Requests per minute globally
    ddos_threshold := data.policy_config.ddos_threshold
    
    global_request_rate > ddos_threshold
}

count_global_requests(seconds_ago) := count if {
    cutoff_time := time.now_ns() - (seconds_ago * 1000000000)
    
    all_user_logs := data.request_logs.user
    all_requests := [req |
        user_logs := all_user_logs[_]
        req := user_logs[_]
        time.parse_rfc3339_ns(req.timestamp) > cutoff_time
    ]
    
    count := count(all_requests)
}

# Enhanced denial when DDoS protection is active
allow if {
    ddos_protection_triggered
    input.user_id in data.policy_config.priority_users
    valid_network_input
    within_emergency_rate_limits
}

within_emergency_rate_limits if {
    user_requests := count_recent_requests(input.user_id, "user", 60)
    emergency_limit := data.policy_config.emergency_rate_limit
    
    user_requests < emergency_limit
}

# Decision rationale for audit and debugging
rationale := reasons if {
    allow
    not ddos_protection_triggered
    reasons := [
        "network request approved",
        sprintf("user rate: %v/%v requests/min", [
            count_recent_requests(input.user_id, "user", 60),
            get_rate_limits(input.user_id).requests_per_minute
        ]),
        sprintf("bandwidth: %v/%v bytes/sec", [
            calculate_bandwidth_usage(input.user_id, 60),
            get_rate_limits(input.user_id).max_bandwidth_bps
        ]),
        sprintf("protocol: %v allowed", [input.protocol]),
        sprintf("endpoint: %v accessible", [input.destination_endpoint])
    ]
}

rationale := reasons if {
    allow
    ddos_protection_triggered
    reasons := [
        "network request approved under DDoS protection",
        "priority user access granted",
        sprintf("emergency rate: %v/%v requests/min", [
            count_recent_requests(input.user_id, "user", 60),
            data.policy_config.emergency_rate_limit
        ])
    ]
}

rationale := reasons if {
    not allow
    reasons := denial_reasons
}

denial_reasons := ["invalid input parameters"] if {
    not valid_network_input
}

denial_reasons := [reason] if {
    valid_network_input
    not within_request_rate_limits
    user_requests := count_recent_requests(input.user_id, "user", 60)
    rate_limit := get_rate_limits(input.user_id).requests_per_minute
    reason := sprintf("rate limit exceeded: %v >= %v requests/min", [user_requests, rate_limit])
}

denial_reasons := [reason] if {
    valid_network_input
    within_request_rate_limits
    not within_bandwidth_limits
    current_bw := calculate_bandwidth_usage(input.user_id, 60)
    max_bw := get_rate_limits(input.user_id).max_bandwidth_bps
    reason := sprintf("bandwidth limit exceeded: %v >= %v bytes/sec", [current_bw, max_bw])
}

denial_reasons := [reason] if {
    valid_network_input
    within_request_rate_limits
    within_bandwidth_limits
    not protocol_allowed
    reason := sprintf("protocol not allowed: %v", [input.protocol])
}

denial_reasons := [reason] if {
    valid_network_input
    within_request_rate_limits
    within_bandwidth_limits
    protocol_allowed
    not endpoint_accessible
    reason := sprintf("endpoint not accessible: %v", [input.destination_endpoint])
}

denial_reasons := [reason] if {
    valid_network_input
    within_request_rate_limits
    within_bandwidth_limits
    protocol_allowed
    endpoint_accessible
    not geographic_access_allowed
    source_country := get_country_from_ip(input.source_ip)
    dest_country := get_country_from_endpoint(input.destination_endpoint)
    reason := sprintf("geographic access denied: %v -> %v", [source_country, dest_country])
}

denial_reasons := [reason] if {
    suspicious_pattern
    reason := "suspicious request pattern detected"
}

denial_reasons := ["source is blacklisted"] if {
    blacklisted_source
}

denial_reasons := ["DDoS protection active - non-priority user"] if {
    ddos_protection_triggered
    not (input.user_id in data.policy_config.priority_users)
}

# Emergency bypass for critical operations
emergency_bypass if {
    input.emergency_token
    input.admin_user_id
    verify_emergency_token(input.emergency_token, input.admin_user_id)
    input.operation in ["emergency_communication", "incident_response"]
}

verify_emergency_token(token, admin_id) if {
    admin_data := data.admins[admin_id]
    admin_data.emergency_network_access == true
    startswith(token, "net_emergency_")
}

allow if {
    emergency_bypass
}

rationale := ["emergency network bypass applied"] if {
    emergency_bypass
}
