# Build System Diagram

## Mermaid Build Flow

```mermaid
graph TB
    subgraph "Source Code"
        A[Rust Sources]
        B[Protocol Buffers]
        C[Configuration Files]
        D[Documentation]
    end
    
    subgraph "Bazel Build System"
        E[Workspace Rules]
        F[BUILD Files]
        G[Dependency Resolution]
        H[Build Execution]
    end
    
    subgraph "Compilation Pipeline"
        I[Rust Compiler]
        J[Protoc Generator]
        K[Asset Bundler]
        L[Test Runner]
    end
    
    subgraph "Verification Steps"
        M[Unit Tests]
        N[Integration Tests]
        O[Security Tests]
        P[Linting]
    end
    
    subgraph "Artifacts"
        Q[Binaries]
        R[Libraries]
        S[Documentation]
        T[Container Images]
    end
    
    subgraph "Distribution"
        U[Package Registry]
        V[Container Registry]
        W[GitHub Releases]
        X[Documentation Site]
    end
    
    %% Source to Build System
    A --> E
    B --> F
    C --> G
    D --> H
    
    %% Build System to Compilation
    E --> I
    F --> J
    G --> K
    H --> L
    
    %% Compilation to Verification
    I --> M
    J --> N
    K --> O
    L --> P
    
    %% Verification to Artifacts
    M --> Q
    N --> R
    O --> S
    P --> T
    
    %% Artifacts to Distribution
    Q --> U
    R --> V
    S --> W
    T --> X
    
    classDef source fill:#e3f2fd,stroke:#1565c0,stroke-width:2px
    classDef build fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef compile fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef verify fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef artifacts fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    classDef distribution fill:#f1f8e9,stroke:#558b2f,stroke-width:2px
    
    class A,B,C,D source
    class E,F,G,H build
    class I,J,K,L compile
    class M,N,O,P verify
    class Q,R,S,T artifacts
    class U,V,W,X distribution
```

## PlantUML Build Process

```plantuml
@startuml build-system

!define RECTANGLE class

package "Source Management" {
    [Git Repository] as Git
    [Source Files] as Sources
    [BUILD Files] as BuildFiles
    [Configuration] as Config
}

package "Bazel Build System" {
    [Workspace] as Workspace
    [Target Resolution] as Targets
    [Dependency Graph] as DepGraph
    [Build Actions] as Actions
}

package "Compilation Tools" {
    [Rust Toolchain] as RustChain
    [Protocol Buffers] as Protobuf
    [LLVM/Clang] as LLVM
    [Linker] as Linker
}

package "Testing Framework" {
    [Unit Test Runner] as UnitTests
    [Integration Tests] as IntegrationTests
    [Fuzz Testing] as FuzzTests
    [Security Scanner] as SecurityScan
}

package "Quality Gates" {
    [Linting] as Lint
    [Code Coverage] as Coverage
    [Security Audit] as Audit
    [Performance Tests] as PerfTests
}

package "Artifact Generation" {
    [Binary Output] as Binaries
    [Library Output] as Libraries
    [Documentation] as Docs
    [Container Images] as Containers
}

' Source to Bazel
Git --> Workspace
Sources --> Targets
BuildFiles --> DepGraph
Config --> Actions

' Bazel to Tools
Workspace --> RustChain
Targets --> Protobuf
DepGraph --> LLVM
Actions --> Linker

' Tools to Testing
RustChain --> UnitTests
Protobuf --> IntegrationTests
LLVM --> FuzzTests
Linker --> SecurityScan

' Testing to Quality
UnitTests --> Lint
IntegrationTests --> Coverage
FuzzTests --> Audit
SecurityScan --> PerfTests

' Quality to Artifacts
Lint --> Binaries
Coverage --> Libraries
Audit --> Docs
PerfTests --> Containers

note right of Workspace : Hermetic builds\nwith reproducible\noutputs
note right of RustChain : Cross-compilation\nsupport for multiple\narchitectures
note right of SecurityScan : Static analysis\nand vulnerability\nscanning
note right of Binaries : Signed binaries\nwith SBOM metadata

@enduml
```

## Dependency Graph

```mermaid
graph TD
    subgraph "External Dependencies"
        A[Crates.io]
        B[GitHub Packages]
        C[System Libraries]
        D[Tool Binaries]
    end
    
    subgraph "Internal Dependencies"
        E[Kernel Crate]
        F[Services Crates]
        G[Crypto Crates]
        H[Utility Crates]
    end
    
    subgraph "Generated Code"
        I[Proto Generated]
        J[Schema Generated]
        K[Config Generated]
        L[Documentation]
    end
    
    subgraph "Build Outputs"
        M[Kernel Binary]
        N[Service Binaries]
        O[Library Archives]
        P[Test Binaries]
    end
    
    %% External to Internal
    A --> E
    A --> F
    B --> G
    C --> H
    
    %% Generated dependencies
    I --> E
    I --> F
    J --> G
    K --> H
    
    %% Internal to Outputs
    E --> M
    F --> N
    G --> O
    H --> P
    
    %% Generated to Outputs
    I --> N
    J --> O
    L --> P
    
    classDef external fill:#ffebee,stroke:#c62828
    classDef internal fill:#e8f5e8,stroke:#2e7d32
    classDef generated fill:#fff3e0,stroke:#ef6c00
    classDef outputs fill:#e3f2fd,stroke:#1565c0
    
    class A,B,C,D external
    class E,F,G,H internal
    class I,J,K,L generated
    class M,N,O,P outputs
```

## Build Configuration

### Workspace Configuration
```python
# WORKSPACE file
workspace(name = "polymera_os")

# Rust rules
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "...",
    urls = ["https://github.com/bazelbuild/rules_rust/releases/..."],
)

load("@rules_rust//rust:repositories.bzl", "rules_rust_dependencies", "rust_register_toolchains")
rules_rust_dependencies()

rust_register_toolchains(
    edition = "2021",
    versions = ["1.70.0"],
)

# Protocol buffer rules
http_archive(
    name = "rules_proto",
    sha256 = "...",
    urls = ["https://github.com/bazelbuild/rules_proto/releases/..."],
)

# Container rules
http_archive(
    name = "rules_oci",
    sha256 = "...",
    urls = ["https://github.com/bazel-contrib/rules_oci/releases/..."],
)
```

### BUILD File Example
```python
# services/keystore/BUILD

load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_test")
load("@rules_proto//proto:defs.bzl", "proto_library")

proto_library(
    name = "keystore_proto",
    srcs = ["keystore.proto"],
    deps = ["//proto:common_proto"],
)

rust_library(
    name = "keystore_lib",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":keystore_proto",
        "//crypto:crypto_lib",
        "@crates//:tokio",
        "@crates//:serde",
        "@crates//:tracing",
    ],
    proc_macro_deps = [
        "@crates//:serde_derive",
    ],
)

rust_binary(
    name = "keystore_service",
    srcs = ["src/main.rs"],
    deps = [":keystore_lib"],
)

rust_test(
    name = "keystore_tests",
    crate = ":keystore_lib",
    deps = [
        "@crates//:tokio-test",
        "@crates//:tempfile",
    ],
)

# Fuzz testing
rust_binary(
    name = "keystore_fuzz",
    srcs = ["fuzz/fuzz_targets/keystore.rs"],
    deps = [
        ":keystore_lib",
        "@crates//:libfuzzer-sys",
    ],
)
```

## Build Targets

```mermaid
graph LR
    subgraph "Core Targets"
        A[//kernel:polymera_kernel]
        B[//services:all]
        C[//crypto:crypto_lib]
        D[//proto:all_protos]
    end
    
    subgraph "Service Targets"
        E[//services/keystore:keystore_service]
        F[//services/identity:identity_service]
        G[//services/privacy:privacy_service]
        H[//services/network:network_service]
    end
    
    subgraph "Test Targets"
        I[//...:unit_tests]
        J[//...:integration_tests]
        K[//...:fuzz_tests]
        L[//...:security_tests]
    end
    
    subgraph "Distribution Targets"
        M[//dist:container_images]
        N[//dist:release_artifacts]
        O[//docs:documentation_site]
        P[//tools:developer_tools]
    end
    
    A --> E
    B --> F
    C --> G
    D --> H
    
    E --> I
    F --> J
    G --> K
    H --> L
    
    I --> M
    J --> N
    K --> O
    L --> P
    
    classDef core fill:#e3f2fd,stroke:#1565c0
    classDef services fill:#f3e5f5,stroke:#7b1fa2
    classDef tests fill:#e8f5e8,stroke:#2e7d32
    classDef distribution fill:#fff3e0,stroke:#ef6c00
    
    class A,B,C,D core
    class E,F,G,H services
    class I,J,K,L tests
    class M,N,O,P distribution
```

## Cross-Compilation Support

```mermaid
graph TB
    subgraph "Host Platform"
        A[x86_64-unknown-linux-gnu]
    end
    
    subgraph "Target Platforms"
        B[x86_64-pc-windows-msvc]
        C[aarch64-unknown-linux-gnu]
        D[x86_64-apple-darwin]
        E[aarch64-apple-darwin]
        F[wasm32-unknown-unknown]
    end
    
    subgraph "Cross-Compilation Tools"
        G[Cross Compiler]
        H[Target Linkers]
        I[Platform Libraries]
        J[Target SDKs]
    end
    
    A --> G
    G --> B
    G --> C
    G --> D
    G --> E
    G --> F
    
    H --> B
    H --> C
    I --> D
    I --> E
    J --> F
    
    classDef host fill:#e3f2fd,stroke:#1565c0
    classDef targets fill:#e8f5e8,stroke:#2e7d32
    classDef tools fill:#fff3e0,stroke:#ef6c00
    
    class A host
    class B,C,D,E,F targets
    class G,H,I,J tools
```

## Build Performance

### Build Metrics
```mermaid
graph LR
    subgraph "Performance Metrics"
        A[Build Time]
        B[Cache Hit Rate]
        C[Parallelization]
        D[Resource Usage]
    end
    
    subgraph "Target Values"
        E[< 5 minutes clean]
        F[> 90% cache hits]
        G[16+ parallel jobs]
        H[< 8GB RAM]
    end
    
    A --> E
    B --> F
    C --> G
    D --> H
    
    classDef metrics fill:#e3f2fd,stroke:#1565c0
    classDef targets fill:#e8f5e8,stroke:#2e7d32
    
    class A,B,C,D metrics
    class E,F,G,H targets
```

### Cache Strategy
- **Local Cache**: Build outputs cached locally for faster rebuilds
- **Remote Cache**: Shared cache across development team
- **Content Addressing**: Hermetic builds with content-based caching
- **Incremental Builds**: Only rebuild changed components

## Continuous Integration

### Build Pipeline
```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Git as Git Repository
    participant CI as CI System
    participant Cache as Build Cache
    participant Reg as Registry
    
    Dev->>Git: Push Changes
    Git->>CI: Trigger Build
    CI->>Cache: Check Cache
    Cache->>CI: Cache Miss/Hit
    CI->>CI: Execute Build
    CI->>CI: Run Tests
    CI->>CI: Security Scan
    CI->>Reg: Publish Artifacts
    CI->>Dev: Build Status
```

### Quality Gates
1. **Compilation**: All targets must compile successfully
2. **Unit Tests**: All unit tests must pass
3. **Integration Tests**: Integration test suite must pass
4. **Security Scan**: No high-severity security issues
5. **Linting**: Code style and quality checks
6. **Coverage**: Minimum code coverage thresholds

---

This build system diagram shows the comprehensive Bazel-based build infrastructure for Polymera OS, including source management, compilation pipelines, testing frameworks, and artifact generation with support for cross-platform builds and continuous integration.
