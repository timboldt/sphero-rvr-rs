use crate::error::{Result, RvrError};
use crate::protocol::framing::{EOP, ESC, ESC_MASK, SOP};
use crate::protocol::packet::Packet;

/// Parser state machine for streaming UART input
#[derive(Debug)]
enum ParserState {
    /// Idle, waiting for SOP byte (0x8D)
    WaitingForSop,

    /// Inside a packet, accumulating unescaped bytes
    ReadingPacket {
        /// Unescaped buffer (does NOT include SOP/EOP framing bytes)
        buffer: Vec<u8>,
        /// True if the previous byte was ESC (0xAB)
        is_escaped: bool,
    },
}

/// Streaming parser for Sphero RVR protocol packets
///
/// This parser operates byte-by-byte on UART input streams and handles:
/// - SLIP-style escape sequence decoding
/// - Variable-length payloads
/// - Automatic resynchronization on errors
/// - Checksum verification
///
/// # Example
///
/// ```no_run
/// use sphero_rvr::protocol::parser::SpheroParser;
///
/// let mut parser = SpheroParser::new();
///
/// // Feed bytes one at a time from UART
/// # let uart_stream: Vec<u8> = vec![];
/// for byte in uart_stream {
///     match parser.feed(byte) {
///         Ok(Some(packet)) => {
///             // Complete packet received
///             println!("Received packet: {:?}", packet);
///         }
///         Ok(None) => {
///             // Still accumulating bytes
///         }
///         Err(e) => {
///             // Parser error (bad checksum, unexpected SOP, etc.)
///             // Parser is already in a valid state, safe to continue
///             eprintln!("Parser error: {}", e);
///         }
///     }
/// }
/// ```
pub struct SpheroParser {
    state: ParserState,
}

impl SpheroParser {
    /// Create a new parser in the initial state
    pub fn new() -> Self {
        Self {
            state: ParserState::WaitingForSop,
        }
    }

    /// Feed one byte into the parser
    ///
    /// Returns:
    /// - `Ok(Some(packet))` when a complete, valid packet is parsed
    /// - `Ok(None)` when still accumulating bytes
    /// - `Err(...)` on parse errors (parser automatically recovers to valid state)
    ///
    /// # Error Recovery
    ///
    /// On any error, the parser automatically resets to a valid state:
    /// - Checksum failures: Parser resets to `WaitingForSop`
    /// - Unexpected SOP mid-packet: Parser discards corrupted buffer and starts fresh
    /// - Incomplete escape sequences: Reported as error, parser resets
    ///
    /// The caller should log errors and continue reading bytes.
    pub fn feed(&mut self, byte: u8) -> Result<Option<Packet>> {
        match &mut self.state {
            ParserState::WaitingForSop => {
                if byte == SOP {
                    // Start of new packet detected
                    self.state = ParserState::ReadingPacket {
                        buffer: Vec::with_capacity(32), // Pre-allocate for typical packet size
                        is_escaped: false,
                    };
                }
                // Ignore all other bytes while waiting for SOP
                Ok(None)
            }

            ParserState::ReadingPacket {
                ref mut buffer,
                ref mut is_escaped,
            } => {
                if *is_escaped {
                    // Previous byte was ESC, we expect an escaped data byte
                    // Invalid: EOP, SOP, or ESC appearing directly after ESC
                    // (they should be escaped to 0x50, 0x05, 0x23 respectively)
                    if byte == EOP || byte == SOP || byte == ESC {
                        self.state = ParserState::WaitingForSop;
                        return Err(RvrError::Protocol("Invalid escape sequence".to_string()));
                    }
                    // Valid escaped byte: unescape it
                    // SLIP decoding: escaped_byte | ESC_MASK restores original value
                    buffer.push(byte | ESC_MASK);
                    *is_escaped = false;
                    Ok(None)
                } else if byte == ESC {
                    // Next byte needs unescaping
                    *is_escaped = true;
                    Ok(None)
                } else if byte == SOP {
                    // RESYNC: Unexpected SOP mid-packet means we missed the EOP
                    // Discard corrupted buffer and start fresh packet
                    self.state = ParserState::ReadingPacket {
                        buffer: Vec::with_capacity(32),
                        is_escaped: false,
                    };
                    Err(RvrError::Protocol(
                        "Unexpected SOP mid-packet, resyncing".to_string(),
                    ))
                } else if byte == EOP {
                    // End of packet detected

                    // Check for incomplete escape sequence FIRST (before moving buffer)
                    let was_escaped = *is_escaped;

                    // CRITICAL: Extract buffer and reset state BEFORE parsing
                    // This ensures parser is in a valid state even if parse_buffer() fails
                    let final_buffer = std::mem::take(buffer);
                    self.state = ParserState::WaitingForSop;

                    if was_escaped {
                        return Err(RvrError::Protocol(
                            "Incomplete escape sequence at EOP".to_string(),
                        ));
                    }

                    // Parse the accumulated buffer
                    match Self::parse_buffer(&final_buffer) {
                        Ok(packet) => Ok(Some(packet)),
                        Err(e) => Err(e),
                    }
                } else {
                    // Normal data byte, add to buffer
                    buffer.push(byte);
                    Ok(None)
                }
            }
        }
    }

    /// Parse an unescaped buffer into a Packet
    ///
    /// Buffer format: [FLAGS] [TARGET_ID?] [SOURCE_ID?] [DEVICE_ID] [COMMAND_ID] [SEQ] [PAYLOAD...] [CHECKSUM]
    ///
    /// This is called when EOP is received and contains all packet parsing logic.
    fn parse_buffer(buffer: &[u8]) -> Result<Packet> {
        // Delegate to Packet::from_bytes which handles all the parsing
        Packet::from_bytes(buffer)
    }

    /// Reset the parser to initial state
    ///
    /// Useful for explicit error recovery or reinitialization
    pub fn reset(&mut self) {
        self.state = ParserState::WaitingForSop;
    }
}

impl Default for SpheroParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::framing::encode_bytes;
    use crate::protocol::packet::Packet;

    /// Helper to feed a slice of bytes to the parser
    fn feed_bytes(parser: &mut SpheroParser, bytes: &[u8]) -> Result<Option<Packet>> {
        let mut result = None;
        for &byte in bytes {
            if let Some(packet) = parser.feed(byte)? {
                result = Some(packet);
            }
        }
        Ok(result)
    }

    #[test]
    fn test_parse_simple_packet() {
        let mut parser = SpheroParser::new();

        // Create a simple packet
        let packet = Packet::new_command(0x10, 0x20, 5, vec![]);
        let unescaped_bytes = packet.to_bytes();

        // Build framed packet: SOP + unescaped + EOP
        let mut framed = vec![SOP];
        framed.extend_from_slice(&unescaped_bytes);
        framed.push(EOP);

        // Feed byte-by-byte
        let parsed = feed_bytes(&mut parser, &framed).unwrap().unwrap();

        assert_eq!(parsed.device_id, 0x10);
        assert_eq!(parsed.command_id, 0x20);
        assert_eq!(parsed.sequence_number, 5);
        assert!(parsed.payload.is_empty());
    }

    #[test]
    fn test_parse_packet_with_payload() {
        let mut parser = SpheroParser::new();

        let packet = Packet::new_command(0x13, 0x07, 42, vec![0x01, 0x02, 0x03]);
        let unescaped_bytes = packet.to_bytes();

        let mut framed = vec![SOP];
        framed.extend_from_slice(&unescaped_bytes);
        framed.push(EOP);

        let parsed = feed_bytes(&mut parser, &framed).unwrap().unwrap();

        assert_eq!(parsed.device_id, 0x13);
        assert_eq!(parsed.command_id, 0x07);
        assert_eq!(parsed.sequence_number, 42);
        assert_eq!(parsed.payload, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_parse_packet_with_escaped_bytes() {
        let mut parser = SpheroParser::new();

        // Create packet with payload containing special bytes that need escaping
        let packet = Packet::new_command(0x13, 0x07, 1, vec![0x8D, 0xD8, 0xAB]); // SOP, EOP, ESC in payload
        let unescaped_bytes = packet.to_bytes();

        // Apply SLIP encoding to the unescaped packet bytes
        let escaped_bytes = encode_bytes(&unescaped_bytes);

        // Build framed packet: SOP + escaped + EOP
        let mut framed = vec![SOP];
        framed.extend_from_slice(&escaped_bytes);
        framed.push(EOP);

        // Feed to parser
        let parsed = feed_bytes(&mut parser, &framed).unwrap().unwrap();

        assert_eq!(parsed.device_id, 0x13);
        assert_eq!(parsed.command_id, 0x07);
        assert_eq!(parsed.payload, vec![0x8D, 0xD8, 0xAB]);
    }

    #[test]
    fn test_multiple_packets_back_to_back() {
        let mut parser = SpheroParser::new();

        // Create two packets
        let packet1 = Packet::new_command(0x10, 0x20, 1, vec![0xAA]);
        let packet2 = Packet::new_command(0x11, 0x21, 2, vec![0xBB]);

        // Frame both packets
        let mut stream = vec![SOP];
        stream.extend_from_slice(&packet1.to_bytes());
        stream.push(EOP);
        stream.push(SOP);
        stream.extend_from_slice(&packet2.to_bytes());
        stream.push(EOP);

        // Feed entire stream
        let mut packets = Vec::new();
        for &byte in &stream {
            if let Some(packet) = parser.feed(byte).unwrap() {
                packets.push(packet);
            }
        }

        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].sequence_number, 1);
        assert_eq!(packets[0].payload, vec![0xAA]);
        assert_eq!(packets[1].sequence_number, 2);
        assert_eq!(packets[1].payload, vec![0xBB]);
    }

    #[test]
    fn test_junk_data_before_sop() {
        let mut parser = SpheroParser::new();

        let packet = Packet::new_command(0x10, 0x20, 5, vec![]);

        // Add junk bytes before SOP
        let mut stream = vec![0xFF, 0x00, 0x12, 0x34]; // Junk
        stream.push(SOP);
        stream.extend_from_slice(&packet.to_bytes());
        stream.push(EOP);

        let parsed = feed_bytes(&mut parser, &stream).unwrap().unwrap();
        assert_eq!(parsed.device_id, 0x10);
    }

    #[test]
    fn test_unexpected_sop_mid_packet() {
        let mut parser = SpheroParser::new();

        let packet = Packet::new_command(0x10, 0x20, 5, vec![]);
        let bytes = packet.to_bytes();

        // Start packet, then send unexpected SOP mid-stream
        let mut stream = vec![SOP];
        stream.extend_from_slice(&bytes[..2]); // Partial packet
        stream.push(SOP); // Unexpected SOP (should trigger resync)
        stream.extend_from_slice(&bytes); // Complete valid packet
        stream.push(EOP);

        let mut error_count = 0;
        let mut parsed = None;

        for &byte in &stream {
            match parser.feed(byte) {
                Ok(Some(packet)) => parsed = Some(packet),
                Ok(None) => {}
                Err(_) => error_count += 1,
            }
        }

        // Should have received one error (resync) and one valid packet
        assert_eq!(error_count, 1);
        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap().device_id, 0x10);
    }

    #[test]
    fn test_bad_checksum() {
        let mut parser = SpheroParser::new();

        let packet = Packet::new_command(0x10, 0x20, 5, vec![]);
        let mut bytes = packet.to_bytes();

        // Corrupt checksum
        let len = bytes.len();
        bytes[len - 1] ^= 0xFF;

        let mut stream = vec![SOP];
        stream.extend_from_slice(&bytes);
        stream.push(EOP);

        let result = feed_bytes(&mut parser, &stream);
        assert!(matches!(result, Err(RvrError::Checksum { .. })));

        // Verify parser is still in valid state after error
        let packet2 = Packet::new_command(0x11, 0x21, 6, vec![]);
        let mut stream2 = vec![SOP];
        stream2.extend_from_slice(&packet2.to_bytes());
        stream2.push(EOP);

        let parsed = feed_bytes(&mut parser, &stream2).unwrap().unwrap();
        assert_eq!(parsed.device_id, 0x11);
    }

    #[test]
    fn test_incomplete_escape_at_eop() {
        let mut parser = SpheroParser::new();

        // Construct a malformed packet: valid start, ESC as last byte before EOP
        let stream = vec![SOP, 0x02, 0x10, 0x20, 0x05, ESC, EOP];

        let result = feed_bytes(&mut parser, &stream);
        assert!(matches!(result, Err(RvrError::Protocol(_))));
    }

    #[test]
    fn test_reset() {
        let mut parser = SpheroParser::new();

        // Start feeding a packet but don't complete it
        parser.feed(SOP).unwrap();
        parser.feed(0x02).unwrap();
        parser.feed(0x10).unwrap();

        // Manually reset
        parser.reset();

        // Should be able to parse a new packet cleanly
        let packet = Packet::new_command(0x13, 0x07, 1, vec![]);
        let mut stream = vec![SOP];
        stream.extend_from_slice(&packet.to_bytes());
        stream.push(EOP);

        let parsed = feed_bytes(&mut parser, &stream).unwrap().unwrap();
        assert_eq!(parsed.device_id, 0x13);
    }

    #[test]
    fn test_integration_full_roundtrip() {
        // This test validates the entire encode -> parse pipeline

        let mut parser = SpheroParser::new();

        // Create a packet with various challenging payload bytes
        let original = Packet::new_command(
            0x13,
            0x07,
            42,
            vec![0x00, 0x8D, 0xD8, 0xAB, 0xFF, 0x01], // Includes SOP, EOP, ESC
        );

        // Serialize to unescaped bytes
        let unescaped = original.to_bytes();

        // Apply SLIP encoding
        let escaped = encode_bytes(&unescaped);

        // Add framing
        let mut framed = vec![SOP];
        framed.extend_from_slice(&escaped);
        framed.push(EOP);

        // Parse byte-by-byte
        let parsed = feed_bytes(&mut parser, &framed).unwrap().unwrap();

        // Verify round-trip
        assert_eq!(parsed.device_id, original.device_id);
        assert_eq!(parsed.command_id, original.command_id);
        assert_eq!(parsed.sequence_number, original.sequence_number);
        assert_eq!(parsed.payload, original.payload);
    }
}
