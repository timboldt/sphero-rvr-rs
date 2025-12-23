use crate::error::{Result, RvrError};
use crate::protocol::packet::Packet;

/// Represents a response from the RVR
#[derive(Debug, Clone)]
pub struct Response {
    pub sequence_number: u8,
    pub error_code: u8,
    pub payload: Vec<u8>,
}

impl Response {
    /// Parse a response packet
    pub fn from_packet(packet: Packet) -> Result<Self> {
        if !packet.flags.is_response {
            return Err(RvrError::InvalidResponse(
                "Packet is not a response".to_string(),
            ));
        }

        // Stage 2 will add full parsing logic
        // For now, basic structure
        Ok(Self {
            sequence_number: packet.sequence_number,
            error_code: 0, // TODO: Extract from payload
            payload: packet.payload,
        })
    }

    /// Check if response indicates success
    pub fn is_success(&self) -> bool {
        self.error_code == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::packet::{Packet, PacketFlags};

    #[test]
    fn test_response_from_packet() {
        let packet = Packet {
            flags: PacketFlags {
                is_response: true,
                requests_response: false,
                is_activity: false,
                has_target_id: false,
                has_source_id: false,
                reserved: 0,
            },
            target_id: None,
            source_id: None,
            device_id: 0x10,
            command_id: 0x20,
            sequence_number: 5,
            payload: vec![0x00, 0x01, 0x02],
        };

        let response = Response::from_packet(packet).unwrap();
        assert_eq!(response.sequence_number, 5);
        assert_eq!(response.payload, vec![0x00, 0x01, 0x02]);
    }

    #[test]
    fn test_response_from_non_response_packet() {
        let packet = Packet::new_command(0x10, 0x20, 5, vec![]);
        let result = Response::from_packet(packet);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_success() {
        let response = Response {
            sequence_number: 1,
            error_code: 0,
            payload: vec![],
        };
        assert!(response.is_success());

        let error_response = Response {
            sequence_number: 1,
            error_code: 1,
            payload: vec![],
        };
        assert!(!error_response.is_success());
    }
}
