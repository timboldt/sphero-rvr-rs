# Sphero RVR Rust Library - Development Guide

## Project Overview

This is a Rust library for controlling Sphero RVR robots via UART serial communication, specifically designed for Raspberry Pi 3B+ (but portable to other platforms).

### Architecture

The library follows a layered architecture:

1. **Protocol Layer** (`src/protocol/`): Low-level packet handling
   - Packet structure (SOP, flags, IDs, payload, checksum, EOP)
   - SLIP-style encoding/decoding for special characters
   - Checksum calculation and verification

2. **Connection Layer** (`src/connection.rs`): Serial port management
   - Async I/O using tokio-serial
   - Sequence number tracking
   - Connection lifecycle management

3. **Command Layer** (`src/commands/`): High-level API
   - Command builder pattern
   - Type-safe command construction
   - Stage 2+: LED control, status queries, etc.

4. **Response Layer** (`src/response.rs`): Response parsing
   - Packet-to-response conversion
   - Error code handling
   - Success/failure determination

### Implementation Stages

**Stage 1 (Complete)**: Baseline project setup
- ✅ Project structure and dependencies
- ✅ Protocol infrastructure (packet encoding, SLIP, checksums)
- ✅ Connection management
- ✅ Basic connection example
- ✅ Cross-compilation and deployment tooling

**Stage 2 (Complete)**: LED control and status queries
- ✅ LED control commands (`set_all_leds`)
- ✅ Battery status queries (`get_battery_percentage`, `get_battery_voltage_state`)
- ✅ Power management (`wake`, `sleep`)
- ✅ Command/response handling with packet serialization
- ✅ Example programs (led_control, battery_status, power_management)
- ⏳ Hardware testing (pending physical RVR access)

**Stage 3 (Next)**: Full API implementation
- Motor control and driving commands
- Sensor data streaming
- Complete Sphero RVR API coverage
- Advanced features and optimizations

## Technical Details

### Sphero RVR Protocol

Based on Sphero SDK documentation:

**Packet Structure:**
```
[SOP] [FLAGS] [TARGET_ID?] [SOURCE_ID?] [DEVICE_ID] [COMMAND_ID] [SEQ] [PAYLOAD...] [CHECKSUM] [EOP]
```

**Key Specifications:**
- Baud rate: 115200
- Voltage: 3.3V (NOT 5V tolerant!)
- Encoding: SLIP-style with ESC sequences
- Checksum: `0xFF - (sum of bytes & 0xFF)`
- Byte order: Big-endian

**SLIP Encoding:**
- Special bytes (0x8D, 0xD8, 0xAB) are escaped
- Escape sequence: ESC (0xAB) followed by `(byte & !0x88)`
- Decode: `escaped_byte | 0x88`

### Cross-Compilation Setup

**Target**: aarch64-unknown-linux-gnu (Raspberry Pi 64-bit)

**Requirements:**
1. Install target: `rustup target add aarch64-unknown-linux-gnu`
2. Install ARM toolchain: `sudo apt install gcc-aarch64-linux-gnu`
3. Configure linker in `.cargo/config.toml`

**Build Commands:**
```bash
# Debug build
cargo build --target=aarch64-unknown-linux-gnu --example basic_connection

# Release build
cargo build --target=aarch64-unknown-linux-gnu --release --example basic_connection
```

**Deployment:**
```bash
# Deploy and run
./deploy.sh --example basic_connection --run

# Custom Pi host
./deploy.sh --pi-host 192.168.1.100 --example basic_connection
```

### Development Workflow

1. **Write code** on development machine
2. **Test** with unit tests: `cargo test`
3. **Build** for ARM: `cargo build --target=aarch64-unknown-linux-gnu`
4. **Deploy** to Pi: `./deploy.sh --example basic_connection`
5. **Test** on hardware via SSH

### Raspberry Pi UART Setup

**Enable UART on Pi:**
1. Edit `/boot/config.txt`:
   ```
   enable_uart=1
   dtoverlay=disable-bt
   ```
2. Reboot
3. UART available on GPIO 14 (TX) and GPIO 15 (RX)
4. Serial device: `/dev/serial0`

**Wiring:**
- Pi TX (GPIO 14) → RVR RX
- Pi RX (GPIO 15) → RVR TX
- Pi GND → RVR GND
- **WARNING**: RVR is 3.3V - do NOT connect 5V!

### Dependencies

- **tokio**: Async runtime (full features for flexibility)
- **tokio-serial**: Async serial port I/O
- **thiserror**: Error handling with derive macros
- **bytes**: Efficient byte buffer manipulation
- **tracing**: Structured logging for async code

### Testing Strategy

**Unit Tests:**
- Protocol encoding/decoding
- Checksum calculation
- Packet construction

**Integration Tests:**
- Mock serial port for command/response cycle
- Sequence number handling

**Hardware Tests:**
- Examples serve as hardware integration tests
- Verify on real RVR hardware

## Common Tasks

### Adding a New Command

1. Define device ID and command ID constants in `src/commands/mod.rs`
   ```rust
   pub const DEVICE_SYSTEM: u8 = 0x11;
   pub const CMD_GET_VERSION: u8 = 0x00;
   ```

2. Add high-level method to `RvrConnection` in `src/connection.rs`
   ```rust
   pub async fn get_version(&mut self) -> Result<String> {
       let seq = self.next_sequence();
       let packet = Packet::new_command(DEVICE_SYSTEM, CMD_GET_VERSION, seq, vec![]);
       let response = self.send_command_with_response(packet).await?;
       // Parse response payload...
       Ok(version)
   }
   ```

3. Create example in `examples/` demonstrating the command

4. Update `Cargo.toml` with new example entry

5. Test locally: `cargo test`

6. Cross-compile: `cargo build --target=aarch64-unknown-linux-gnu --example your_example`

7. Test on hardware: `./deploy.sh --example your_example --run`

### Debugging Serial Communication

Enable debug logging:
```bash
RUST_LOG=debug ./basic_connection
```

Use `stty` to verify UART settings:
```bash
stty -F /dev/serial0
```

### Troubleshooting

**"Permission denied" on /dev/serial0:**
```bash
sudo usermod -a -G dialout $USER
# Logout and login again
```

**Cross-compilation linker errors:**
```bash
sudo apt install gcc-arm-linux-gnueabihf g++-arm-linux-gnueabihf
```

**SSH connection timeout:**
- Verify Pi is on network: `ping raspberrypi.local`
- Check SSH enabled: `sudo raspi-config` → Interface Options → SSH

## Project Structure

```
sphero-rvr-rs/
├── .cargo/
│   └── config.toml          # Cross-compilation configuration
├── .gitignore               # Rust-specific ignores
├── Cargo.toml               # Project manifest and dependencies
├── claude.md                # This file
├── README.md                # User-facing documentation
├── deploy.sh                # Cross-compile and deployment script
├── src/
│   ├── lib.rs              # Library root, public API exports
│   ├── connection.rs       # Serial connection management
│   ├── protocol/
│   │   ├── mod.rs          # Protocol module root
│   │   ├── packet.rs       # Packet structure and parsing
│   │   ├── encoding.rs     # SLIP-style encoding/decoding
│   │   └── checksum.rs     # Checksum calculation
│   ├── commands/
│   │   ├── mod.rs          # Commands module root
│   │   └── builder.rs      # Command builder pattern
│   ├── response.rs         # Response types and parsing
│   └── error.rs            # Error types
└── examples/
    └── basic_connection.rs  # Example: Connect and validate communication
```

## References

- [Sphero SDK Documentation](https://sdk.sphero.com/)
- [Sphero API Documents](https://sdk.sphero.com/documentation/api-documents)
- [tokio-serial Documentation](https://docs.rs/tokio-serial)
- [Cross-Compilation Guide](https://chacin.dev/blog/cross-compiling-rust-for-the-raspberry-pi/)

## Future Enhancements

- [ ] Async response handling with background task
- [ ] Command timeout and retry logic
- [ ] Connection health monitoring (ping/keepalive)
- [ ] Sensor data streaming
- [ ] Motor control API
- [ ] Configuration file support (TOML)
- [ ] CLI tool for interactive control
- [ ] Web API for remote control (future stage)
