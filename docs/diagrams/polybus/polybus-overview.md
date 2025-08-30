# PolyBus Overview Diagram

## Mermaid Architecture Diagram

```mermaid
graph TB
    subgraph "Application Layer"
        A[Client Applications]
        B[System Services]
        C[gRPC Services]
        D[Web Interfaces]
    end
    
    subgraph "PolyBus Message Layer"
        E[Message Router]
        F[Topic Manager]
        G[Subscription Handler]
        H[Message Serializer]
    end
    
    subgraph "PolyBus Transport Layer"
        I[Transport Abstraction]
        J[QUIC Transport]
        K[WebSocket Transport]
        L[Unix Socket Transport]
    end
    
    subgraph "Security & Privacy Layer"
        M[Message Encryption]
        N[Authentication]
        O[Privacy Budget Tracker]
        P[Access Control]
    end
    
    subgraph "Network Layer"
        Q[Mesh Network Manager]
        R[DTN Protocol]
        S[Discovery Service]
        T[Routing Engine]
    end
    
    subgraph "Storage Layer"
        U[Message Store]
        V[Topic Store]
        W[Subscription Store]
        X[Delivery Queue]
    end
    
    %% Application to Message Layer
    A --> E
    B --> F
    C --> G
    D --> H
    
    %% Message Layer to Transport
    E --> I
    F --> J
    G --> K
    H --> L
    
    %% Transport to Security
    I --> M
    J --> N
    K --> O
    L --> P
    
    %% Security to Network
    M --> Q
    N --> R
    O --> S
    P --> T
    
    %% Network to Storage
    Q --> U
    R --> V
    S --> W
    T --> X
    
    %% Cross-cutting concerns
    E -.-> M
    F -.-> N
    G -.-> O
    E -.-> U
    F -.-> V
    G -.-> W
    
    classDef application fill:#e3f2fd,stroke:#1565c0,stroke-width:2px
    classDef message fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef transport fill:#e8f5e8,stroke:#2e7d32,stroke-width:2px
    classDef security fill:#fff3e0,stroke:#ef6c00,stroke-width:2px
    classDef network fill:#fce4ec,stroke:#c2185b,stroke-width:2px
    classDef storage fill:#f1f8e9,stroke:#558b2f,stroke-width:2px
    
    class A,B,C,D application
    class E,F,G,H message
    class I,J,K,L transport
    class M,N,O,P security
    class Q,R,S,T network
    class U,V,W,X storage
```

## PlantUML Component Diagram

```plantuml
@startuml polybus-overview

!define RECTANGLE class

package "PolyBus Core" {
    [Message Router] as Router
    [Topic Manager] as TopicMgr
    [Subscription Engine] as SubEngine
    [Message Broker] as Broker
}

package "Transport Layer" {
    [QUIC Transport] as QUIC
    [WebSocket Transport] as WS
    [Unix Socket Transport] as Unix
    [Transport Manager] as TransportMgr
}

package "Security Layer" {
    [Message Encryption] as Encryption
    [Authentication] as Auth
    [Authorization] as Authz
    [Privacy Enforcer] as Privacy
}

package "Network Layer" {
    [Mesh Network] as Mesh
    [DTN Protocol] as DTN
    [Discovery Service] as Discovery
    [Peer Manager] as PeerMgr
}

package "Storage Layer" {
    [Message Store] as MsgStore
    [Topic Store] as TopicStore
    [Subscription Store] as SubStore
    [Delivery Queue] as Queue
}

package "API Layer" {
    [gRPC API] as GrpcAPI
    [REST API] as RestAPI
    [WebSocket API] as WsAPI
    [SDK Interface] as SDK
}

package "Client Applications" {
    [Service A] as ServiceA
    [Service B] as ServiceB
    [Web App] as WebApp
    [Mobile App] as MobileApp
}

' Client to API
ServiceA --> GrpcAPI
ServiceB --> RestAPI
WebApp --> WsAPI
MobileApp --> SDK

' API to Core
GrpcAPI --> Router
RestAPI --> TopicMgr
WsAPI --> SubEngine
SDK --> Broker

' Core to Transport
Router --> TransportMgr
TopicMgr --> QUIC
SubEngine --> WS
Broker --> Unix

' Transport to Security
TransportMgr --> Encryption
QUIC --> Auth
WS --> Authz
Unix --> Privacy

' Security to Network
Encryption --> Mesh
Auth --> DTN
Authz --> Discovery
Privacy --> PeerMgr

' Network to Storage
Mesh --> MsgStore
DTN --> TopicStore
Discovery --> SubStore
PeerMgr --> Queue

' Cross-cutting
Router --> MsgStore
TopicMgr --> TopicStore
SubEngine --> SubStore
Broker --> Queue

Router --> Encryption
TopicMgr --> Auth
SubEngine --> Privacy

note right of Router : Core message routing\nwith topic-based\npublish/subscribe
note right of Encryption : End-to-end encryption\nwith post-quantum\ncryptography
note right of Mesh : Decentralized mesh\nnetworking with\nself-healing topology
note right of MsgStore : Persistent message\nstorage with\ncompression

@enduml
```

## PolyBus Message Flow

```mermaid
sequenceDiagram
    participant P as Publisher
    participant R as Router
    participant TM as Topic Manager
    participant SE as Security Engine
    participant NL as Network Layer
    participant S as Subscriber
    
    Note over P,S: Message Publication Flow
    
    P->>R: Publish Message
    R->>TM: Route to Topic
    TM->>SE: Encrypt & Sign
    SE->>NL: Send to Network
    
    Note over NL: Message Routing
    NL->>NL: Find Subscribers
    NL->>NL: Apply Routing Policy
    
    Note over NL,S: Message Delivery
    NL->>SE: Deliver to Subscriber
    SE->>SE: Decrypt & Verify
    SE->>TM: Forward Message
    TM->>R: Route to Subscriber
    R->>S: Deliver Message
    
    Note over S: Acknowledgment
    S->>R: Send ACK
    R->>TM: Process ACK
    TM->>SE: Sign ACK
    SE->>NL: Send ACK
    NL->>P: Delivery Confirmation
```

## Topic Hierarchy

```mermaid
graph TD
    A[Root Topic Space] --> B[system]
    A --> C[services]
    A --> D[applications]
    A --> E[events]
    
    B --> B1[kernel]
    B --> B2[network]
    B --> B3[security]
    
    C --> C1[keystore]
    C --> C2[identity]
    C --> C3[privacy]
    
    D --> D1[user-apps]
    D --> D2[system-apps]
    D --> D3[third-party]
    
    E --> E1[audit]
    E --> E2[alerts]
    E --> E3[metrics]
    
    B1 --> B1A[memory]
    B1 --> B1B[processes]
    B1 --> B1C[scheduler]
    
    C1 --> C1A[keys.created]
    C1 --> C1B[keys.rotated]
    C1 --> C1C[keys.revoked]
    
    E1 --> E1A[security.events]
    E1 --> E1B[access.logs]
    E1 --> E1C[privacy.budget]
    
    classDef rootTopic fill:#e1f5fe,stroke:#01579b,stroke-width:3px
    classDef categoryTopic fill:#f3e5f5,stroke:#4a148c,stroke-width:2px
    classDef serviceTopic fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px
    classDef eventTopic fill:#fff3e0,stroke:#e65100,stroke-width:2px
    
    class A rootTopic
    class B,C,D,E categoryTopic
    class B1,B2,B3,C1,C2,C3,D1,D2,D3 serviceTopic
    class E1,E2,E3,B1A,B1B,B1C,C1A,C1B,C1C,E1A,E1B,E1C eventTopic
```

## PolyBus Quality of Service

```mermaid
graph LR
    subgraph "QoS Levels"
        A[Best Effort]
        B[At Least Once]
        C[Exactly Once]
        D[Ordered Delivery]
    end
    
    subgraph "Message Properties"
        E[TTL]
        F[Priority]
        G[Persistence]
        H[Routing Hints]
    end
    
    subgraph "Delivery Guarantees"
        I[Fire and Forget]
        J[Acknowledged]
        K[Transactional]
        L[Durable]
    end
    
    A --> I
    B --> J
    C --> K
    D --> L
    
    E --> A
    F --> B
    G --> C
    H --> D
    
    classDef qos fill:#e3f2fd,stroke:#1565c0
    classDef properties fill:#f3e5f5,stroke:#7b1fa2
    classDef guarantees fill:#e8f5e8,stroke:#2e7d32
    
    class A,B,C,D qos
    class E,F,G,H properties
    class I,J,K,L guarantees
```

## PolyBus Configuration

### Transport Configuration
```yaml
# PolyBus transport configuration
transports:
  quic:
    enabled: true
    bind_address: "0.0.0.0:4001"
    max_connections: 1000
    idle_timeout: "30s"
    keep_alive: "10s"
    
  websocket:
    enabled: true
    bind_address: "0.0.0.0:8080"
    path: "/polybus/ws"
    compression: true
    
  unix_socket:
    enabled: true
    socket_path: "/var/run/polybus.sock"
    permissions: "0660"

security:
  encryption: "chacha20poly1305"
  signature: "dilithium3"
  key_exchange: "kyber768"
  
routing:
  strategy: "mesh"
  max_hops: 10
  cache_size: 10000
  cleanup_interval: "5m"

storage:
  backend: "sled"
  path: "/var/lib/polybus"
  max_message_size: "64MB"
  retention_policy: "7d"
```

### Topic Configuration
```yaml
# Topic management configuration
topics:
  default_retention: "24h"
  max_subscribers: 1000
  message_ordering: true
  compression: "lz4"
  
  policies:
    "system.**":
      retention: "30d"
      priority: "high"
      encryption: "required"
      
    "services.**":
      retention: "7d"
      priority: "normal"
      replication: 3
      
    "events.audit.**":
      retention: "1y"
      priority: "high"
      immutable: true
```

## Performance Characteristics

### Throughput and Latency
```mermaid
graph LR
    subgraph "Performance Metrics"
        A[Message Throughput]
        B[End-to-End Latency]
        C[Memory Usage]
        D[CPU Utilization]
    end
    
    subgraph "Target Values"
        E[1M msg/sec]
        F[< 10ms p99]
        G[< 100MB RSS]
        H[< 50% CPU]
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

## Monitoring and Observability

### Metrics Collection
- **Message Rates**: Published, delivered, failed messages per second
- **Latency Distribution**: P50, P95, P99 end-to-end latency
- **Resource Usage**: Memory, CPU, network bandwidth utilization
- **Error Rates**: Failed deliveries, timeouts, authentication failures

### Health Checks
- **Transport Health**: Connection status and performance
- **Storage Health**: Disk usage and write performance
- **Network Health**: Peer connectivity and routing efficiency
- **Security Health**: Certificate validity and encryption status

---

This PolyBus overview diagram illustrates the complete messaging infrastructure for Polymera OS, showing how components communicate through a secure, privacy-preserving, and high-performance message bus with support for mesh networking and delay-tolerant protocols.
