package polymera.policy

# Default deny - explicit allow required
default allow = false

# Default deny with rationale
default deny = {
    "allowed": false,
    "reason": "No explicit allow rule matched",
    "code": "DEFAULT_DENY"
}

# Example: File access policy
# Allows file access based on user permissions and file path
file_access = {
    "allowed": true,
    "reason": "File access granted",
    "permissions": ["read", "write"],
    "constraints": {
        "max_size": "100MB",
        "allowed_extensions": [".txt", ".md", ".rs", ".go", ".py", ".js"],
        "restricted_paths": ["/etc/", "/sys/", "/proc/"]
    }
} {
    # User has appropriate role
    input.user.role == "developer"
    
    # File path is not restricted
    not startswith(input.file.path, "/etc/")
    not startswith(input.file.path, "/sys/")
    not startswith(input.file.path, "/proc/")
    
    # File extension is allowed
    input.file.extension in [".txt", ".md", ".rs", ".go", ".py", ".js"]
    
    # File size is within limits
    input.file.size <= 104857600  # 100MB in bytes
}

# Example: Network access policy
# Controls network connections based on destination and protocol
network_access = {
    "allowed": true,
    "reason": "Network access granted",
    "constraints": {
        "allowed_protocols": ["HTTP", "HTTPS", "SSH"],
        "allowed_ports": [22, 80, 443, 8080, 8443],
        "blocked_domains": ["malware.example.com", "phishing.example.com"]
    }
} {
    # Protocol is allowed
    input.network.protocol in ["HTTP", "HTTPS", "SSH"]
    
    # Port is allowed
    input.network.port in [22, 80, 443, 8080, 8443]
    
    # Domain is not blocked
    not input.network.domain in ["malware.example.com", "phishing.example.com"]
    
    # For HTTPS, require valid certificate
    input.network.protocol == "HTTPS"
    input.network.certificate.valid == true
}

# Example: Process execution policy
# Controls which processes can be executed
process_execution = {
    "allowed": true,
    "reason": "Process execution granted",
    "constraints": {
        "allowed_binaries": ["/usr/bin/", "/usr/local/bin/"],
        "blocked_commands": ["rm -rf /", "format C:"],
        "max_memory": "512MB",
        "max_cpu": "100%"
    }
} {
    # Binary path is in allowed directories
    startswith(input.process.binary, "/usr/bin/") || startswith(input.process.binary, "/usr/local/bin/")
    
    # Command is not blocked
    not input.process.command in ["rm -rf /", "format C:"]
    
    # Resource limits are reasonable
    input.process.memory_limit <= 536870912  # 512MB in bytes
    input.process.cpu_limit <= 100
}

# Example: Container policy
# Controls container creation and execution
container_policy = {
    "allowed": true,
    "reason": "Container creation granted",
    "constraints": {
        "allowed_images": ["alpine:latest", "ubuntu:20.04", "debian:11"],
        "blocked_images": ["malware:latest", "suspicious:tag"],
        "resource_limits": {
            "max_memory": "2GB",
            "max_cpu": "4 cores",
            "max_disk": "10GB"
        },
        "security_requirements": {
            "readonly_root": true,
            "no_privileged": true,
            "no_host_network": true
        }
    }
} {
    # Image is allowed
    input.container.image in ["alpine:latest", "ubuntu:20.04", "debian:11"]
    
    # Image is not blocked
    not input.container.image in ["malware:latest", "suspicious:tag"]
    
    # Resource limits are within bounds
    input.container.resources.memory <= 2147483648  # 2GB in bytes
    input.container.resources.cpu <= 4
    input.container.resources.disk <= 10737418240  # 10GB in bytes
    
    # Security requirements are met
    input.container.security.readonly_root == true
    input.container.security.privileged == false
    input.container.security.host_network == false
}

# Example: API access policy
# Controls API endpoint access based on authentication and rate limiting
api_access = {
    "allowed": true,
    "reason": "API access granted",
    "constraints": {
        "rate_limit": "100 requests per minute",
        "required_headers": ["Authorization", "User-Agent"],
        "blocked_ips": ["192.168.1.100", "10.0.0.50"]
    }
} {
    # User is authenticated
    input.api.user.authenticated == true
    
    # Rate limit not exceeded
    input.api.rate_limit.current < 100
    
    # Required headers are present
    input.api.headers.Authorization != ""
    input.api.headers.User_Agent != ""
    
    # IP is not blocked
    not input.api.client_ip in ["192.168.1.100", "10.0.0.50"]
    
    # User has appropriate permissions for the endpoint
    input.api.user.permissions[input.api.endpoint] != ""
}

# Example: Data access policy
# Controls access to sensitive data based on classification and user clearance
data_access = {
    "allowed": true,
    "reason": "Data access granted",
    "constraints": {
        "data_classification": ["public", "internal", "confidential"],
        "user_clearance": ["public", "internal", "confidential", "secret"],
        "audit_required": true,
        "encryption_required": true
    }
} {
    # User clearance meets or exceeds data classification
    clearance_level(input.api.user.clearance) >= clearance_level(input.data.classification)
    
    # Audit logging is enabled
    input.data.audit.enabled == true
    
    # Encryption is required for sensitive data
    input.data.classification in ["confidential", "secret"]
    input.data.encryption.enabled == true
}

# Helper function to convert clearance levels to numeric values
clearance_level(clearance) = 1 { clearance == "public" }
clearance_level(clearance) = 2 { clearance == "internal" }
clearance_level(clearance) = 3 { clearance == "confidential" }
clearance_level(clearance) = 4 { clearance == "secret" }

# Example: System configuration policy
# Controls system configuration changes
system_config = {
    "allowed": true,
    "reason": "System configuration change granted",
    "constraints": {
        "allowed_settings": ["network", "storage", "security"],
        "blocked_settings": ["kernel", "firmware", "bios"],
        "maintenance_window": "02:00-04:00 UTC",
        "approval_required": true
    }
} {
    # Setting category is allowed
    input.config.category in ["network", "storage", "security"]
    
    # Setting category is not blocked
    not input.config.category in ["kernel", "firmware", "bios"]
    
    # Change is within maintenance window
    input.config.timestamp >= "02:00"
    input.config.timestamp <= "04:00"
    
    # Change has been approved
    input.config.approved == true
    input.config.approved_by != ""
}

# Example: Package installation policy
# Controls software package installation
package_install = {
    "allowed": true,
    "reason": "Package installation granted",
    "constraints": {
        "allowed_sources": ["official", "trusted"],
        "blocked_sources": ["unverified", "suspicious"],
        "max_package_size": "100MB",
        "security_scan_required": true
    }
} {
    # Package source is trusted
    input.package.source in ["official", "trusted"]
    
    # Package source is not blocked
    not input.package.source in ["unverified", "suspicious"]
    
    # Package size is within limits
    input.package.size <= 104857600  # 100MB in bytes
    
    # Security scan has been performed
    input.package.security_scan.completed == true
    input.package.security_scan.vulnerabilities == 0
}

# Example: User management policy
# Controls user account creation and modification
user_management = {
    "allowed": true,
    "reason": "User management operation granted",
    "constraints": {
        "allowed_operations": ["create", "modify", "disable"],
        "blocked_operations": ["delete", "privilege_escalation"],
        "password_requirements": {
            "min_length": 12,
            "complexity": "high",
            "expiration": "90 days"
        }
    }
} {
    # Operation is allowed
    input.user_operation.action in ["create", "modify", "disable"]
    
    # Operation is not blocked
    not input.user_operation.action in ["delete", "privilege_escalation"]
    
    # User has appropriate permissions
    input.user_operation.operator.role == "admin"
    
    # For user creation, password meets requirements
    input.user_operation.action == "create"
    input.user_operation.user.password != ""
    count(input.user_operation.user.password) >= 12
}

# Example: Backup and recovery policy
# Controls backup operations and data recovery
backup_recovery = {
    "allowed": true,
    "reason": "Backup/recovery operation granted",
    "constraints": {
        "allowed_operations": ["backup", "restore", "verify"],
        "encryption_required": true,
        "retention_period": "7 years",
        "offsite_storage": true
    }
} {
    # Operation is allowed
    input.backup_operation.action in ["backup", "restore", "verify"]
    
    # Data is encrypted
    input.backup_operation.encryption.enabled == true
    input.backup_operation.encryption.algorithm in ["AES-256", "ChaCha20"]
    
    # Retention policy is followed
    input.backup_operation.retention.days >= 2555  # 7 years
    
    # Offsite storage is configured
    input.backup_operation.storage.offsite == true
}

# Example: Compliance policy
# Ensures operations meet regulatory requirements
compliance_check = {
    "allowed": true,
    "reason": "Operation meets compliance requirements",
    "constraints": {
        "regulations": ["GDPR", "SOX", "HIPAA"],
        "data_retention": "7 years",
        "audit_trail": true,
        "encryption": "at rest and in transit"
    }
} {
    # GDPR compliance
    input.compliance.gdpr.data_processing_basis != ""
    input.compliance.gdpr.data_subject_rights == true
    
    # SOX compliance
    input.compliance.sox.financial_controls == true
    input.compliance.sox.change_management == true
    
    # HIPAA compliance
    input.compliance.hipaa.phi_protection == true
    input.compliance.hipaa.access_controls == true
    
    # General compliance requirements
    input.compliance.audit.enabled == true
    input.compliance.encryption.at_rest == true
    input.compliance.encryption.in_transit == true
}
