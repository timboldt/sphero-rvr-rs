//! Sphero RVR control library for Rust
//!
//! This library provides an async interface to control Sphero RVR robots
//! via UART serial communication on Raspberry Pi (and other platforms).
//!
//! # Examples
//!
//! ```no_run
//! use sphero_rvr::{RvrConnection, RvrConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = RvrConfig::default();
//!     let mut rvr = RvrConnection::open("/dev/serial0", config).await?;
//!
//!     // Future: Send commands
//!     // rvr.set_led_color(255, 0, 0).await?;
//!
//!     rvr.close().await?;
//!     Ok(())
//! }
//! ```

// Stage 1: Allow unused code warnings - infrastructure will be used in Stage 2
#![allow(dead_code)]
#![allow(unused_imports)]

// Module declarations
mod commands;
mod connection;
mod error;
mod protocol;
mod response;

// Public API exports
pub use connection::{RvrConfig, RvrConnection};
pub use error::{Result, RvrError};

// Re-export commonly used types from sub-modules
// (Will expand in Stage 2/3)
