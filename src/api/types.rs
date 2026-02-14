//! High-level types for the Sphero RVR API

/// RGB Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
}

impl Color {
    /// Create a new color from RGB components
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create a color from a hex value (e.g., 0xFF0000 for red)
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }

    /// Convert to a byte array [R, G, B]
    pub const fn to_bytes(self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    // Common colors
    pub const BLACK: Self = Self::new(0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255);
    pub const RED: Self = Self::new(255, 0, 0);
    pub const GREEN: Self = Self::new(0, 255, 0);
    pub const BLUE: Self = Self::new(0, 0, 255);
    pub const YELLOW: Self = Self::new(255, 255, 0);
    pub const CYAN: Self = Self::new(0, 255, 255);
    pub const MAGENTA: Self = Self::new(255, 0, 255);
    pub const ORANGE: Self = Self::new(255, 165, 0);
    pub const PURPLE: Self = Self::new(128, 0, 128);
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::new(r, g, b)
    }
}

impl From<[u8; 3]> for Color {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::new(r, g, b)
    }
}

/// Battery state information
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BatteryState {
    /// Battery percentage (0-100)
    pub percentage: u8,
}

/// Firmware version information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareVersion {
    /// Major version
    pub major: u8,
    /// Minor version
    pub minor: u8,
    /// Patch version
    pub patch: u8,
}

impl std::fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let color = Color::new(255, 128, 64);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
    }

    #[test]
    fn test_color_from_hex() {
        let red = Color::from_hex(0xFF0000);
        assert_eq!(red, Color::RED);

        let green = Color::from_hex(0x00FF00);
        assert_eq!(green, Color::GREEN);

        let blue = Color::from_hex(0x0000FF);
        assert_eq!(blue, Color::BLUE);
    }

    #[test]
    fn test_color_to_bytes() {
        let color = Color::new(10, 20, 30);
        assert_eq!(color.to_bytes(), [10, 20, 30]);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::RED, Color::new(255, 0, 0));
        assert_eq!(Color::GREEN, Color::new(0, 255, 0));
        assert_eq!(Color::BLUE, Color::new(0, 0, 255));
        assert_eq!(Color::WHITE, Color::new(255, 255, 255));
        assert_eq!(Color::BLACK, Color::new(0, 0, 0));
    }

    #[test]
    fn test_color_from_tuple() {
        let color: Color = (100, 150, 200).into();
        assert_eq!(color, Color::new(100, 150, 200));
    }

    #[test]
    fn test_color_from_array() {
        let color: Color = [50, 100, 150].into();
        assert_eq!(color, Color::new(50, 100, 150));
    }

    #[test]
    fn test_firmware_version_display() {
        let version = FirmwareVersion {
            major: 1,
            minor: 2,
            patch: 3,
        };
        assert_eq!(version.to_string(), "1.2.3");
    }
}
