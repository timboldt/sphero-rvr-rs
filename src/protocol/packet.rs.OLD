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

    // Stage 2 will add:
    // - to_bytes() for serialization
    // - from_bytes() for deserialization
    // - Validation methods
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
}
