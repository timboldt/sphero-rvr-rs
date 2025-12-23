use crate::error::{Result, RvrError};
use bytes::{BufMut, BytesMut};

// Protocol constants
pub const SOP: u8 = 0x8D;
pub const EOP: u8 = 0xD8;
pub const ESC: u8 = 0xAB;
pub const ESC_MASK: u8 = 0x88;

/// Encode a byte slice using SLIP-style encoding
///
/// Special bytes (ESC, SOP, EOP) are escaped:
/// - ESC -> ESC (original_byte & !ESC_MASK)
/// - Original value = escaped_value | ESC_MASK
pub fn encode_bytes(data: &[u8]) -> BytesMut {
    let mut encoded = BytesMut::with_capacity(data.len() * 2);

    for &byte in data {
        if byte == ESC || byte == SOP || byte == EOP {
            encoded.put_u8(ESC);
            encoded.put_u8(byte & !ESC_MASK);
        } else {
            encoded.put_u8(byte);
        }
    }

    encoded
}

/// Decode SLIP-style encoded bytes
pub fn decode_bytes(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoded = Vec::with_capacity(data.len());
    let mut iter = data.iter();

    while let Some(&byte) = iter.next() {
        if byte == ESC {
            let escaped = iter
                .next()
                .ok_or_else(|| RvrError::Protocol("Incomplete escape sequence".to_string()))?;
            decoded.push(*escaped | ESC_MASK);
        } else {
            decoded.push(byte);
        }
    }

    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_no_special_bytes() {
        let data = vec![0x01, 0x02, 0x03];
        let encoded = encode_bytes(&data);
        assert_eq!(encoded.as_ref(), &data[..]);
    }

    #[test]
    fn test_encode_with_escape() {
        let data = vec![0xAB]; // ESC byte
        let encoded = encode_bytes(&data);
        assert_eq!(encoded.as_ref(), &[ESC, 0xAB & !ESC_MASK]);
    }

    #[test]
    fn test_encode_with_sop() {
        let data = vec![0x8D]; // SOP byte
        let encoded = encode_bytes(&data);
        assert_eq!(encoded.as_ref(), &[ESC, 0x8D & !ESC_MASK]);
    }

    #[test]
    fn test_encode_with_eop() {
        let data = vec![0xD8]; // EOP byte
        let encoded = encode_bytes(&data);
        assert_eq!(encoded.as_ref(), &[ESC, 0xD8 & !ESC_MASK]);
    }

    #[test]
    fn test_decode_no_escape() {
        let data = vec![0x01, 0x02, 0x03];
        let decoded = decode_bytes(&data).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_decode_with_escape() {
        let data = vec![ESC, 0xAB & !ESC_MASK];
        let decoded = decode_bytes(&data).unwrap();
        assert_eq!(decoded, vec![0xAB]);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = vec![0x01, 0xAB, 0x8D, 0xD8, 0x02];
        let encoded = encode_bytes(&original);
        let decoded = decode_bytes(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_incomplete_escape() {
        let data = vec![ESC]; // Incomplete escape sequence
        let result = decode_bytes(&data);
        assert!(result.is_err());
    }
}
