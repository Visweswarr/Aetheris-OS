package firewall

import rego.v1

default allow := false

# Allow localhost connections
allow if {
    input.connection.source_ip == "127.0.0.1"
}

# Allow HTTPS traffic during business hours
allow if {
    input.connection.destination_port == 443
    input.connection.protocol == "TCP"
    is_business_hours(input.current_time)
}

# Allow internal network traffic
allow if {
    input.connection.source_ip in data.internal_networks
    input.connection.destination_ip in data.internal_networks
}

# Allow development traffic for dev processes
allow if {
    input.connection.process_capability == "net:dev"
    input.connection.source_ip in ["127.0.0.1", "::1"]
}

# Deny suspicious IPs
allow := false if {
    input.connection.source_ip in data.suspicious_ips
}

# Deny non-TLS traffic to sensitive ports
allow := false if {
    input.connection.destination_port in data.sensitive_ports
    input.connection.protocol != "TCP"
}

# Rate limiting for API endpoints
allow if {
    input.connection.destination_port in data.api_ports
    rate_limit_check(input.connection.source_ip)
}

# Time-based restrictions
allow := false if {
    input.connection.destination_port in data.restricted_ports
    not is_business_hours(input.current_time)
}

# Process capability checks
allow := false if {
    input.connection.destination_port < 1024
    not input.connection.process_capability in ["net:admin", "net:privileged"]
}

# Network namespace isolation
allow := false if {
    input.connection.network_namespace != "default"
    not input.connection.process_capability in ["net:multi-ns"]
}

# Helper functions
is_business_hours(time) := true if {
    hour := time.hour
    hour >= 9
    hour <= 17
    time.weekday >= 1
    time.weekday <= 5
}

rate_limit_check(ip) := true if {
    count(requests[ip]) < 100
}

# Data definitions (would be loaded from external source)
data := {
    "internal_networks": ["192.168.0.0/16", "10.0.0.0/8", "172.16.0.0/12"],
    "suspicious_ips": ["192.168.1.100", "10.0.0.100"],
    "sensitive_ports": [22, 23, 135, 139, 445, 1433, 3389],
    "api_ports": [8080, 8443, 9090],
    "restricted_ports": [22, 23, 135, 139, 445]
}
