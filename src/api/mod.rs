//! High-level synchronous API for Sphero RVR
//!
//! This module provides a clean, strongly-typed interface for controlling
//! the Sphero RVR robot. All operations are synchronous and blocking.
//!
//! The API layer has zero knowledge of transport details or byte-level
//! protocol framing. It works purely with high-level commands and responses.
//!
//! # Example
//!
//! ```no_run
//! use sphero_rvr::SpheroRvr;
//! use sphero_rvr::api::types::Color;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut rvr = SpheroRvr::connect("/dev/serial0")?;
//!
//! rvr.wake()?;
//! rvr.set_all_leds(Color::GREEN)?;
//! rvr.sleep()?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod constants;
pub mod types;

// Re-export main types
pub use client::SpheroRvr;
pub use types::{BatteryState, Color, FirmwareVersion};
