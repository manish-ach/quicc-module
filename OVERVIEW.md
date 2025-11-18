# 1. System Overview

The program is a **peer-to-peer file transfer system on a local network**.
It uses:

1. **UDP multicast** → to discover peers and broadcast nicknames.
2. **QUIC (via Quinn)** → to transfer files between selected peers.
3. **Send/Receive modes** → users choose whether to send a file or wait to receive one.
4. **Nickname identity** → each device announces its chosen name over the network.

This forms a lightweight AirDrop-like system for LAN.

---

# 2. Peer Discovery Theory (UDP Multicast)

LAN devices normally do not know each other's IP addresses.
To solve this, each device sends a broadcast-style message:

```
nickname | ip:port
```

via **UDP multicast** to a group address (e.g., 239.255.0.1:4000).

All devices subscribed to the group receive these messages.

### Why multicast?

* Works on any LAN without servers
* One-to-many
* Lightweight
* No NAT problems inside LAN
* Simple to implement with async sockets

Each device listens and builds a table of peers:

```
{
   "aarav": (192.168.1.14:5000),
   "sujit": (192.168.1.20:5000)
}
```

This table updates continuously because peers send an announcement every few seconds.

---

# 3. QUIC Theory (Quinn Library)

After discovery, actual file transfer uses **QUIC**, not UDP multicast.

QUIC provides:

* Reliable delivery (like TCP)
* Stream-based communication
* Faster handshakes (0-RTT)
* Built-in encryption (TLS)
* Works over UDP

Quinn is a Rust implementation of QUIC.

### Why QUIC instead of TCP?

* Faster connection setup on LAN
* Multiplexed streams
* Built-in encryption without extra setup
* Modern, efficient, easy async API

The program uses:

* **Unidirectional streams** for sending file bytes
* **Server endpoint** for receiving
* **Client endpoint** for sending

---

# 4. Send/Receive Modes Theory

The application has two modes:

### Send Mode:

1. Wait for discovery results
2. User selects a peer
3. Establish QUIC connection
4. Open a unidirectional stream
5. Read file → send bytes
6. Close the stream

### Receive Mode:

1. Always run QUIC server endpoint
2. When a peer connects and sends a stream:

   * Read bytes
   * Write to a file (e.g., received.bin)
   * Close stream when no more data

Receive mode is passive. Send mode is active.

Both modes can run in parallel, so you always accept incoming transfers even if you are in "send mode" later.

---

# 5. Nickname System Theory

Users type a nickname on startup.

Nickname is included in the discovery packet:

```
NICKNAME | IP:PORT
```

Other devices store this to identify the peer.

This avoids showing raw IP addresses in UI.

---

# 6. Process Flow Theory (Full System)

### Step 1 — Start the app

User sets a nickname.
App starts UDP and QUIC tasks.

### Step 2 — Join multicast group

Program listens for peer announcements on the LAN.

### Step 3 — Broadcast identity

Every 2 seconds:

```
Manish|192.168.1.23:5000
```

### Step 4 — Build peer list

Each new announcement updates the peer directory.

### Step 5 — User selects Send or Receive

* Receive: do nothing; wait for incoming QUIC connections.
* Send: choose a peer from the discovered list.

### Step 6 — QUIC connection

Sender opens a QUIC connection to the selected peer’s `ip:port`.

### Step 7 — Data transfer

Sender uses a QUIC **unidirectional stream** to push file bytes.
Receiver reads bytes and writes to disk.

### Step 8 — Completion

Both sides close the stream and connection cleanly.

---

# 7. Why This Architecture Works (Important Theory)

1. **No centralized server.**
   Peers discover each other without any external system.

2. **LAN-only simplifies NAT.**
   Multicast and QUIC both work perfectly on local networks.

3. **QUIC is suitable for high-speed LAN transfers.**
   Low-latency, secure, reliable.

4. **Separation of roles keeps control simple.**
   Even though both devices are peers, one acts as client during transfer, the other acts as server.

5. **Nickname + multicast = human-friendly addressing.**
   Users choose by name, not IP.

---
