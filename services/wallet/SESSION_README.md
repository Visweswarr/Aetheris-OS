# Session Key Types for Polymera OS

## Overview

The Session Key Types module provides purpose-bound, time-scoped, and amount-limited session keys with comprehensive validation and constraint checking. This module enables fine-grained access control for cryptographic operations based on session context, geographic location, network conditions, device characteristics, and usage patterns.

## Features

### 🎯 Purpose-Bound Sessions
- **Authentication Sessions**: User login, SSO, MFA operations
- **API Access Sessions**: Read, write, and administrative operations
- **Data Access Sessions**: Controlled access to different data types and classifications
- **Network Sessions**: VPN, proxy, and network access control
- **Device Sessions**: Device-specific access and control operations
- **Custom Sessions**: User-defined session types and purposes

### ⏰ Time-Based Constraints
- **Session Duration**: Start time, end time, and maximum duration limits
- **Time Windows**: Day-of-week and time-of-day restrictions
- **Timezone Requirements**: Geographic timezone enforcement
- **Expiration Handling**: Automatic session expiration and cleanup

### 📊 Amount-Based Constraints
- **Operation Limits**: Maximum number of cryptographic operations
- **Data Volume Limits**: Maximum data transfer and storage amounts
- **Financial Limits**: Maximum transaction amounts for financial operations
- **Rate Limiting**: Token bucket, leaky bucket, and fixed window algorithms
- **Usage Quotas**: Periodic quota resets and monitoring

### 🌍 Geographic Constraints
- **Country Restrictions**: ISO 3166-1 alpha-2 country code filtering
- **Regional Limits**: State/province level access control
- **City Restrictions**: Metropolitan area access limitations
- **Radius Constraints**: Circular geographic boundaries
- **Timezone Enforcement**: Geographic timezone validation

### 🌐 Network Constraints
- **IP Range Filtering**: CIDR notation IP address restrictions
- **Network Type Control**: WiFi, Ethernet, mobile, VPN requirements
- **Domain Restrictions**: Allowed domain name filtering
- **VPN Requirements**: Mandatory VPN usage and protocol specifications

### 📱 Device Constraints
- **Device Type Control**: Desktop, laptop, mobile, server restrictions
- **Manufacturer Limits**: Allowed device manufacturer filtering
- **Security Feature Requirements**: Biometric, TEE, secure boot requirements
- **Device Fingerprinting**: Hardware and software fingerprint validation

### 🔒 Security Constraints
- **Post-Quantum Security**: Dilithium, Kyber, and hybrid key support
- **Authentication Factors**: Multi-factor authentication requirements
- **Key Strength Requirements**: Minimum cryptographic key lengths
- **Security Policy Enforcement**: Organizational security policy compliance

## Architecture

### Core Components

```rust
pub struct SessionKey {
    pub id: String,                           // Unique session identifier
    pub keystore_key_id: String,              // Associated keystore key
    pub session_type: SessionType,            // Session purpose and type
    pub scope: SessionScope,                  // Access control scope
    pub time_constraints: TimeConstraints,    // Time-based limitations
    pub amount_constraints: AmountConstraints, // Usage-based limitations
    pub security_info: SecurityInfo,          // Security requirements
    pub metadata: SessionMetadata,            // Session metadata
    pub state: SessionState,                  // Current session state
}
```

### Validation Flow

1. **Session Validation**: Check time constraints and quota limits
2. **Scope Validation**: Verify geographic, network, and device constraints
3. **Operation Validation**: Ensure requested operation is within scope
4. **Security Validation**: Verify authentication and security requirements

## Usage Examples

### Creating a Session Key

```rust
use polymera_wallet::session::*;
use chrono::{Utc, Duration};

// Create geographic constraints
let geographic = GeographicConstraints {
    allowed_countries: vec!["US".to_string()],
    allowed_regions: vec!["CA".to_string()],
    allowed_cities: vec![],
    radius_constraint: None,
    allowed_timezones: vec!["America/Los_Angeles".to_string()],
};

// Create network constraints
let network = NetworkConstraints {
    allowed_ip_ranges: vec!["192.168.1.0/24".to_string()],
    allowed_network_types: vec![NetworkType::Wifi, NetworkType::Vpn],
    allowed_domains: vec!["company.com".to_string()],
    vpn_requirement: Some(VpnRequirement {
        vpn_provider: "company-vpn".to_string(),
        vpn_location: "US-West".to_string(),
        vpn_protocol: VpnProtocol::WireGuard,
    }),
};

// Create device constraints
let device = DeviceConstraints {
    allowed_device_types: vec![DeviceType::Desktop, DeviceType::Laptop],
    allowed_manufacturers: vec!["Dell".to_string(), "HP".to_string()],
    required_security_features: vec![DeviceSecurityFeature::SecureBoot],
};

// Create data constraints
let data = DataConstraints {
    allowed_data_types: vec![DataType::Technical, DataType::Operational],
    allowed_classifications: vec![DataClassification::Internal, DataClassification::Confidential],
    allowed_data_sources: vec!["internal-api".to_string()],
};

// Create session scope
let scope = SessionScope {
    resources: vec!["user-data".to_string(), "analytics".to_string()],
    actions: vec!["read".to_string(), "query".to_string()],
    geographic,
    network,
    device,
    data,
    custom_attributes: HashMap::new(),
};

// Create time constraints
let time_constraints = TimeConstraints {
    start_time: Utc::now(),
    end_time: Some(Utc::now() + Duration::hours(8)),
    max_duration: Some(Duration::hours(8)),
    allowed_windows: vec![
        TimeWindow {
            day_of_week: 1, // Monday
            start_time: "09:00".to_string(),
            end_time: "17:00".to_string(),
            timezone: "America/Los_Angeles".to_string(),
        }
    ],
    required_timezone: Some("America/Los_Angeles".to_string()),
};

// Create amount constraints
let amount_constraints = AmountConstraints {
    max_operations: Some(1000),
    max_data_volume: Some(1024 * 1024 * 100), // 100MB
    max_financial_amount: Some(10000), // $100.00
    max_sessions: Some(1),
    rate_limit: Some(RateLimit {
        requests_per_window: 100,
        window_duration: Duration::minutes(1),
        burst_allowance: 20,
        algorithm: RateLimitAlgorithm::TokenBucket,
    }),
    usage_quotas: vec![
        UsageQuota {
            quota_type: QuotaType::Operations,
            limit: 1000,
            period: Duration::hours(8),
        }
    ],
};

// Create security info
let security_info = SecurityInfo {
    algorithm: "Ed25519".to_string(),
    key_strength: 256,
    pq_security_level: PostQuantumSecurityLevel::Level1,
    required_factors: vec![
        AuthenticationFactor::Password,
        AuthenticationFactor::Biometric,
    ],
    security_policies: vec!["corporate-policy".to_string()],
};

// Create session key
let session_key = SessionKey::new(
    "keystore-key-123".to_string(),
    SessionType::DataAccess,
    scope,
    time_constraints,
    amount_constraints,
    security_info,
    "Daily Work Session".to_string(),
    "user@company.com".to_string(),
);
```

### Validating Session Access

```rust
// Create validation context
let context = ValidationContext {
    current_time: Utc::now(),
    current_location: GeographicLocation {
        latitude: 37.7749,
        longitude: -122.4194,
        country_code: "US".to_string(),
        region: "CA".to_string(),
        city: "San Francisco".to_string(),
        timezone: "America/Los_Angeles".to_string(),
    },
    current_network: NetworkInfo {
        ip_address: "192.168.1.100".to_string(),
        network_type: NetworkType::Vpn,
        is_vpn: true,
        domain: "company.com".to_string(),
    },
    current_device: DeviceInfo {
        device_type: DeviceType::Laptop,
        manufacturer: "Dell".to_string(),
        model: "XPS 13".to_string(),
        operating_system: "Ubuntu 22.04".to_string(),
        security_features: vec![DeviceSecurityFeature::SecureBoot],
    },
};

// Validate session access
match session_key.is_operation_allowed("read", "user-data", &context) {
    Ok(_) => {
        println!("Access granted to user-data");
        // Perform the operation
    }
    Err(SessionKeyError::InvalidScope(msg)) => {
        println!("Access denied: {}", msg);
    }
    Err(SessionKeyError::SessionExpired) => {
        println!("Session has expired");
    }
    Err(SessionKeyError::QuotaExceeded) => {
        println!("Usage quota exceeded");
    }
    Err(e) => {
        println!("Validation error: {}", e);
    }
}
```

### Using the Session Validator

```rust
// Validate session scope
let validator = SessionKeyValidator;
match validator.validate_scope(&session_key, "read", "user-data", &context) {
    Ok(_) => println!("Scope validation passed"),
    Err(e) => println!("Scope validation failed: {}", e),
}
```

## Protocol Buffer Integration

The session key types are defined in Protocol Buffers for cross-language compatibility and gRPC service integration:

```protobuf
message SessionKey {
    string id = 1;
    string keystore_key_id = 2;
    SessionType session_type = 3;
    SessionScope scope = 4;
    TimeConstraints time_constraints = 5;
    AmountConstraints amount_constraints = 6;
    SecurityInfo security_info = 7;
    SessionMetadata metadata = 8;
    SessionState state = 9;
}
```

## Testing

The module includes comprehensive tests to ensure proper validation and constraint enforcement:

```bash
# Run all session tests
cargo test session

# Run specific test
cargo test test_invalid_scope_rejection
```

### Test Coverage

- ✅ **Session Type Operations**: Validates allowed operations for each session type
- ✅ **Geographic Constraints**: Tests location-based access control
- ✅ **Invalid Scope Rejection**: Ensures unauthorized access is properly rejected
- ✅ **Invalid Action Rejection**: Tests action-based access control
- ✅ **Invalid Resource Rejection**: Tests resource-based access control
- ✅ **Time Constraints**: Validates session timing and expiration
- ✅ **Amount Constraints**: Tests quota and rate limiting

## Security Considerations

### Constraint Enforcement
- All constraints are enforced at runtime during session validation
- Geographic constraints use precise coordinate and timezone validation
- Network constraints support CIDR notation and VPN requirements
- Device constraints enforce security feature requirements

### Session Isolation
- Each session key has isolated scope and constraints
- No cross-session privilege escalation
- Secure session state management

### Audit and Monitoring
- Comprehensive session validation logging
- Constraint violation tracking
- Usage pattern analysis

## Performance

### Optimization Features
- Efficient constraint checking algorithms
- Minimal memory overhead for constraint structures
- Fast validation path for common operations

### Scalability
- Support for large numbers of concurrent sessions
- Efficient constraint caching and validation
- Horizontal scaling through stateless validation

## Integration

### Keystore Integration
- Seamless integration with Polymera OS keystore
- Session keys bound to specific keystore keys
- Unified cryptographic operation validation

### Service Integration
- gRPC service definitions for remote validation
- REST API support through serde serialization
- Cross-language client library support

## Future Enhancements

### Planned Features
- **Advanced Geographic Constraints**: Polygon-based boundaries, multi-location support
- **Dynamic Constraint Updates**: Runtime constraint modification
- **Machine Learning Integration**: Behavioral pattern analysis for access control
- **Blockchain Integration**: Decentralized session validation
- **Quantum-Safe Extensions**: Enhanced post-quantum cryptography support

### Extension Points
- **Custom Constraint Types**: User-defined constraint implementations
- **Plugin Architecture**: Third-party constraint validation plugins
- **Policy Engine Integration**: Advanced policy-based access control

## Contributing

### Development Guidelines
- Follow Rust coding standards and best practices
- Include comprehensive tests for new features
- Update documentation for API changes
- Ensure backward compatibility

### Testing Requirements
- Unit tests for all new functionality
- Integration tests for constraint interactions
- Performance benchmarks for validation operations
- Security tests for constraint bypass attempts

## License

This module is part of Polymera OS and is licensed under the same terms as the main project.

## Support

For questions, issues, or contributions:
- Create an issue in the Polymera OS repository
- Review the main project documentation
- Check the test suite for usage examples
- Consult the API documentation for detailed specifications
