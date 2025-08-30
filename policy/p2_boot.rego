package polymera.policy

import future.keywords.if
import future.keywords.in

# Default policy mode - can be overridden by environment
default policy_mode = "fail_closed"

# Policy mode determination based on environment
policy_mode = "fail_open" if {
    # Development environment
    input.environment == "development"
}

policy_mode = "fail_open" if {
    # Debug build
    input.debug_build == true
}

policy_mode = "fail_closed" if {
    # Production environment
    input.environment == "production"
}

# IPC authentication policy evaluation
allow_ipc_send = true if {
    policy_mode == "fail_closed"
    input.has_capability == true
    input.has_valid_mac == true
}

allow_ipc_send = false if {
    policy_mode == "fail_closed"
    input.has_capability == false
}

allow_ipc_send = false if {
    policy_mode == "fail_closed"
    input.has_valid_mac == false
}

# Fail-open mode allows operations with minimal authentication
allow_ipc_send = true if {
    policy_mode == "fail_open"
    input.has_capability == true
    input.has_valid_mac == true
}

allow_ipc_send = true if {
    policy_mode == "fail_open"
    input.has_capability == true
    input.has_valid_mac == false
}

allow_ipc_send = true if {
    policy_mode == "fail_open"
    input.has_capability == false
    input.has_valid_mac == true
}

allow_ipc_send = true if {
    policy_mode == "fail_open"
    input.has_capability == false
    input.has_valid_mac == false
}

# Audit requirements
require_audit = true if {
    policy_mode == "fail_open"
    input.has_capability == false
    input.has_valid_mac == false
}

require_audit = true if {
    policy_mode == "fail_open"
    input.has_capability == true
    input.has_valid_mac == false
}

require_audit = true if {
    policy_mode == "fail_open"
    input.has_capability == false
    input.has_valid_mac == true
}

require_audit = false if {
    policy_mode == "fail_open"
    input.has_capability == true
    input.has_valid_mac == true
}

require_audit = false if {
    policy_mode == "fail_closed"
    input.has_capability == true
    input.has_valid_mac == true
}

# Decision reason
decision_reason = "Full authentication (capability + MAC)" if {
    policy_mode == "fail_closed"
    input.has_capability == true
    input.has_valid_mac == true
}

decision_reason = "Missing capability token" if {
    policy_mode == "fail_closed"
    input.has_capability == false
}

decision_reason = "Missing message authentication code" if {
    policy_mode == "fail_closed"
    input.has_capability == true
    input.has_valid_mac == false
}

decision_reason = "No capability or MAC provided" if {
    policy_mode == "fail_closed"
    input.has_capability == false
    input.has_valid_mac == false
}

decision_reason = "Full authentication (capability + MAC)" if {
    policy_mode == "fail_open"
    input.has_capability == true
    input.has_valid_mac == true
}

decision_reason = "Capability-only authentication (dev mode)" if {
    policy_mode == "fail_open"
    input.has_capability == true
    input.has_valid_mac == false
}

decision_reason = "MAC-only authentication (dev mode)" if {
    policy_mode == "fail_open"
    input.has_capability == false
    input.has_valid_mac == true
}

decision_reason = "No authentication (dev mode - NOT FOR PRODUCTION)" if {
    policy_mode == "fail_open"
    input.has_capability == false
    input.has_valid_mac == false
}

# Security risk assessment
security_risk = "low" if {
    policy_mode == "fail_closed"
    input.has_capability == true
    input.has_valid_mac == true
}

security_risk = "medium" if {
    policy_mode == "fail_open"
    input.has_capability == true
    input.has_valid_mac == false
}

security_risk = "medium" if {
    policy_mode == "fail_open"
    input.has_capability == false
    input.has_valid_mac == true
}

security_risk = "high" if {
    policy_mode == "fail_open"
    input.has_capability == false
    input.has_valid_mac == false
}

# Policy metadata
policy_metadata = {
    "version": "1.0.0",
    "mode": policy_mode,
    "security_level": security_risk,
    "audit_required": require_audit,
    "capability_required": input.has_capability,
    "mac_required": input.has_valid_mac
}

# Main policy decision
main = {
    "allow": allow_ipc_send,
    "reason": decision_reason,
    "audit": require_audit,
    "metadata": policy_metadata
}

