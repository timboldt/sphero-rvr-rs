//! Sphero RVR Hardware Abstraction Layer for Rust
//!
//! This library provides a synchronous, thread-based interface to control
//! Sphero RVR robots via UART serial communication on Linux (Raspberry Pi).
//!
//! # Architecture
//!
//! The library is organized into three layers:
//!
//! - **API Layer** (`api`): High-level, synchronous interface with strongly
//!   typed commands and responses. Zero knowledge of transport or protocol details.
//!
//! - **Transport Layer** (`transport`): Manages serial port connection,
//!   sequence tracking, and routes messages between API and protocol layers
//!   using background threads and channels.
//!
//! - **Protocol Layer** (`protocol`): Pure state machines for packet parsing,
//!   SLIP-style byte framing, and checksum calculation.
//!
//! # Examples
//!
//! ```no_run
//! // To be implemented in Phase 3
//! // use sphero_rvr::SpheroRvr;
//! //
//! // fn main() -> Result<(), Box<dyn std::error::Error>> {
//! //     let mut rvr = SpheroRvr::connect("/dev/serial0")?;
//! //     rvr.wake()?;
//! //     rvr.set_all_leds(0, 255, 0)?; // Green
//! //     rvr.sleep()?;
//! //     Ok(())
//! // }
//! ```

// Allow unused code during development phases
#![allow(dead_code)]
#![allow(unused_imports)]

// Module declarations
pub mod api;
pub mod error;
pub mod protocol;
pub mod transport;

// Public API exports
pub use error::{Result, RvrError};

// High-level client will be exported from api module once implemented
// pub use api::SpheroRvr;
