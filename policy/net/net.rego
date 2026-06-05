package aetheris.net

import rego.v1

# Default deny policy
default allow := false

# Allow local network access
allow if {
    input.operation == "connect"
    input.target_address =~ "127\\..*"
    input.process_cap[_] == "net:local"
}

# Allow WAN access with capability
allow if {
    input.operation == "connect"
    input.target_address !~ "127\\..*"
    input.target_address !~ "::1"
    input.process_cap[_] == "net:wan"
    input.target_port <= 1024
}

# Allow privileged port binding with capability
allow if {
    input.operation == "bind"
    input.target_port <= 1024
    input.process_cap[_] == "net:privileged"
}

# Allow unprivileged port binding
allow if {
    input.operation == "bind"
    input.target_port > 1024
    input.process_cap[_] == "net:socket"
}

# Allow listening with socket capability
allow if {
    input.operation == "listen"
    input.process_cap[_] == "net:socket"
}

# Allow sending data with send capability
allow if {
    input.operation == "send"
    input.process_cap[_] == "net:send"
    input.data_size <= 65536  # 64KB limit
}

# Allow receiving data with recv capability
allow if {
    input.operation == "recv"
    input.process_cap[_] == "net:recv"
    input.buffer_size <= 65536  # 64KB limit
}

# Allow DNS resolution with DNS capability
allow if {
    input.operation == "dns_resolve"
    input.process_cap[_] == "net:dns"
    not is_blocked_domain(input.domain)
}

# Allow TLS operations with TLS capability
allow if {
    input.operation in ["tls_handshake", "tls_send", "tls_recv"]
    input.process_cap[_] == "net:tls"
}

# Allow QUIC operations with QUIC capability
allow if {
    input.operation in ["quic_connect", "quic_stream", "quic_send", "quic_recv"]
    input.process_cap[_] == "net:quic"
}

# Allow libp2p operations with libp2p capability
allow if {
    input.operation in ["p2p_connect", "p2p_publish", "p2p_subscribe"]
    input.process_cap[_] == "net:libp2p"
}

# Check if domain is blocked
is_blocked_domain(domain) if {
    blocked_domains := ["malware.com", "phishing.net", "spam.org"]
    domain in blocked_domains
}

# Rate limiting check
rate_limit_exceeded if {
    input.operation == "connect"
    input.rate_limit_count > 100  # 100 connections per minute
}

# Bandwidth limiting check
bandwidth_exceeded if {
    input.operation == "send"
    input.bandwidth_usage > 100000000  # 100MB per hour
}

# Time-based restrictions
time_restricted if {
    input.operation == "connect"
    input.target_address !~ "127\\..*"
    input.time_hour < 8 or input.time_hour > 18  # Business hours only
}

# Deny if rate limit exceeded
deny if {
    rate_limit_exceeded
}

# Deny if bandwidth exceeded
deny if {
    bandwidth_exceeded
}

# Deny if time restricted
deny if {
    time_restricted
}

# Audit logging
audit_required if {
    input.operation in ["connect", "bind", "listen", "send", "recv"]
    input.target_address !~ "127\\..*"
}

# Security event detection
security_event if {
    input.operation == "connect"
    input.target_address in ["0.0.0.0", "::"]
}

# Policy decision with reason
policy_decision := {
    "allowed": allow,
    "reason": reason,
    "audit_required": audit_required,
    "security_event": security_event
}

# Generate reason for decision
reason := "Operation allowed" if {
    allow
}

reason := "Operation denied - insufficient capabilities" if {
    not allow
    not input.process_cap[_] in ["net:socket", "net:connect", "net:bind", "net:listen", "net:send", "net:recv", "net:dns", "net:tls", "net:quic", "net:libp2p", "net:wan", "net:privileged"]
}

reason := "Operation denied - rate limit exceeded" if {
    rate_limit_exceeded
}

reason := "Operation denied - bandwidth exceeded" if {
    bandwidth_exceeded
}

reason := "Operation denied - time restricted" if {
    time_restricted
}

reason := "Operation denied - blocked domain" if {
    input.operation == "dns_resolve"
    is_blocked_domain(input.domain)
}

reason := "Operation denied - security event detected" if {
    security_event
}
