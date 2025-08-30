# Phase 2 Design: PQC + Syscall/ABI Hardening

## 🏗️ **System Architecture Overview**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PHASE 2 ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   User      │  │   User      │  │   User      │  │   User      │      │
│  │   App A     │  │   App B     │  │   App C     │  │   App D     │      │
│  │             │  │             │  │             │  │             │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
├─────────────────────────────────────────────────────────────────────────────┤
│                        CAPABILITY-BASED SECURITY LAYER                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   Cap       │  │   Cap       │  │   Cap       │  │   Cap       │      │
│  │   Token A   │  │   Token B   │  │   Token C   │  │   Token D   │      │
│  │   (PQC-Sig) │  │   (PQC-Sig) │  │   (PQC-Sig) │  │   (PQC-Sig) │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
├─────────────────────────────────────────────────────────────────────────────┤
│                           SYSTEM CALL INTERFACE                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   Syscall   │  │   Cap      │  │   ABI      │  │   Version   │      │
│  │   Table     │  │   Check    │  │   Router   │  │   Manager   │      │
│  │             │  │             │  │             │  │             │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
├─────────────────────────────────────────────────────────────────────────────┤
│                              KERNEL CORE                                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   SecMan    │  │   PolyBus   │  │   APIC     │  │   PF        │      │
│  │   (PQC)     │  │   (Auth)    │  │   Timer    │  │   Handler   │      │
│  │             │  │             │  │             │  │   V2        │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 🔐 **Capability Token Format & PQC Integration**

### **Capability Token Structure**
```
┌─────────────┬─────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
│   Header    │   Version   │   Type      │   Rights    │   Expiry    │   Metadata  │
│   (32 bits) │   (8 bits)  │   (8 bits)  │  (16 bits)  │  (32 bits)  │  (64 bits)  │
├─────────────┼─────────────┼─────────────┼─────────────┼─────────────┼─────────────┤
│   Target    │   Source    │   Context   │   Nonce     │   Reserved  │   Padding   │
│   (64 bits) │   (64 bits) │  (128 bits) │  (64 bits)  │  (32 bits)  │  (32 bits)  │
├─────────────┴─────────────┴─────────────┴─────────────┴─────────────┴─────────────┤
│                              Dilithium Signature (256 bits)                      │
└─────────────────────────────────────────────────────────────────────────────────┘
```

**Token Components:**
- **Header**: Magic number and format version
- **Version**: Capability protocol version
- **Type**: Access rights classification
- **Rights**: Bitmap of specific permissions
- **Expiry**: Unix timestamp for token validity
- **Metadata**: Application-specific data
- **Target**: Resource identifier
- **Source**: Token issuer identifier
- **Context**: Execution context hash
- **Nonce**: Cryptographic nonce for uniqueness
- **Signature**: Dilithium signature over all fields

### **PQC-Signed Capability Flow**
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   SecMan    │    │   Cap      │    │   PolyBus   │    │   Target    │
│   Core      │───►│   Store    │───►│   Router    │───►│   Resource  │
│             │    │             │    │             │    │             │
│ 1. Generate │    │ 2. Store   │    │ 3. Validate │    │ 4. Execute  │
│    Kyber    │    │    Token   │    │    Token    │    │    Access   │
│    Keypair  │    │ 3. Sign    │    │ 4. Check    │    │             │
│ 2. Create   │    │    with    │    │    Rights   │    │             │
│    Token    │    │  Dilithium │    │ 5. Route   │    │             │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

## 🚌 **PolyBus Authentication Flow**

### **IPC Authentication Architecture**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Sender App    │    │   PolyBus       │    │   Receiver App  │
│                 │    │   Router        │    │                 │
│ 1. Create      │    │                 │    │                 │
│    Message     │    │ 2. Validate     │    │                 │
│ 2. Attach      │    │    Cap Token    │    │                 │
│    Cap Token   │───►│ 3. Check        │───►│ 4. Receive      │
│ 3. Sign with   │    │    Rights       │    │    Message      │
│    Dilithium   │    │ 4. Route        │    │ 5. Validate     │
│                 │    │    Message      │    │    Sender       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### **Message Authentication Header**
```
┌─────────────┬─────────────┬─────────────┬─────────────┬─────────────┐
│   Version   │   Type      │   Length    │   Cap      │   Nonce     │
│   (8 bits)  │   (8 bits)  │  (16 bits)  │  Token ID  │  (64 bits)  │
├─────────────┼─────────────┼─────────────┼─────────────┼─────────────┤
│   Timestamp │   Priority  │   Reserved  │   Padding   │   Checksum  │
│  (32 bits)  │   (8 bits)  │  (16 bits)  │  (16 bits)  │  (32 bits)  │
├─────────────┴─────────────┴─────────────┴─────────────┴─────────────┤
│                              Message Payload                              │
├─────────────┬─────────────┬─────────────┬─────────────┬─────────────┤
│   Dilithium │   Reserved  │   Padding   │   End      │             │
│   Signature │   (32 bits) │  (32 bits)  │   Marker   │             │
│  (256 bits) │             │             │  (32 bits) │             │
└─────────────┴─────────────┴─────────────┴─────────────┴─────────────┘
```

## 🖥️ **System Call Table Generator**

### **Syscall Generation Architecture**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Syscall      │    │   Generator     │    │   Output       │
│   Definitions  │    │   Engine        │    │   Files        │
│                 │    │                 │    │                 │
│ • Function     │    │ • Parse         │    │ • Rust         │
│   signatures   │    │   definitions   │    │   bindings     │
│ • Parameter    │    │ • Generate      │    │ • C headers    │
│   types        │    │   bindings      │    │ • Assembly     │
│ • Return       │    │ • Validate      │    │   stubs        │
│   types        │    │   types         │    │ • Tests        │
│ • Error        │    │ • Generate      │    │ • Docs         │
│   codes        │    │   tests         │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### **Syscall Definition Format**
```yaml
syscalls:
  read:
    number: 0
    signature: "read(fd: i32, buf: *mut u8, count: usize) -> Result<usize, Error>"
    parameters:
      - name: fd
        type: i32
        description: "File descriptor"
      - name: buf
        type: "*mut u8"
        description: "Buffer to read into"
      - name: count
        type: usize
        description: "Maximum bytes to read"
    returns:
      type: "Result<usize, Error>"
      description: "Number of bytes read or error"
    errors:
      - EBADF: "Invalid file descriptor"
      - EINVAL: "Invalid parameters"
      - EIO: "I/O error"
    capabilities:
      - READ
      - target: "file:{fd}"
```

## 🔒 **User/Kernel Boundary Enforcement**

### **Boundary Security Model**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              USER SPACE                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   App A     │  │   App B     │  │   App C     │  │   App D     │      │
│  │             │  │             │  │             │  │             │      │
│  │ • Cap       │  │ • Cap       │  │ • Cap       │  │ • Cap       │      │
│  │   Tokens    │  │   Tokens    │  │   Tokens    │  │   Tokens    │      │
│  │ • Syscalls  │  │ • Syscalls  │  │ • Syscalls  │  │ • Syscalls  │      │
│  │ • Memory    │  │ • Memory    │  │ • Memory    │  │ • Memory    │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
├─────────────────────────────────────────────────────────────────────────────┤
│                           SECURITY BOUNDARY                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   Cap       │  │   Memory    │  │   Syscall   │  │   Audit     │      │
│  │   Validator │  │   Protector │  │   Filter    │  │   Logger    │      │
│  │             │  │             │  │             │  │             │      │
│  │ • Token     │  │ • Page      │  │ • Number    │  │ • Security  │      │
│  │   Verify    │  │   Perms     │  │   Validate  │  │   Events    │      │
│  │ • Rights    │  │ • Cap       │  │ • Param     │  │ • Access    │      │
│  │   Check     │  │   Check     │  │   Validate  │  │   Logs      │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
├─────────────────────────────────────────────────────────────────────────────┤
│                              KERNEL SPACE                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   SecMan    │  │   PolyBus   │  │   APIC      │  │   PF        │      │
│  │             │  │             │  │   Timer     │  │   Handler   │      │
│  │ • PQC       │  │ • IPC       │  │             │  │             │      │
│  │   Engine    │  │   Auth      │  │ • High      │  │ • Enhanced  │      │
│  │ • Key       │  │ • Cap       │  │   Precision │  │   Security  │      │
│  │   Store     │  │   Routing   │  │ • Hardware  │  │ • Cap       │      │
│  │ • Token     │  │ • Message   │  │   Timer     │  │   Validation│      │
│  │   Issuer    │  │   Routing   │  │             │  │             │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
└─────────────────────────────────────────────────────────────────────────────┘
```

## ⏱️ **APIC vs PIT Timer Architecture**

### **Timer System Comparison**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              APIC TIMER                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   Local     │  │   Local     │  │   Local     │  │   Local     │      │
│  │   APIC 0    │  │   APIC 1    │  │   APIC 2    │  │   APIC 3    │      │
│  │             │  │             │  │             │  │             │      │
│  │ • 64-bit    │  │ • 64-bit    │  │ • 64-bit    │  │ • 64-bit    │      │
│  │   Counter   │  │   Counter   │  │   Counter   │  │   Counter   │      │
│  │ • High      │  │ • High      │  │ • High      │  │ • High      │      │
│  │   Precision │  │   Precision │  │   Precision │  │   Precision │      │
│  │ • Per-CPU   │  │ • Per-CPU   │  │ • Per-CPU   │  │ • Per-CPU   │      │
│  │   Control   │  │   Control   │  │   Control   │  │   Control   │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
├─────────────────────────────────────────────────────────────────────────────┤
│                              PIT TIMER                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                           Programmable                                  │ │
│  │                        Interval Timer                                  │ │
│  │                                                                         │ │
│  │ • 16-bit Counter                                                        │ │
│  │ • System-wide (not per-CPU)                                            │ │
│  │ • Lower precision (±1ms)                                               │ │
│  │ • Legacy compatibility                                                 │ │
│  │ • Single interrupt source                                              │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

### **Timer Selection Strategy**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Boot         │    │   Runtime       │    │   Fallback      │
│   Detection    │    │   Selection     │    │   Mode          │
│                 │    │                 │    │                 │
│ 1. Check       │    │ 1. APIC         │    │ 1. PIT         │
│    APIC        │    │    Available    │    │    Available    │
│    Presence    │    │ 2. High         │    │ 2. Lower       │
│ 2. Validate    │    │    Precision    │    │    Precision    │
│    Function    │    │    Required     │    │ 3. Legacy      │
│ 3. Initialize  │    │ 3. Per-CPU      │    │    Support     │
│    if Valid    │    │    Control      │    │ 4. System-     │
│                 │    │ 4. Select      │    │    wide        │
│                 │    │    APIC        │    │    Timer       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🧪 **Testing & Validation Architecture**

### **Test Suite Organization**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              PHASE 2 TEST SUITE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   PQC       │  │   ABI       │  │   Security  │  │   Performance│      │
│  │   Tests     │  │   Tests     │  │   Tests     │  │   Tests     │      │
│  │             │  │             │  │             │  │             │      │
│  │ • Kyber     │  │ • Syscall   │  │ • Cap       │  │ • Latency   │      │
│  │   Vector    │  │   Table     │  │   Token     │  │   Benchmarks│      │
│  │ • Dilithium │  │ • ABI       │  │   Validation│  │ • Throughput│      │
│  │   Vector    │  │   Version   │  │ • Boundary  │  │ • Memory    │      │
│  │ • Interop   │  │ • Backward  │  │   Enforcement│  │   Usage     │      │
│  │ • Side-     │  │   Compat    │  │ • Audit     │  │ • CPU       │      │
│  │   Channel   │  │ • Forward   │  │   Logging   │  │   Overhead  │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
├─────────────────────────────────────────────────────────────────────────────┤
│                              INTEGRATION TESTS                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   End-to-   │  │   Stress    │  │   Security  │  │   Regression│      │
│  │   End       │  │   Tests     │  │   Fuzzing   │  │   Tests     │      │
│  │             │  │             │  │             │  │             │      │
│  │ • Full      │  │ • High      │  │ • Cap       │  │ • Phase 1.5 │      │
│  │   Workflow  │  │ • Real      │  │   Token     │  │   Compatibility│    │
│  │ • Real      │  │ • Memory    │  │   Fuzzing   │  │ • Performance│      │
│  │   Scenarios │  │   Pressure  │  │ • Syscall   │  │   Baselines │      │
│  │ • User      │  │ • CPU       │  │   Fuzzing   │  │ • Security  │      │
│  │   Stories   │  │ • Boundary  │  │   Baselines │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 🔄 **Migration & Compatibility Strategy**

### **Phase 1.5 → Phase 2 Transition**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              TRANSITION TIMELINE                           │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │   Phase     │  │   Hybrid    │  │   PQC-      │  │   Full      │      │
│  │   1.5      │  │   Mode      │  │   Primary   │  │   PQC       │      │
│  │             │  │             │  │             │  │             │      │
│  │ • Classical │  │ • Both      │  │ • PQC       │  │ • PQC       │      │
│  │   Crypto    │  │   Crypto    │  │   Primary   │  │   Only      │      │
│  │ • Basic     │  │   Systems   │  │ • Classical │  │ • No        │      │
│  │   Security  │  │ • Gradual   │  │   Fallback  │  │   Fallback  │      │
│  │ • Phase 1.5 │  │   Migration │  │ • Enhanced  │  │ • Maximum   │      │
│  │   Features  │  │ • Testing   │  │   Security  │  │   Security  │      │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘      │
│      Week 0       Week 1-4        Week 5-8        Week 9+                │
└─────────────────────────────────────────────────────────────────────────────┘
```

### **Backward Compatibility Matrix**
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              COMPATIBILITY MATRIX                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  Feature          │ Phase 1.5 │ Hybrid Mode │ PQC-Primary │ Full PQC      │
├───────────────────┼───────────┼─────────────┼─────────────┼───────────────┤
│ Classical Crypto  │    ✅     │     ✅      │     ⚠️      │      ❌       │
│ PQC Operations    │    ❌     │     ✅      │     ✅      │      ✅       │
│ Phase 1.5 APIs   │    ✅     │     ✅      │     ✅      │      ✅       │
│ New PQC APIs     │    ❌     │     ✅      │     ✅      │      ✅       │
│ Performance       │   100%    │    95%      │    90%      │     85%      │
│ Security Level    │   Medium  │    High     │   Higher    │   Highest    │
└───────────────────┴───────────┴─────────────┴─────────────┴───────────────┘
```

---

**Document Version**: 1.0  
**Last Updated**: Phase 2 Kickoff  
**Next Review**: Phase 2 Implementation Planning
