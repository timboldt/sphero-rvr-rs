# sphero-rvr-rs

Rust library for controlling Sphero RVR robots via UART serial communication from a Raspberry Pi 3B+ (and other platforms).

## Features

- **Synchronous, thread-safe API** that abstracts away asynchronous hardware complexity
- **Multi-threaded dispatcher** handling full-duplex UART communication in the background
- **Type-safe commands** leveraging Rust's type system for hardware domains
- **Cross-compilation support** for `aarch64` (Raspberry Pi)

## Current Status

**Phase 1: Core Domain & Protocol (In Progress)**
- Protocol layer (packet encoding, SLIP escaping, checksums)
- API vocabulary and data structures

**Phase 2: Dispatcher (Upcoming)**
- Background RX thread and MPSC channel routing
- UART connection management

**Phase 3: High-Level API (Upcoming)**
- Synchronous client wrapper
- Commands: Wake, Sleep, Set LEDs, Drive

**Phase 4: Hardware Validation (Upcoming)**
- Cross-compilation and physical hardware testing

## Quick Start

### Prerequisites

**Development Machine:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install ARM cross-compilation toolchain
rustup target add aarch64-unknown-linux-gnu
sudo apt install gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
```

**Raspberry Pi:**
1. Enable UART in `/boot/config.txt`:
   ```
   enable_uart=1
   dtoverlay=disable-bt
   ```
2. Reboot
3. Add user to dialout group:
   ```bash
   sudo usermod -a -G dialout $USER
   ```
4. Logout and login again

### Building

```bash
# Build for development machine (won't run on Pi)
cargo build --example basic_connection

# Cross-compile for Raspberry Pi
cargo build --target=aarch64-unknown-linux-gnu --example basic_connection

# Build optimized release version
cargo build --target=aarch64-unknown-linux-gnu --release --example basic_connection
```

### Deploying to Raspberry Pi

```bash
# Deploy example to Pi
./deploy.sh --example basic_connection

# Deploy and run immediately
./deploy.sh --example basic_connection --run

# Deploy to custom Pi host
./deploy.sh --pi-host 192.168.1.100 --pi-user pi --example basic_connection

# Deploy release build
./deploy.sh --release --example basic_connection
```

### Usage Example

```rust
use sphero_rvr::{api::SpheroRvr, transport::Dispatcher};

fn main() -> Result<(), String> {
    // 1. Open the physical serial port
    let port = serialport::new("/dev/serial0", 115_200)
        .timeout(std::time::Duration::from_millis(10))
        .open()
        .map_err(|e| e.to_string())?;

    // 2. Initialize the dispatcher and background threads
    let dispatcher = Dispatcher::new(port);
    
    // 3. Instantiate the synchronous API client
    let mut rvr = SpheroRvr::new(dispatcher);

    // 4. Send synchronous commands
    rvr.wake()?;
    rvr.set_all_leds(0, 255, 0)?; // Set LEDs green
    
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    rvr.sleep()?;

    Ok(())
}

## Hardware Setup

### Wiring RVR to Raspberry Pi

Connect the RVR's UART expansion port to the Raspberry Pi GPIO pins:

- **Pi TX (GPIO 14 / Pin 8)** → **RVR RX**
- **Pi RX (GPIO 15 / Pin 10)** → **RVR TX**
- **Pi GND (Pin 6)** → **RVR GND**

**⚠️ WARNING**: The RVR UART logic level is strictly 3.3V. Do NOT connect 5V to the RVR data lines or you will damage the board!

### Serial Port

On Raspberry Pi with UART enabled, the serial device is `/dev/serial0`.

## Running Tests

```bash
# Run unit tests
cargo test

# Run with logging
RUST_LOG=debug cargo test
```

## Development

For detailed development information, architecture details, and troubleshooting, see [claude.md](claude.md).

### Project Structure

### Project Structure

- `src/` - Library source code
  - `api/` - High-level, synchronous API
    - `client.rs` - `SpheroRvr` wrapper and domain methods
    - `types.rs` - Strongly typed commands, responses, and IDs
  - `transport/` - Hardware abstraction and concurrency
    - `dispatcher.rs` - Background TX/RX threads and MPSC routing
  - `protocol/` - Pure state machines and byte manipulation
    - `parser.rs` - `SpheroParser` state machine for incoming frames
    - `framing.rs` - SLIP-style byte escaping and unescaping
    - `checksum.rs` - Modulo 256 bitwise NOT calculations
- `examples/` - Example programs
- `deploy.sh` - Cross-compilation and deployment script
- `.cargo/config.toml` - Cross-compilation configuration

## License

MIT OR Apache-2.0

## References

- [Sphero SDK Documentation](https://sdk.sphero.com/)
- [Sphero RVR Official Docs](https://sdk.sphero.com/docs/rvr)
- [Project Development Guide](claude.md)
