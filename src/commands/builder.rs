use crate::protocol::packet::Packet;

/// Builder pattern for constructing commands
///
/// Stage 1: Basic structure
/// Stage 2: Will add LED, status commands
pub struct CommandBuilder {
    device_id: u8,
    command_id: u8,
    payload: Vec<u8>,
}

impl CommandBuilder {
    /// Create a new command builder
    pub fn new(device_id: u8, command_id: u8) -> Self {
        Self {
            device_id,
            command_id,
            payload: Vec::new(),
        }
    }

    /// Add a payload to the command
    pub fn with_payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = payload;
        self
    }

    /// Build the final packet with the given sequence number
    pub fn build(self, sequence_number: u8) -> Packet {
        Packet::new_command(
            self.device_id,
            self.command_id,
            sequence_number,
            self.payload,
        )
    }
}

// Stage 2 will add high-level command builders:
// - LedCommandBuilder
// - StatusQueryBuilder
// etc.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_builder() {
        let packet = CommandBuilder::new(0x10, 0x20)
            .with_payload(vec![0x01, 0x02, 0x03])
            .build(5);

        assert_eq!(packet.device_id, 0x10);
        assert_eq!(packet.command_id, 0x20);
        assert_eq!(packet.sequence_number, 5);
        assert_eq!(packet.payload, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_command_builder_no_payload() {
        let packet = CommandBuilder::new(0x15, 0x25).build(10);

        assert_eq!(packet.device_id, 0x15);
        assert_eq!(packet.command_id, 0x25);
        assert_eq!(packet.sequence_number, 10);
        assert!(packet.payload.is_empty());
    }
}
