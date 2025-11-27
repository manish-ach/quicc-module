quicc-module/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, CLI parsing
│   ├── config/
│   │   ├── mod.rs           # Re-exports
│   │   ├── tls.rs           # TLS/certificate config (your current code)
│   │   └── quic.rs          # QUIC transport config (optional)
│   ├── server/
│   │   ├── mod.rs           # Server logic
│   │   └── receiver.rs      # File receiving logic
│   ├── client/
│   │   ├── mod.rs           # Client logic
│   │   └── sender.rs        # File sending logic
│   ├── discovery/
│   │   ├── mod.rs           # Discovery module
│   │   ├── multicast.rs     # UDP multicast logic
│   │   └── peer_table.rs    # Peer management
│   └── protocol/
│       ├── mod.rs           # Protocol definitions
│       └── message.rs       # Message types
