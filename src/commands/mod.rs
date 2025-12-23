//! Command building and execution
//!
//! Stage 1: Structure only
//! Stage 2: LED control, status queries, power management
//! Stage 3: Full API implementation

pub mod builder;

// Device IDs (confirmed from Sphero SDK documentation)
pub const DEVICE_IO: u8 = 0x1A; // IO subsystem (LEDs)
pub const DEVICE_POWER: u8 = 0x13; // Power subsystem

// IO Commands (LED control)
pub const CMD_SET_ALL_LEDS: u8 = 0x1C; // Set all LEDs to RGB color

// Power Commands
pub const CMD_WAKE: u8 = 0x0D; // Wake from sleep
pub const CMD_SLEEP: u8 = 0x01; // Enter sleep mode

// Power/System Query Commands
pub const CMD_GET_BATTERY_PERCENTAGE: u8 = 0x10; // Get battery % (0-100)
pub const CMD_GET_BATTERY_VOLTAGE_STATE: u8 = 0x17; // Get voltage state (ok/low/critical)

// Note: Some command IDs above are based on common Sphero protocol patterns
// and will be verified during hardware testing in Stage 2
