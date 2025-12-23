/// Calculate checksum for Sphero packet
///
/// Checksum is calculated as: 0xFF - (sum of all bytes & 0xFF)
/// Applied to all bytes except SOP and EOP markers
pub fn calculate_checksum(data: &[u8]) -> u8 {
    let sum: u16 = data.iter().map(|&b| b as u16).sum();
    0xFF - (sum & 0xFF) as u8
}

/// Verify checksum matches expected value
pub fn verify_checksum(data: &[u8], expected: u8) -> bool {
    calculate_checksum(data) == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        // Example from protocol documentation
        let data = vec![0x01, 0x02, 0x03];
        let checksum = calculate_checksum(&data);
        assert_eq!(checksum, 0xFF - 6);
    }

    #[test]
    fn test_checksum_verification() {
        let data = vec![0x10, 0x20, 0x30];
        let checksum = calculate_checksum(&data);
        assert!(verify_checksum(&data, checksum));
        assert!(!verify_checksum(&data, checksum + 1));
    }
}
