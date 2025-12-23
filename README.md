# sphero-rvr-rs

Rust library for controlling Sphero RVR robots via UART serial communication from a Raspberry Pi 3B+ (and other platforms).

## Features

- **Async-first design** using Tokio runtime
- **Type-safe API** leveraging Rust's type system
- **Cross-compilation support** for Raspberry Pi
- **Protocol abstraction** - clean high-level API hiding protocol details
- **Comprehensive error handling** with domain-specific error types

## Current Status: Stage 1 (Baseline Setup)

Stage 1 provides the foundational infrastructure:
- ✅ Complete project structure and build configuration
- ✅ Protocol layer (packet encoding, SLIP encoding, checksums)
- ✅ Connection management
- ✅ Cross-compilation and deployment automation
- ✅ Basic connection example

**Coming in Stage 2:**
- LED control commands
- Status information queries
- Hardware testing

**Coming in Stage 3:**
- Complete Sphero RVR API implementation
- Sensor data, motor control, etc.

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
use sphero_rvr::{RvrConnection, RvrConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure connection
    let config = RvrConfig::default();

    // Open connection to RVR
    let mut rvr = RvrConnection::open("/dev/serial0", config).await?;

    // Stage 2 will add commands like:
    // rvr.set_led_color(255, 0, 0).await?;
    // let battery = rvr.get_battery_percentage().await?;

    // Close connection
    rvr.close().await?;
    Ok(())
}
```

## Hardware Setup

### Wiring RVR to Raspberry Pi

Connect the RVR's UART to the Raspberry Pi GPIO pins:

- **Pi TX (GPIO 14)** → **RVR RX**
- **Pi RX (GPIO 15)** → **RVR TX**
- **Pi GND** → **RVR GND**

**⚠️ WARNING**: The RVR operates at 3.3V. Do NOT connect 5V to the RVR or you may damage it!

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

- `src/` - Library source code
  - `protocol/` - Low-level packet handling
  - `commands/` - Command builders
  - `connection.rs` - Serial connection management
  - `error.rs` - Error types
- `examples/` - Example programs
- `deploy.sh` - Cross-compilation and deployment script
- `.cargo/config.toml` - Cross-compilation configuration

## License

MIT OR Apache-2.0

## References

- [Sphero SDK Documentation](https://sdk.sphero.com/)
- [Sphero RVR Official Docs](https://sdk.sphero.com/docs/rvr)
- [Project Development Guide](claude.md)
