# Sphero RVR Rust Library - Development Guide

## Project Overview

This is a Rust Hardware Abstraction Layer (HAL) for controlling Sphero RVR robots via UART serial communication. It is designed for Linux environments (specifically Raspberry Pi) and prioritizes a clean, synchronous high-level API backed by a robust, multi-threaded hardware dispatcher.

### Architecture

The library explicitly avoids heavy async runtimes to maintain predictable execution and clean API boundaries. It uses standard library threads and channels to bridge the full-duplex asynchronous serial line to a synchronous application interface.

1. **API Layer** (`src/api/`): High-level, synchronous interface
   - Strongly typed structs and enums for domains (Drive, Power, UserIo).
   - Zero knowledge of transport or byte-level framing.
   - Blocks on oneshot channels waiting for specific Ack sequences.

2. **Dispatcher Layer** (`src/transport/`): Concurrency and routing
   - Owns the physical serial port (`serialport` crate).
   - Manages sequence IDs and a `HashMap` of pending requests.
   - Runs a background RX thread to constantly consume the UART buffer.
   - Routes incoming Acks to waiting callers and pushes asynchronous events/sensors to dedicated MPSC channels.

3. **Protocol Layer** (`src/protocol/`): Pure state machines
   - `SpheroParser`: Feeds on raw bytes, handles SOP/EOP detection.
   - Implements SLIP-style encoding/decoding for special characters.
   - Handles checksum calculation and verification.

### Implementation Phases

**Phase 1**: Core Domain and Protocol Definitions
- Define API vocabulary (Device IDs, Command IDs).
- Implement byte-escaping and checksum rules.
- Build the `SpheroParser` state machine.

**Phase 2**: The Dispatcher (HAL)
- Implement `Dispatcher` thread management and sequence tracking.
- Build the RX deserialization loop.

**Phase 3**: High-Level API Implementation
- Implement the `SpheroRvr` client wrapper.
- Add baseline commands: Wake, Sleep, Set LEDs.

**Phase 4**: Hardware Validation
- Cross-compile for aarch64.
- Validate the "Hello World" (Wake -> Green LEDs) on real hardware.

## Technical Details

### Sphero RVR Protocol

**Packet Structure:**
`[SOP] [FLAGS] [TARGET_ID] [SOURCE_ID] [DEVICE_ID] [COMMAND_ID] [SEQ] [PAYLOAD...] [CHECKSUM] [EOP]`

**Key Specifications:**
- Baud rate: 115200
- Parity: None, 8 Data bits, 1 Stop bit.
- Voltage: 3.3V (NOT 5V tolerant!)
- Checksum: Modulo 256 bitwise NOT (`!sum(bytes) & 0xFF`)

**SLIP Encoding:**
- The protocol escapes specific byte values within the payload to ensure they aren't confused with framing bytes.
- Special bytes (0xAB, 0x8D, 0x8E) are escaped.
- Escape sequence: ESC (0xAB) followed by `(byte & !0x88)`.
- Decode: `escaped_byte | 0x88`.

### Cross-Compilation Setup

**Target**: `aarch64-unknown-linux-gnu` (Raspberry Pi 64-bit)

**Requirements:**
1. Install target: `rustup target add aarch64-unknown-linux-gnu`
2. Install ARM toolchain: `sudo apt install gcc-aarch64-linux-gnu`
3. If using `serialport`, you may need `pkg-config` and `libudev-dev` configured for the target architecture.

**Build Commands:**
`cargo build --target=aarch64-unknown-linux-gnu --release --example basic_connection`

### Raspberry Pi UART Setup

**Enable UART on Pi:**
1. Edit `/boot/config.txt`:
   ```text
   enable_uart=1
   dtoverlay=disable-bt
   ```
2. Reboot. UART is available on GPIO 14 (TX) and GPIO 15 (RX).
3. Serial device is typically `/dev/serial0` or `/dev/ttyS0`.

**Wiring:**
- Pi TX (GPIO 14) → RVR RX
- Pi RX (GPIO 15) → RVR TX
- Pi GND → RVR GND

### Dependencies

- **serialport**: Blocking serial port access.
- **bitflags**: Ergonomic bitmasking for sensor selection.
- **crossbeam-channel** (Optional): If you prefer it over `std::sync::mpsc` for the MPMC sensor streaming.
- **log** / **env_logger**: For hex-dumping frames during protocol debugging.

## Project Structure

```text
sphero-rvr-rs/
├── .cargo/
│   └── config.toml          
├── Cargo.toml               
├── DEVELOPMENT.md           # This file
├── src/
│   ├── lib.rs              
│   ├── api/
│   │   ├── mod.rs          
│   │   ├── client.rs        # SpheroRvr high-level wrapper
│   │   └── types.rs         # Commands, Responses, DeviceIds
│   ├── transport/
│   │   ├── mod.rs          
│   │   └── dispatcher.rs    # Thread management, TX/RX loops, MPSC routing
│   └── protocol/
│   │   ├── mod.rs          
│   │   ├── parser.rs        # SpheroParser state machine
│   │   ├── framing.rs       # SLIP-style escaping
│   │   └── checksum.rs      
└── examples/
    └── hello_rvr.rs         # Wake, set LEDs, sleep
```