//! Sphero RVR Protocol Constants
//!
//! Device IDs, Command IDs, and other protocol-level constants
//! based on the Sphero RVR SDK documentation.

/// Routing node IDs for UART board-to-board communication
///
/// When communicating over the RVR's external UART expansion port,
/// packets must include source and target node IDs for the internal
/// routing mesh.
pub mod routing_node {
    /// Primary processor (Nordic MCU) - target for most commands
    pub const PRIMARY_PROCESSOR: u8 = 0x01;

    /// UART expansion port - source when sending commands externally
    pub const UART_PORT: u8 = 0x02;
}

/// Device IDs for RVR subsystems
pub mod device {
    /// Power device - controls wake, sleep, battery status
    pub const POWER: u8 = 0x13;

    /// IO device - controls LEDs, buttons, IR
    pub const IO: u8 = 0x1A;

    /// Drive device - controls motors, speed, heading
    pub const DRIVE: u8 = 0x16;

    /// Sensor device - IMU, color sensor, encoders
    pub const SENSOR: u8 = 0x18;

    /// System Info device - firmware version, hardware info
    pub const SYSTEM_INFO: u8 = 0x11;
}

/// Command IDs for the Power device
pub mod power_command {
    /// Wake the robot from sleep
    pub const WAKE: u8 = 0x0D;

    /// Put the robot to sleep
    pub const SLEEP: u8 = 0x01;

    /// Get battery percentage
    pub const GET_BATTERY_PERCENTAGE: u8 = 0x10;

    /// Get battery voltage state
    pub const GET_BATTERY_VOLTAGE_STATE: u8 = 0x17;
}

/// Command IDs for the IO device
pub mod io_command {
    /// Set all LEDs to a single color
    pub const SET_ALL_LEDS: u8 = 0x1A;

    /// Set individual LED colors
    pub const SET_LEDS: u8 = 0x1B;

    /// Get RGB LED values
    pub const GET_RGB_LED: u8 = 0x1C;
}

/// Command IDs for the Drive device
pub mod drive_command {
    /// Set raw motors (left, right)
    pub const SET_RAW_MOTORS: u8 = 0x01;

    /// Reset yaw angle
    pub const RESET_YAW: u8 = 0x06;

    /// Drive with heading and speed
    pub const DRIVE_WITH_HEADING: u8 = 0x07;

    /// Stop both motors
    pub const STOP: u8 = 0x08;
}

/// Command IDs for the Sensor device
pub mod sensor_command {
    /// Enable/disable sensor streaming
    pub const SET_SENSOR_STREAMING: u8 = 0x39;

    /// Start sensor streaming
    pub const START_SENSOR_STREAMING: u8 = 0x3A;

    /// Stop sensor streaming
    pub const STOP_SENSOR_STREAMING: u8 = 0x3B;

    /// Clear sensor streaming configuration
    pub const CLEAR_SENSOR_STREAMING: u8 = 0x3C;

    /// Configure sensor streaming interval
    pub const SET_STREAMING_INTERVAL: u8 = 0x46;
}

/// Command IDs for System Info device
pub mod system_info_command {
    /// Get firmware version
    pub const GET_FIRMWARE_VERSION: u8 = 0x02;

    /// Get hardware version
    pub const GET_HARDWARE_VERSION: u8 = 0x03;

    /// Get MAC address
    pub const GET_MAC_ADDRESS: u8 = 0x06;
}

/// LED bitmasks for targeting specific LEDs
pub mod led_bitmask {
    /// Right headlight LED
    pub const RIGHT_HEADLIGHT: u8 = 0x01;

    /// Left headlight LED
    pub const LEFT_HEADLIGHT: u8 = 0x02;

    /// Left status indication LED
    pub const LEFT_STATUS: u8 = 0x04;

    /// Right status indication LED
    pub const RIGHT_STATUS: u8 = 0x08;

    /// Battery door LEDs (front)
    pub const BATTERY_DOOR_FRONT: u8 = 0x10;

    /// Battery door LEDs (rear)
    pub const BATTERY_DOOR_REAR: u8 = 0x20;

    /// All LEDs
    pub const ALL: u8 = 0x3F;
}

/// Drive control modes
pub mod drive_mode {
    /// Stop mode (0 = coast, 1 = brake)
    pub const COAST: u8 = 0x00;
    pub const BRAKE: u8 = 0x01;
}

/// Response error codes
pub mod error_code {
    /// Command executed successfully
    pub const SUCCESS: u8 = 0x00;

    /// Bad device ID
    pub const BAD_DEVICE_ID: u8 = 0x01;

    /// Bad command ID
    pub const BAD_COMMAND_ID: u8 = 0x02;

    /// Not yet implemented
    pub const NOT_YET_IMPLEMENTED: u8 = 0x03;

    /// Command is restricted
    pub const RESTRICTED: u8 = 0x04;

    /// Bad data length
    pub const BAD_DATA_LENGTH: u8 = 0x05;

    /// Command failed
    pub const FAILED: u8 = 0x06;

    /// Bad parameter value
    pub const BAD_PARAMETER_VALUE: u8 = 0x07;

    /// Busy (try again later)
    pub const BUSY: u8 = 0x08;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_ids() {
        assert_eq!(device::POWER, 0x13);
        assert_eq!(device::IO, 0x1A);
        assert_eq!(device::DRIVE, 0x16);
    }

    #[test]
    fn test_led_bitmask() {
        // All LEDs should be the OR of individual LEDs
        assert_eq!(
            led_bitmask::ALL,
            led_bitmask::RIGHT_HEADLIGHT
                | led_bitmask::LEFT_HEADLIGHT
                | led_bitmask::LEFT_STATUS
                | led_bitmask::RIGHT_STATUS
                | led_bitmask::BATTERY_DOOR_FRONT
                | led_bitmask::BATTERY_DOOR_REAR
        );
    }
}
