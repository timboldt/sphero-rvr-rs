# Implementation Plan: Sphero RVR Rust HAL

## Objective
Implement a Hardware Abstraction Layer (HAL) in Rust for the Sphero RVR over a UART serial connection. The architecture must provide a synchronous, thread-safe high-level API to the caller while handling full-duplex, asynchronous serial communication and byte-level protocol framing on a background thread.

## Architectural Constraints
1.  **Concurrency:** Strictly use `std::thread` and `std::sync::mpsc` (or `crossbeam-channel`). Do not introduce `tokio` or other async runtimes. Keep the high-level API functions synchronous and blocking.
2.  **Separation of Concerns:** * `api`: Semantic commands and data structures. Zero transport logic.
    * `dispatcher`: Thread management, channel routing, and transport locking.
    * `protocol`: Pure state machines and functions for byte escaping, unescaping, and checksum calculations.
3.  **Platform:** Target Linux. Assume standard TTY serial devices (e.g., `/dev/ttyS0` or `/dev/ttyUSB0`).

## Dependencies
* `serialport`: For serial communication.
* `bitflags`: For sensor bitmasks.
* `log` / `env_logger`: For trace logging of raw hex frames and debug routing.

---

## Phase 1: Core Domain and Protocol Definitions
**Goal:** Define the vocabulary of the API and the byte-level framing rules.

1.  **Create `src/api/types.rs`:**
    * Define Enums for `DeviceId` (e.g., Power, Drive, Sensor, UserIo).
    * Define Enums/Structs for specific commands (e.g., `SetAllLeds`, `Wake`, `DriveWithHeading`).
    * Define `Command` struct (target_node, source_node, device_id, command_id, payload).
    * Define `Response` enum (`Ack`, `Data(Vec<u8>)`, `Error(u8)`).
2.  **Create `src/protocol/framing.rs`:**
    * Implement Sphero byte-escaping rules (escape `0xAB`, `0x8D`, `0x8E` using the `0xAB` escape byte).
    * Implement unescaping logic for incoming bytes.
    * Implement the modulo-256 bitwise NOT checksum algorithm.
3.  **Create `src/protocol/parser.rs`:**
    * Implement `SpheroParser`, a state machine that consumes raw bytes (`feed(&[u8])`) and yields complete frames.
    * Must handle SOP (Start of Packet), extracting sequence IDs, and EOP (End of Packet).

## Phase 2: The Dispatcher (HAL)
**Goal:** Implement the thread-safe bridge between the API and the UART hardware.

1.  **Create `src/transport/dispatcher.rs`:**
    * Define `Dispatcher<W: Write + Send + 'static>`.
    * Implement a `HashMap<u8, Sender<Response>>` protected by `Arc<Mutex<>>` to map outbound Sequence IDs to pending callers.
    * Implement the `send(Command) -> Result<Response, Error>` function:
        * Acquires the next Sequence ID.
        * Creates an MPSC oneshot channel.
        * Registers the sender in the HashMap.
        * Serializes the `Command` into escaped bytes using `framing.rs`.
        * Locks the TX serial port and writes the bytes.
        * Blocks on the receiver to await the `Response`.
2.  **Implement the RX Deserialization Thread:**
    * Implement `Dispatcher::spawn_rx_thread(rx_port: impl Read, pending_requests, sensor_tx, event_tx)`.
    * Continuously block on `rx_port.read()`.
    * Feed bytes to `SpheroParser`.
    * For yielded packets: route `Ack` and command responses to the `pending_requests` HashMap using the sequence ID. Route unsolicted data to `sensor_tx` or `event_tx`.

## Phase 3: High-Level API Implementation
**Goal:** Expose the ergonomic, semantic interface.

1.  **Create `src/api/client.rs`:**
    * Define `SpheroRvr` containing the `Dispatcher`.
    * Implement initialization routine (open serial port, spawn RX thread, instantiate `SpheroRvr`).
2.  **Implement Subsystem Traits/Methods:**
    * `wake(&mut self)`
    * `sleep(&mut self)`
    * `set_all_leds(&mut self, r: u8, g: u8, b: u8)`
    * `drive_with_heading(&mut self, speed: i8, heading: u16)`
    * For each method, construct the specific `Command` and pass it to `self.dispatcher.send(cmd)`.
    * Handle the resulting `Response`, converting wire errors into standard Rust `Result` types.

## Phase 4: Hardware Testing and Validation
**Goal:** Verify communication against the physical hardware.

1.  **Write `src/main.rs`:**
    * Accept a serial port path via CLI argument (e.g., `/dev/ttyS0`).
    * Instantiate the `SpheroRvr`.
    * Execute a "Hello World" sequence: Wake up -> Set LEDs to Green -> Sleep.
2.  **Agent Verification Checklist:**
    * Ensure baud rate is set to 115200.
    * Ensure parity is None, 8 data bits, 1 stop bit.
    * Verify Linux user has `dialout` or equivalent permissions for the TTY device.