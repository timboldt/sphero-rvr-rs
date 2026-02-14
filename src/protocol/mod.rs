//! Sphero RVR protocol implementation
//!
//! Based on Sphero API specification:
//! - SLIP-style encoding with escape sequences
//! - Start of Packet (SOP) and End of Packet (EOP) markers
//! - Checksum validation
//! - Big-endian multi-byte values
//!
//! Architecture:
//! - `checksum`: Pure checksum calculation
//! - `framing`: SLIP-style byte encoding/decoding
//! - `packet`: Packet data structures and serialization
//! - `parser`: Streaming parser state machine

pub mod checksum;
pub mod framing;
pub mod packet;
pub mod parser;

// Re-export commonly used items
pub use checksum::{calculate_checksum, verify_checksum};
pub use framing::{decode_bytes, encode_bytes, EOP, ESC, ESC_MASK, SOP};
pub use packet::{Packet, PacketFlags};
pub use parser::SpheroParser;
