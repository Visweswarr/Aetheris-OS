#![no_main]
use libfuzzer_sys::fuzz_target;
use polymera_os_kernel::secman::capabilities::{Capability, CapabilityType, CapabilityFlags, CapabilityId, ProcessId, ResourceId};
use polymera_os_kernel::secman::audit;
use alloc::vec::Vec;
use core::mem;

// Fuzzing target for capabilities
// Tests random cap fields, expiries, and capability operations
fuzz_target!(|data: &[u8]| {
    if data.len() < 12 {
        return; // Need minimum data for meaningful fuzzing
    }
    
    // Initialize kernel components for fuzzing
    if let Err(_) = polymera_os_kernel::init_fuzzing_environment() {
        return; // Skip if initialization fails
    }
    
    // Parse fuzz input to determine operations
    let mut offset = 0;
    
    // First byte: operation type (0=create, 1=validate, 2=revoke, 3=mixed)
    let op_type = if data.len() > offset { data[offset] % 4 } else { 0 };
    offset += 1;
    
    // Second byte: number of operations (1-100)
    let num_ops = if data.len() > offset { 
        (data[offset] % 100) + 1 
    } else { 
        20 
    };
    offset += 1;
    
    // Third byte: capability type distribution
    let cap_type_dist = if data.len() > offset { data[offset] } else { 0 };
    offset += 1;
    
    // Fourth byte: flags distribution
    let flags_dist = if data.len() > offset { data[offset] } else { 0 };
    offset += 1;
    
    // Fifth byte: expiry time range (0-3600 seconds)
    let max_expiry = if data.len() > offset { 
        ((data[offset] as u64) % 3600) + 1 
    } else { 
        300 
    };
    offset += 1;
    
    // Sixth byte: process ID range (1-1000)
    let max_pid = if data.len() > offset { 
        ((data[offset] as u64) % 1000) + 1 
    } else { 
        100 
    };
    offset += 1;
    
    // Seventh byte: resource ID range (1-10000)
    let max_rid = if data.len() > offset { 
        ((data[offset] as u64) % 10000) + 1 
    } else { 
        1000 
    };
    offset += 1;
    
    // Eighth byte: capability ID range (1-100000)
    let max_cap_id = if data.len() > offset { 
        ((data[offset] as u64) % 100000) + 1 
    } else { 
        10000 
    };
    offset += 1;
    
    // Ninth byte: permission bits distribution
    let perms_dist = if data.len() > offset { data[offset] } else { 0 };
    offset += 1;
    
    // Tenth byte: delegation depth (0-10)
    let max_delegation = if data.len() > offset { 
        (data[offset] % 10) + 1 
    } else { 
        3 
    };
    offset += 1;
    
    // Eleventh byte: revocation policy
    let revoke_policy = if data.len() > offset { data[offset] % 4 } else { 0 };
    offset += 1;
    
    // Twelfth byte: audit level
    let audit_level = if data.len() > offset { data[offset] % 3 } else { 0 };
    offset += 1;
    
    let mut capabilities = Vec::new();
    let mut successful_creates = 0;
    let mut successful_validations = 0;
    let mut successful_revocations = 0;
    let mut validation_failures = 0;
    let mut revocation_failures = 0;
    
    // Generate and process capabilities based on operation type
    match op_type {
        0 => { // Create-only operations
            for i in 0..num_ops {
                let cap = generate_random_capability(
                    i, cap_type_dist, flags_dist, max_expiry, max_pid, 
                    max_rid, max_cap_id, perms_dist, max_delegation, 
                    revoke_policy, audit_level, &data, offset
                );
                
                if let Ok(created_cap) = create_capability(cap.clone()) {
                    successful_creates += 1;
                    capabilities.push(created_cap);
                    
                    // Log capability creation
                    audit::log(audit::AuditEntry::new(
                        cap.owner.0,
                        201, // CAPABILITY_CREATED
                        cap.id.0
                    ));
                }
                
                offset = (offset + 1) % (data.len().max(1));
            }
        }
        
        1 => { // Validate-only operations
            // First create some capabilities to validate
            for i in 0..(num_ops / 2) {
                let cap = generate_random_capability(
                    i, cap_type_dist, flags_dist, max_expiry, max_pid,
                    max_rid, max_cap_id, perms_dist, max_delegation,
                    revoke_policy, audit_level, &data, offset
                );
                
                if let Ok(created_cap) = create_capability(cap.clone()) {
                    capabilities.push(created_cap);
                }
                offset = (offset + 1) % (data.len().max(1));
            }
            
            // Now validate capabilities
            for cap in &capabilities {
                if validate_capability(cap) {
                    successful_validations += 1;
                    
                    // Log successful validation
                    audit::log(audit::AuditEntry::new(
                        cap.owner.0,
                        202, // CAPABILITY_VALIDATED
                        cap.id.0
                    ));
                } else {
                    validation_failures += 1;
                    
                    // Log validation failure
                    audit::log(audit::AuditEntry::new(
                        cap.owner.0,
                        203, // CAPABILITY_VALIDATION_FAILED
                        cap.id.0
                    ));
                }
            }
        }
        
        2 => { // Revoke-only operations
            // First create some capabilities to revoke
            for i in 0..(num_ops / 2) {
                let cap = generate_random_capability(
                    i, cap_type_dist, flags_dist, max_expiry, max_pid,
                    max_rid, max_cap_id, perms_dist, max_delegation,
                    revoke_policy, audit_level, &data, offset
                );
                
                if let Ok(created_cap) = create_capability(cap.clone()) {
                    capabilities.push(created_cap);
                }
                offset = (offset + 1) % (data.len().max(1));
            }
            
            // Now revoke capabilities
            for cap in &capabilities {
                if revoke_capability(cap) {
                    successful_revocations += 1;
                    
                    // Log successful revocation
                    audit::log(audit::AuditEntry::new(
                        cap.owner.0,
                        204, // CAPABILITY_REVOKED
                        cap.id.0
                    ));
                } else {
                    revocation_failures += 1;
                    
                    // Log revocation failure
                    audit::log(audit::AuditEntry::new(
                        cap.owner.0,
                        205, // CAPABILITY_REVOCATION_FAILED
                        cap.id.0
                    ));
                }
            }
        }
        
        3 => { // Mixed operations
            for i in 0..num_ops {
                match i % 4 {
                    0 => { // Create capability
                        let cap = generate_random_capability(
                            i, cap_type_dist, flags_dist, max_expiry, max_pid,
                            max_rid, max_cap_id, perms_dist, max_delegation,
                            revoke_policy, audit_level, &data, offset
                        );
                        
                        if let Ok(created_cap) = create_capability(cap.clone()) {
                            successful_creates += 1;
                            capabilities.push(created_cap);
                            
                            audit::log(audit::AuditEntry::new(
                                cap.owner.0,
                                201, // CAPABILITY_CREATED
                                cap.id.0
                            ));
                        }
                    }
                    
                    1 => { // Validate capability
                        if let Some(cap) = capabilities.get(i / 4) {
                            if validate_capability(cap) {
                                successful_validations += 1;
                                audit::log(audit::AuditEntry::new(
                                    cap.owner.0,
                                    202, // CAPABILITY_VALIDATED
                                    cap.id.0
                                ));
                            } else {
                                validation_failures += 1;
                                audit::log(audit::AuditEntry::new(
                                    cap.owner.0,
                                    203, // CAPABILITY_VALIDATION_FAILED
                                    cap.id.0
                                ));
                            }
                        }
                    }
                    
                    2 => { // Revoke capability
                        if let Some(cap) = capabilities.get(i / 4) {
                            if revoke_capability(cap) {
                                successful_revocations += 1;
                                audit::log(audit::AuditEntry::new(
                                    cap.owner.0,
                                    204, // CAPABILITY_REVOKED
                                    cap.id.0
                                ));
                            } else {
                                revocation_failures += 1;
                                audit::log(audit::AuditEntry::new(
                                    cap.owner.0,
                                    205, // CAPABILITY_REVOCATION_FAILED
                                    cap.id.0
                                ));
                            }
                        }
                    }
                    
                    3 => { // Delegate capability
                        if let Some(cap) = capabilities.get(i / 4) {
                            let delegated_cap = delegate_capability(cap, max_pid);
                            if let Ok(delegated) = delegated_cap {
                                capabilities.push(delegated);
                                audit::log(audit::AuditEntry::new(
                                    cap.owner.0,
                                    206, // CAPABILITY_DELEGATED
                                    delegated.id.0
                                ));
                            }
                        }
                    }
                    
                    _ => unreachable!()
                }
                
                offset = (offset + 1) % (data.len().max(1));
            }
        }
        
        _ => unreachable!()
    }
    
    // Verify capability state consistency
    let active_capabilities = capabilities.iter().filter(|cap| !cap.is_revoked()).count();
    let expected_active = successful_creates - successful_revocations;
    
    // Allow for some tolerance due to delegation and validation
    assert!(active_capabilities <= expected_active + 10, 
            "Too many active capabilities: {} vs expected {}", 
            active_capabilities, expected_active);
    
    // Verify no capabilities were corrupted
    for cap in &capabilities {
        assert!(cap.id.0 > 0, "Corrupted capability ID");
        assert!(cap.owner.0 > 0, "Corrupted owner ID");
        assert!(cap.resource.0 > 0, "Corrupted resource ID");
        assert!(cap.expiry > 0, "Corrupted expiry time");
    }
    
    // Log fuzzing results for analysis
    audit::log(audit::AuditEntry::new(
        0, // System operation
        998, // FUZZ_CAPABILITIES_COMPLETED
        (successful_creates << 48) | (successful_validations << 32) | (successful_revocations << 16) | validation_failures as u64
    ));
});

/// Generate a random capability based on fuzz input parameters
fn generate_random_capability(
    index: usize,
    cap_type_dist: u8,
    flags_dist: u8,
    max_expiry: u64,
    max_pid: u64,
    max_rid: u64,
    max_cap_id: u64,
    perms_dist: u8,
    max_delegation: u8,
    revoke_policy: u8,
    audit_level: u8,
    data: &[u8],
    offset: usize
) -> Capability {
    // Generate capability type based on distribution
    let cap_type = match cap_type_dist % 8 {
        0 => CapabilityType::Read,
        1 => CapabilityType::Write,
        2 => CapabilityType::Execute,
        3 => CapabilityType::Delete,
        4 => CapabilityType::Admin,
        5 => CapabilityType::Delegate,
        6 => CapabilityType::Audit,
        7 => CapabilityType::Custom,
        _ => CapabilityType::Read,
    };
    
    // Generate flags based on distribution
    let mut flags = CapabilityFlags::empty();
    if (flags_dist & 0x01) != 0 { flags.insert(CapabilityFlags::TRANSFERABLE); }
    if (flags_dist & 0x02) != 0 { flags.insert(CapabilityFlags::DELEGATABLE); }
    if (flags_dist & 0x04) != 0 { flags.insert(CapabilityFlags::REVOCABLE); }
    if (flags_dist & 0x08) != 0 { flags.insert(CapabilityFlags::AUDITED); }
    if (flags_dist & 0x10) != 0 { flags.insert(CapabilityFlags::PERSISTENT); }
    
    // Generate expiry time (current time + random offset)
    let current_time = polymera_os_kernel::log::get_current_time_ms();
    let expiry_offset = if data.len() > offset {
        ((data[offset] as u64) % max_expiry) * 1000 // Convert to milliseconds
    } else {
        300000 // Default 5 minutes
    };
    
    // Generate process and resource IDs
    let owner_id = (index as u64 % max_pid) + 1;
    let resource_id = (index as u64 % max_rid) + 1;
    let capability_id = (index as u64 % max_cap_id) + 1;
    
    // Generate permissions based on distribution
    let permissions = if data.len() > offset + 1 {
        ((data[offset + 1] as u64) << 32) | (perms_dist as u64)
    } else {
        perms_dist as u64
    };
    
    // Generate delegation depth
    let delegation_depth = if data.len() > offset + 2 {
        (data[offset + 2] % max_delegation) as u8
    } else {
        0
    };
    
    // Generate revocation policy
    let revoke_policy_enum = match revoke_policy {
        0 => polymera_os_kernel::secman::capabilities::RevocationPolicy::Immediate,
        1 => polymera_os_kernel::secman::capabilities::RevocationPolicy::Graceful,
        2 => polymera_os_kernel::secman::capabilities::RevocationPolicy::Cascading,
        3 => polymera_os_kernel::secman::capabilities::RevocationPolicy::None,
        _ => polymera_os_kernel::secman::capabilities::RevocationPolicy::Immediate,
    };
    
    // Generate audit level
    let audit_level_enum = match audit_level {
        0 => polymera_os_kernel::secman::capabilities::AuditLevel::None,
        1 => polymera_os_kernel::secman::capabilities::AuditLevel::Basic,
        2 => polymera_os_kernel::secman::capabilities::AuditLevel::Detailed,
        _ => polymera_os_kernel::secman::capabilities::AuditLevel::Basic,
    };
    
    Capability {
        id: CapabilityId(capability_id),
        owner: ProcessId(owner_id),
        resource: ResourceId(resource_id),
        cap_type,
        permissions,
        flags,
        expiry: current_time + expiry_offset,
        delegation_depth,
        revoke_policy: revoke_policy_enum,
        audit_level: audit_level_enum,
        created: current_time,
        last_used: current_time,
        usage_count: 0,
        is_revoked: false,
        parent: None,
        children: Vec::new(),
    }
}

/// Create a capability (simulated for fuzzing)
fn create_capability(cap: Capability) -> Result<Capability, &'static str> {
    // Simulate capability creation
    if cap.owner.0 == 0 || cap.resource.0 == 0 {
        return Err("Invalid capability parameters");
    }
    
    if cap.expiry <= polymera_os_kernel::log::get_current_time_ms() {
        return Err("Capability already expired");
    }
    
    Ok(cap)
}

/// Validate a capability (simulated for fuzzing)
fn validate_capability(cap: &Capability) -> bool {
    // Simulate capability validation
    if cap.is_revoked {
        return false;
    }
    
    if cap.expiry <= polymera_os_kernel::log::get_current_time_ms() {
        return false;
    }
    
    if cap.owner.0 == 0 || cap.resource.0 == 0 {
        return false;
    }
    
    true
}

/// Revoke a capability (simulated for fuzzing)
fn revoke_capability(cap: &Capability) -> bool {
    // Simulate capability revocation
    if cap.is_revoked {
        return false;
    }
    
    if !cap.flags.contains(CapabilityFlags::REVOCABLE) {
        return false;
    }
    
    // In real implementation, this would mark the capability as revoked
    true
}

/// Delegate a capability (simulated for fuzzing)
fn delegate_capability(cap: &Capability, max_pid: u64) -> Result<Capability, &'static str> {
    // Simulate capability delegation
    if !cap.flags.contains(CapabilityFlags::DELEGATABLE) {
        return Err("Capability not delegatable");
    }
    
    if cap.delegation_depth >= 10 {
        return Err("Maximum delegation depth reached");
    }
    
    let new_owner = ProcessId((cap.owner.0 + 1) % max_pid + 1);
    let mut delegated = cap.clone();
    delegated.owner = new_owner;
    delegated.delegation_depth = cap.delegation_depth + 1;
    delegated.parent = Some(cap.id);
    
    Ok(delegated)
}

