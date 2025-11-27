---

# 1. Architecture

### Components

1. **Discovery**
   Peers broadcast their presence on the LAN. Two typical options:

   * **mDNS** (via `libmdns` or `async-mdns`).
   * **UDP broadcast** (simpler to implement).

2. **Session establishment (QUIC via Quinn)**
   Each peer exposes a QUIC listener (server mode) and also acts as a QUIC client when sending.

3. **Protocol**

   * Control handshake: nickname, capabilities.
   * File metadata: filename, filesize.
   * Chunk streaming using QUIC bidirectional stream.

4. **Transfer**

   * Sender opens a bidi stream.
   * Pushes file in fixed-size chunks (e.g., 64 KiB).
   * Receiver writes to disk until stream ends.

---

# 2. Directory layout

```
quicc/
  src/
    main.rs         # CLI: send/receive modes
    discovery.rs    # find peers
    peer.rs         # peer struct {name, ip}
    quic.rs         # Quinn setup
    send.rs         # send file
    recv.rs         # receive file
```

---

# 3. Discovery (UDP broadcast, simpler than mDNS)

### Sender + Receiver logic (both sides run this)

* Bind to a local UDP socket: `0.0.0.0:9555`
* Every 2 seconds broadcast `"QUICC|<nickname>|<quic_port>"` to `255.255.255.255:9555`.
* Listen on the same socket to discover peers.

Minimal implementation idea:

```rust
// discovery.rs
pub async fn run_discovery(my_name: String, quic_port: u16) -> Receiver<Peer> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    let sock = UdpSocket::bind("0.0.0.0:9555").await.unwrap();
    sock.set_broadcast(true).unwrap();

    let sock_send = sock.try_clone().unwrap();
    tokio::spawn(async move {
        let msg = format!("QUICC|{my_name}|{quic_port}");
        loop {
            let _ = sock_send.send_to(msg.as_bytes(), "255.255.255.255:9555").await;
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    tokio::spawn(async move {
        let mut buf = [0u8; 256];
        loop {
            let (n, addr) = sock.recv_from(&mut buf).await.unwrap();
            if let Ok(s) = std::str::from_utf8(&buf[..n]) {
                if let Some(peer) = parse_peer(s, addr) {
                    let _ = tx.send(peer).await;
                }
            }
        }
    });

    rx
}
```

Peers update a list like:

```rust
HashMap<SocketAddr, Peer>  // nickname, last_seen, quic_port
```

---

# 4. QUIC listener (every peer acts as a server)

```rust
// quic.rs
pub async fn start_quic_server(port: u16) -> Endpoint {
    let cert = ...;  // self-signed
    let key = ...;

    let mut server_config = ServerConfig::with_single_cert(cert, key).unwrap();
    server_config.use_retry(true);

    Endpoint::server(server_config, ([0,0,0,0], port).into()).unwrap()
}
```

This returns an `Endpoint`. You must:

* Handle incoming connections.
* For each connection, `accept_bi()` returns a bidi stream.
* That stream is a file transfer.

---

# 5. File sending

### Protocol

1. Write metadata:

   ```
   FILENAME:<bytes>\n
   FILESIZE:<u64>\n
   ENDHDR\n
   ```
2. Send file in chunks.

```rust
// send.rs
pub async fn send_file(path: &Path, conn: Connection) -> Result<(), Box<dyn Error>> {
    let mut stream = conn.open_bi().await?;
    let (mut send, _) = stream;

    let data = tokio::fs::read(path).await?;
    let size = data.len() as u64;
    let name = path.file_name().unwrap().to_string_lossy();

    send.write_all(format!("FILENAME:{name}\nFILESIZE:{size}\nENDHDR\n").as_bytes()).await?;
    send.write_all(&data).await?;
    send.finish().await?;
    Ok(())
}
```

---

# 6. File receiving

```rust
// recv.rs
pub async fn handle_incoming(mut recv: RecvStream, save_dir: &Path) -> Result<(), Box<dyn Error>> {
    let mut header = Vec::new();
    loop {
        let mut buf = [0u8; 1];
        recv.read_exact(&mut buf).await?;
        header.push(buf[0]);

        if header.ends_with(b"ENDHDR\n") {
            break;
        }
    }

    let hdr = String::from_utf8_lossy(&header);
    let filename = extract("FILENAME:", &hdr);
    let filesize = extract("FILESIZE:", &hdr).parse::<u64>()?;

    let mut file = tokio::fs::File::create(save_dir.join(filename)).await?;
    let mut remaining = filesize;

    let mut buf = [0u8; 64 * 1024];

    while remaining > 0 {
        let n = recv.read(&mut buf).await?.unwrap_or(0);
        if n == 0 { break; }
        file.write_all(&buf[..n]).await?;
        remaining -= n as u64;
    }

    Ok(())
}
```

---

# 7. CLI

`quicc send <peer-name> <file>`
`quicc recv` (always running)
`quicc peers` (list discovered peers + their QUIC ports)

Internally:

* Discovery updates a peer list.
* `send <peer>` → lookup IP → QUIC connect → send file.

---

# 8. Boot sequence when running the program

1. Generate self-signed cert once at startup if missing.
2. Start QUIC server on a random port (e.g., 5000 + random offset).
3. Start discovery (broadcast nickname + port).
4. Maintain peer list.
5. On “send <peer> <file>”:

   * Resolve peer IP + QUIC port.
   * `connect()`
   * `send_file()`

---
