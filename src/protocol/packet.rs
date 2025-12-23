/// Packet flags for command/response classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketFlags {
    pub is_response: bool,
    pub requests_response: bool,
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
            byte |= 0b0000_0001;
        }
        if self.requests_response {
            byte |= 0b0000_0010;
        }
        if self.is_activity {
            byte |= 0b0000_0100;
        }
        if self.has_target_id {
            byte |= 0b0000_1000;
        }
        if self.has_source_id {
            byte |= 0b0001_0000;
        }
        byte |= (self.reserved & 0b111) << 5;
        byte
    }

    /// Create flags from a byte
    pub fn from_byte(byte: u8) -> Self {
        Self {
            is_response: byte & 0b0000_0001 != 0,
            requests_response: byte & 0b0000_0010 != 0,
            is_activity: byte & 0b0000_0100 != 0,
            has_target_id: byte & 0b0000_1000 != 0,
            has_source_id: byte & 0b0001_0000 != 0,
            reserved: (byte >> 5) & 0b111,
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

    /// Serialize packet to bytes for transmission
    /// Format: [FLAGS] [TARGET_ID?] [SOURCE_ID?] [DEVICE_ID] [COMMAND_ID] [SEQ] [PAYLOAD...]
    /// Note: SOP, checksum, and EOP are added by the encoding layer
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Flags byte
        bytes.push(self.flags.to_byte());

        // Optional target ID
        if self.flags.has_target_id {
            bytes.push(self.target_id.unwrap_or(0x01)); // Default to 1 for Nordic target
        }

        // Optional source ID
        if self.flags.has_source_id {
            bytes.push(self.source_id.unwrap_or(0x01));
        }

        // Core fields
        bytes.push(self.device_id);
        bytes.push(self.command_id);
        bytes.push(self.sequence_number);

        // Payload
        bytes.extend_from_slice(&self.payload);

        bytes
    }

    /// Deserialize packet from bytes
    /// Expects bytes after SOP and before checksum/EOP
    pub fn from_bytes(bytes: &[u8]) -> crate::error::Result<Self> {
        if bytes.len() < 4 {
            return Err(crate::error::RvrError::Protocol(
                "Packet too short".to_string()
            ));
        }

        let mut pos = 0;
        let flags = PacketFlags::from_byte(bytes[pos]);
        pos += 1;

        let target_id = if flags.has_target_id {
            let id = bytes.get(pos).copied();
            pos += 1;
            id
        } else {
            None
        };

        let source_id = if flags.has_source_id {
            let id = bytes.get(pos).copied();
            pos += 1;
            id
        } else {
            None
        };

        if bytes.len() < pos + 3 {
            return Err(crate::error::RvrError::Protocol(
                "Packet too short for required fields".to_string()
            ));
        }

        let device_id = bytes[pos];
        pos += 1;
        let command_id = bytes[pos];
        pos += 1;
        let sequence_number = bytes[pos];
        pos += 1;

        let payload = bytes[pos..].to_vec();

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
            is_activity: true,
            has_target_id: false,
            has_source_id: false,
            reserved: 0,
        };
        let byte = flags.to_byte();
        assert_eq!(byte, 0b0000_0101); // bits 0 and 2 set
    }

    #[test]
    fn test_packet_flags_from_byte() {
        let byte = 0b0000_0110; // bits 1 and 2 set
        let flags = PacketFlags::from_byte(byte);
        assert!(!flags.is_response);
        assert!(flags.requests_response);
        assert!(flags.is_activity);
        assert!(!flags.has_target_id);
        assert!(!flags.has_source_id);
    }

    #[test]
    fn test_packet_flags_roundtrip() {
        let original = PacketFlags {
            is_response: true,
            requests_response: true,
            is_activity: false,
            has_target_id: true,
            has_source_id: false,
            reserved: 0b101,
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
    fn test_packet_serialization() {
        let packet = Packet::new_command(0x1A, 0x1C, 5, vec![0x01, 0x02, 0x03]);
        let bytes = packet.to_bytes();

        // FLAGS | DEVICE | COMMAND | SEQ | PAYLOAD...
        assert_eq!(bytes[0], packet.flags.to_byte());
        assert_eq!(bytes[1], 0x1A); // device_id
        assert_eq!(bytes[2], 0x1C); // command_id
        assert_eq!(bytes[3], 5); // sequence
        assert_eq!(&bytes[4..], &[0x01, 0x02, 0x03]); // payload
    }

    #[test]
    fn test_packet_deserialization() {
        let bytes = vec![
            0b0000_0010, // flags: requests_response=true
            0x1A,        // device_id
            0x1C,        // command_id
            5,           // sequence
            0x01,
            0x02,
            0x03, // payload
        ];

        let packet = Packet::from_bytes(&bytes).unwrap();
        assert_eq!(packet.device_id, 0x1A);
        assert_eq!(packet.command_id, 0x1C);
        assert_eq!(packet.sequence_number, 5);
        assert_eq!(packet.payload, vec![0x01, 0x02, 0x03]);
        assert!(packet.flags.requests_response);
    }

    #[test]
    fn test_packet_roundtrip() {
        let original = Packet::new_command(0x13, 0x0D, 42, vec![0xAA, 0xBB, 0xCC]);
        let bytes = original.to_bytes();
        let recovered = Packet::from_bytes(&bytes).unwrap();

        assert_eq!(recovered.device_id, original.device_id);
        assert_eq!(recovered.command_id, original.command_id);
        assert_eq!(recovered.sequence_number, original.sequence_number);
        assert_eq!(recovered.payload, original.payload);
    }
}
