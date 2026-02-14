use crate::error::{Result, RvrError};
use crate::protocol::checksum::calculate_checksum;

/// Packet flags for command/response classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketFlags {
    pub is_response: bool,
    pub requests_response: bool,
    pub requests_only_error_response: bool,
    pub is_activity: bool,
    pub has_target_id: bool,
    pub has_source_id: bool,
    pub reserved: u8,
}

impl PacketFlags {
    /// Convert flags to a byte
    pub fn to_byte(self) -> u8 {
        let mut byte = 0u8;
        if self.is_response {
            byte |= 0b0000_0001; // bit 0
        }
        if self.requests_response {
            byte |= 0b0000_0010; // bit 1
        }
        if self.requests_only_error_response {
            byte |= 0b0000_0100; // bit 2
        }
        if self.is_activity {
            byte |= 0b0000_1000; // bit 3
        }
        if self.has_target_id {
            byte |= 0b0001_0000; // bit 4
        }
        if self.has_source_id {
            byte |= 0b0010_0000; // bit 5
        }
        byte |= (self.reserved & 0b11) << 6; // bits 6-7
        byte
    }

    /// Create flags from a byte
    pub fn from_byte(byte: u8) -> Self {
        Self {
            is_response: byte & 0b0000_0001 != 0,                 // bit 0
            requests_response: byte & 0b0000_0010 != 0,           // bit 1
            requests_only_error_response: byte & 0b0000_0100 != 0, // bit 2
            is_activity: byte & 0b0000_1000 != 0,                 // bit 3
            has_target_id: byte & 0b0001_0000 != 0,               // bit 4
            has_source_id: byte & 0b0010_0000 != 0,               // bit 5
            reserved: (byte >> 6) & 0b11,                         // bits 6-7
        }
    }
}

/// Represents a Sphero API packet
#[derive(Debug, Clone)]
pub struct Packet {
    pub flags: PacketFlags,
    pub target_id: Option<u8>,
    pub source_id: Option<u8>,
    pub device_id: u8,
    pub command_id: u8,
    pub sequence_number: u8,
    pub payload: Vec<u8>,
}

impl Packet {
    /// Create a new command packet
    pub fn new_command(
        device_id: u8,
        command_id: u8,
        sequence_number: u8,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            flags: PacketFlags {
                is_response: false,
                requests_response: true,
                requests_only_error_response: false,
                is_activity: false,
                has_target_id: false,
                has_source_id: false,
                reserved: 0,
            },
            target_id: None,
            source_id: None,
            device_id,
            command_id,
            sequence_number,
            payload,
        }
    }

    /// Serialize packet to raw bytes (before SLIP encoding and framing)
    ///
    /// Returns: [FLAGS] [TARGET_ID?] [SOURCE_ID?] [DEVICE_ID] [COMMAND_ID] [SEQ] [PAYLOAD...] [CHECKSUM]
    /// Note: Does NOT include SOP/EOP markers or apply SLIP encoding
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // FLAGS byte
        bytes.push(self.flags.to_byte());

        // Optional TARGET_ID
        if let Some(target_id) = self.target_id {
            bytes.push(target_id);
        }

        // Optional SOURCE_ID
        if let Some(source_id) = self.source_id {
            bytes.push(source_id);
        }

        // Fixed fields
        bytes.push(self.device_id);
        bytes.push(self.command_id);
        bytes.push(self.sequence_number);

        // Payload
        bytes.extend_from_slice(&self.payload);

        // Checksum (calculated on all bytes so far)
        let checksum = calculate_checksum(&bytes);
        bytes.push(checksum);

        bytes
    }

    /// Parse packet from unescaped buffer (after SLIP decoding, without SOP/EOP)
    ///
    /// Expected format: [FLAGS] [TARGET_ID?] [SOURCE_ID?] [DEVICE_ID] [COMMAND_ID] [SEQ] [PAYLOAD...] [CHECKSUM]
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // Minimum packet: FLAGS + DEVICE_ID + COMMAND_ID + SEQ + CHECKSUM = 5 bytes
        if data.len() < 5 {
            return Err(RvrError::Protocol(format!(
                "Packet too short: {} bytes (minimum 5)",
                data.len()
            )));
        }

        let mut idx = 0;

        // Parse FLAGS
        let flags = PacketFlags::from_byte(data[idx]);
        idx += 1;

        // Parse optional TARGET_ID
        let target_id = if flags.has_target_id {
            if idx >= data.len() {
                return Err(RvrError::Protocol(
                    "Packet truncated: expected target_id".to_string(),
                ));
            }
            let id = data[idx];
            idx += 1;
            Some(id)
        } else {
            None
        };

        // Parse optional SOURCE_ID
        let source_id = if flags.has_source_id {
            if idx >= data.len() {
                return Err(RvrError::Protocol(
                    "Packet truncated: expected source_id".to_string(),
                ));
            }
            let id = data[idx];
            idx += 1;
            Some(id)
        } else {
            None
        };

        // Parse fixed fields (need at least 4 more bytes: DEVICE_ID, COMMAND_ID, SEQ, CHECKSUM)
        if idx + 4 > data.len() {
            return Err(RvrError::Protocol(format!(
                "Packet truncated: expected at least {} bytes, got {}",
                idx + 4,
                data.len()
            )));
        }

        let device_id = data[idx];
        idx += 1;

        let command_id = data[idx];
        idx += 1;

        let sequence_number = data[idx];
        idx += 1;

        // Extract checksum (last byte)
        let checksum = data[data.len() - 1];

        // Extract payload (everything between SEQ and CHECKSUM)
        let payload = data[idx..data.len() - 1].to_vec();

        // Verify checksum (calculated on all bytes except the checksum itself)
        let expected_checksum = calculate_checksum(&data[..data.len() - 1]);
        if checksum != expected_checksum {
            return Err(RvrError::Checksum {
                expected: expected_checksum,
                actual: checksum,
            });
        }

        Ok(Self {
            flags,
            target_id,
            source_id,
            device_id,
            command_id,
            sequence_number,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_flags_to_byte() {
        let flags = PacketFlags {
            is_response: true,
            requests_response: false,
            requests_only_error_response: false,
            is_activity: true,
            has_target_id: false,
            has_source_id: false,
            reserved: 0,
        };
        let byte = flags.to_byte();
        assert_eq!(byte, 0b0000_1001); // bits 0 and 3 set
    }

    #[test]
    fn test_packet_flags_from_byte() {
        let byte = 0b0000_0110; // bits 1 and 2 set
        let flags = PacketFlags::from_byte(byte);
        assert!(!flags.is_response);
        assert!(flags.requests_response);
        assert!(flags.requests_only_error_response);
        assert!(!flags.is_activity);
        assert!(!flags.has_target_id);
        assert!(!flags.has_source_id);
    }

    #[test]
    fn test_packet_flags_roundtrip() {
        let original = PacketFlags {
            is_response: true,
            requests_response: true,
            requests_only_error_response: false,
            is_activity: false,
            has_target_id: true,
            has_source_id: false,
            reserved: 0b01, // Only 2 bits available now (bits 6-7)
        };
        let byte = original.to_byte();
        let recovered = PacketFlags::from_byte(byte);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_new_command_packet() {
        let packet = Packet::new_command(0x10, 0x20, 5, vec![0x01, 0x02]);
        assert_eq!(packet.device_id, 0x10);
        assert_eq!(packet.command_id, 0x20);
        assert_eq!(packet.sequence_number, 5);
        assert_eq!(packet.payload, vec![0x01, 0x02]);
        assert!(!packet.flags.is_response);
        assert!(packet.flags.requests_response);
        assert!(packet.target_id.is_none());
        assert!(packet.source_id.is_none());
    }

    #[test]
    fn test_packet_to_bytes_simple() {
        let packet = Packet::new_command(0x10, 0x20, 5, vec![]);
        let bytes = packet.to_bytes();

        // Expected: [FLAGS] [DEVICE_ID] [COMMAND_ID] [SEQ] [CHECKSUM]
        assert_eq!(bytes.len(), 5);
        assert_eq!(bytes[0], 0b0000_0010); // requests_response flag
        assert_eq!(bytes[1], 0x10); // device_id
        assert_eq!(bytes[2], 0x20); // command_id
        assert_eq!(bytes[3], 5); // sequence_number

        // Verify checksum
        let expected_checksum = calculate_checksum(&bytes[..4]);
        assert_eq!(bytes[4], expected_checksum);
    }

    #[test]
    fn test_packet_to_bytes_with_payload() {
        let packet = Packet::new_command(0x10, 0x20, 5, vec![0xAA, 0xBB]);
        let bytes = packet.to_bytes();

        // Expected: [FLAGS] [DEVICE_ID] [COMMAND_ID] [SEQ] [PAYLOAD...] [CHECKSUM]
        assert_eq!(bytes.len(), 7);
        assert_eq!(bytes[4], 0xAA);
        assert_eq!(bytes[5], 0xBB);
    }

    #[test]
    fn test_packet_from_bytes_simple() {
        // Create a simple packet
        let original = Packet::new_command(0x10, 0x20, 5, vec![]);
        let bytes = original.to_bytes();

        // Parse it back
        let parsed = Packet::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.device_id, 0x10);
        assert_eq!(parsed.command_id, 0x20);
        assert_eq!(parsed.sequence_number, 5);
        assert!(parsed.payload.is_empty());
        assert!(parsed.flags.requests_response);
    }

    #[test]
    fn test_packet_roundtrip_with_payload() {
        let original = Packet::new_command(0x13, 0x07, 42, vec![0x01, 0x02, 0x03, 0x04]);
        let bytes = original.to_bytes();
        let parsed = Packet::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.device_id, original.device_id);
        assert_eq!(parsed.command_id, original.command_id);
        assert_eq!(parsed.sequence_number, original.sequence_number);
        assert_eq!(parsed.payload, original.payload);
    }

    #[test]
    fn test_packet_with_optional_ids() {
        let mut packet = Packet::new_command(0x10, 0x20, 5, vec![0xAA]);
        packet.target_id = Some(0x01);
        packet.source_id = Some(0x02);
        packet.flags.has_target_id = true;
        packet.flags.has_source_id = true;

        let bytes = packet.to_bytes();
        let parsed = Packet::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.target_id, Some(0x01));
        assert_eq!(parsed.source_id, Some(0x02));
        assert_eq!(parsed.device_id, 0x10);
        assert_eq!(parsed.payload, vec![0xAA]);
    }

    #[test]
    fn test_packet_from_bytes_too_short() {
        let data = vec![0x02, 0x10, 0x20]; // Only 3 bytes
        let result = Packet::from_bytes(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_packet_from_bytes_bad_checksum() {
        let packet = Packet::new_command(0x10, 0x20, 5, vec![]);
        let mut bytes = packet.to_bytes();

        // Corrupt the checksum
        let len = bytes.len();
        bytes[len - 1] ^= 0xFF;

        let result = Packet::from_bytes(&bytes);
        assert!(matches!(result, Err(RvrError::Checksum { .. })));
    }
}
