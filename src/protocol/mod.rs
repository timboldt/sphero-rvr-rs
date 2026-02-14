//! Sphero RVR protocol implementation
//!
//! This module contains pure, well-tested protocol logic that survived
//! the async->sync architecture migration (2026-02-14).
//!
//! Based on Sphero API specification:
//! - SLIP-style encoding with escape sequences
//! - Start of Packet (SOP) and End of Packet (EOP) markers
//! - Checksum validation
//! - Big-endian multi-byte values
//!
//! Architecture:
//! - `checksum`: Pure checksum calculation (preserved unchanged)
//! - `framing`: SLIP-style byte encoding/decoding (refactored from encoding.rs)
//!
//! Note: packet.rs.OLD is kept as reference for implementing the new
//! SpheroParser state machine in Phase 1.

pub mod checksum;
pub mod framing;

// Re-export commonly used items
pub use checksum::{calculate_checksum, verify_checksum};
pub use framing::{decode_bytes, encode_bytes, EOP, ESC, ESC_MASK, SOP};
