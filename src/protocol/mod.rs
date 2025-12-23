//! Sphero RVR protocol implementation
//!
//! Based on Sphero API specification:
//! - SLIP-style encoding with escape sequences
//! - Start of Packet (SOP) and End of Packet (EOP) markers
//! - Checksum validation
//! - Big-endian multi-byte values

pub mod checksum;
pub mod encoding;
pub mod packet;

// Re-export commonly used items
pub use checksum::{calculate_checksum, verify_checksum};
pub use encoding::{decode_bytes, encode_bytes, EOP, ESC, ESC_MASK, SOP};
pub use packet::{Packet, PacketFlags};
