# Sphero RVR Rust Library - Copilot Instructions

## Build & Test Commands

```bash
# Build (host)
cargo build

# Build for Raspberry Pi (cross-compile)
cargo build --target=aarch64-unknown-linux-gnu --release --example hello_rvr

# Run all tests
cargo test

# Run a single test by name
cargo test test_encode_decode_roundtrip

# Run tests in a specific module
cargo test protocol::framing::tests

# Run with logging
RUST_LOG=debug cargo test

# Deploy to Raspberry Pi
./deploy.sh --example hello_rvr
./deploy.sh --example hello_rvr --run          # deploy and run
./deploy.sh --release --example hello_rvr      # optimized build
```

## Architecture

The library has three strict layers with no upward dependencies:

```
API Layer (src/api/)          ← synchronous, strongly typed, no protocol knowledge
      ↓
Transport Layer (src/transport/)  ← owns serial port, manages threads, routes packets
      ↓
Protocol Layer (src/protocol/)    ← pure functions/state machines, no I/O
```

**Protocol Layer** (`src/protocol/`): Stateless byte manipulation.
- `framing.rs`: SLIP-style encoding/decoding. Special bytes `SOP=0x8D`, `EOP=0xD8`, `ESC=0xAB`. Escape: `ESC + (byte & !0x88)`. Decode: `escaped | 0x88`.
- `checksum.rs`: `!sum(bytes) & 0xFF` over all bytes between SOP/EOP (excluding them).
- `packet.rs`: `Packet` struct and `PacketFlags` bitfield. `Packet::to_bytes()` produces unescaped payload (no SOP/EOP). `Packet::from_bytes()` expects the same format.
- `parser.rs`: `SpheroParser` state machine — feed bytes one at a time via `parser.feed(byte)`, returns `Ok(Some(Packet))` when complete.

**Transport Layer** (`src/transport/dispatcher.rs`): `Dispatcher` owns the serial port.
- Spawns a background RX thread that reads 1024-byte chunks to avoid per-byte mutex contention.
- `send_command()` assigns a sequence number, registers a `mpsc::Sender<Packet>` in `pending_requests: HashMap<u8, Sender>`, sends the framed packet, then blocks on the receiver with a 2-second timeout.
- RX thread routes: `is_response=true` → looked up by seq number and sent to the waiting caller; otherwise → pushed to `notification_tx` channel.
- `take_receiver()` transfers ownership of the notification `Receiver<Packet>` (one-time call). Callers should spawn a dedicated thread for it.
- Sequence numbers are `AtomicU8` and wrap naturally at 256.

**API Layer** (`src/api/`): `SpheroRvr` wraps `Dispatcher`.
- `SpheroRvr::connect(port)` is the single entry point — opens port at 115200 baud.
- `build_command()` always sets `has_target_id=true` / `has_source_id=true` with `target=PRIMARY_PROCESSOR(0x01)` / `source=UART_PORT(0x02)`. **This UART routing is required** — packets without it will be silently dropped by the RVR's internal mesh.
- `check_response()` reads `payload[0]` as an error code; empty payload means success.
- All constants (device IDs, command IDs, LED bitmasks, error codes) live in `src/api/constants.rs` as nested `pub mod` blocks.

## Key Conventions

**No async runtime.** The library deliberately uses `std::thread` and `std::sync::mpsc`. Do not introduce `tokio`, `async-std`, or similar.

**`serialport` without default features.** `default-features = false` drops `libudev-sys` to enable cross-compilation. Do not re-enable it.

**`#![allow(dead_code)]` and `#![allow(unused_imports)]`** are intentional during active development phases — do not remove them.

**Packet framing pipeline (TX):**
```
Packet::to_bytes() → encode_bytes() → [SOP] + escaped + [EOP] → write to port
```

**Packet framing pipeline (RX):**
```
raw bytes → SpheroParser::feed() → Packet::from_bytes() (decoder strips SOP/EOP, unescapes)
```

**Tests that require serial hardware** skip gracefully by returning early when `Dispatcher::new("/dev/null", 115200)` fails. Follow this pattern for any new tests in the API layer.

**Tracing, not `log`.** The crate uses `tracing` / `tracing-subscriber` for structured logging. Use `tracing::debug!`, `tracing::trace!`, `tracing::warn!`, etc.

**Hardware constraint:** RVR UART is 3.3V only. The serial port on Raspberry Pi is `/dev/serial0` with `enable_uart=1` and `dtoverlay=disable-bt` in `/boot/config.txt`.
