<h1 align="center"> Packet Racers </h1>

Packet Racers is a Rust library designed for exchanging files over a network using TCP and two different UDP implementations: one with guaranteed delivery and one without. The library is optimized for use on Linux and is intended to be integrated into a larger application.

# Features

- [X] Multiple users identified by UUIDs
- [X] File transfer over TCP and two UDP implementations
- [ ] Packet sizes of 100, 500, or 1000 bytes
- [X] Progress updates during file transfers
- [ ] Interruption handling
- [X] No restrictions on file types or sizes
- [X] No authentication or encryption required
- [ ] Network module with list of connected users

# Getting Started

Include the following line in your `Cargo.toml` file to use the Packet Racers library:

```toml
[dependencies]
packet_racers = { version = "0.1.0", features = ["tcp", "udp", "network"] }
```

# Usage

```rust
todo!();
```

# License

Packet Racers is licensed under the *_MIT License_*.
