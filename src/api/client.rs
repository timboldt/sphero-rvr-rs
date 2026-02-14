//! High-level Sphero RVR client

use crate::api::constants::*;
use crate::api::types::{BatteryState, Color, FirmwareVersion};
use crate::error::{Result, RvrError};
use crate::protocol::packet::{Packet, PacketFlags};
use crate::transport::Dispatcher;
use std::sync::mpsc::Receiver;

/// High-level client for controlling Sphero RVR
///
/// This is the main entry point for the Sphero RVR API. It provides
/// strongly-typed, synchronous methods for controlling the robot.
///
/// # Example
///
/// ```no_run
/// use sphero_rvr::SpheroRvr;
/// use sphero_rvr::api::types::Color;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Connect to the robot
/// let mut rvr = SpheroRvr::connect("/dev/serial0")?;
///
/// // Wake up
/// rvr.wake()?;
///
/// // Set all LEDs to green
/// rvr.set_all_leds(Color::GREEN)?;
///
/// // Sleep
/// rvr.sleep()?;
/// # Ok(())
/// # }
/// ```
pub struct SpheroRvr {
    dispatcher: Dispatcher,
}

impl SpheroRvr {
    /// Connect to a Sphero RVR on the specified serial port
    ///
    /// # Arguments
    ///
    /// * `port` - Serial port path (e.g., "/dev/serial0" on Raspberry Pi)
    ///
    /// # Returns
    ///
    /// Returns a connected `SpheroRvr` instance ready to receive commands
    ///
    /// # Errors
    ///
    /// Returns an error if the serial port cannot be opened
    pub fn connect(port: &str) -> Result<Self> {
        let dispatcher = Dispatcher::new(port, 115200)?;
        Ok(Self { dispatcher })
    }

    /// Wake the robot from sleep mode
    ///
    /// The robot must be awake before other commands will work.
    /// This is typically the first command sent after connecting.
    pub fn wake(&mut self) -> Result<()> {
        tracing::debug!("Sending wake command");

        let packet = self.build_command(device::POWER, power_command::WAKE, vec![]);

        let response = self.dispatcher.send_command(packet)?;
        self.check_response(&response)?;

        tracing::debug!("Wake command successful");
        Ok(())
    }

    /// Put the robot to sleep
    ///
    /// The robot will enter low-power sleep mode. Send wake() to resume.
    pub fn sleep(&mut self) -> Result<()> {
        tracing::debug!("Sending sleep command");

        let packet = self.build_command(device::POWER, power_command::SLEEP, vec![]);

        let response = self.dispatcher.send_command(packet)?;
        self.check_response(&response)?;

        tracing::debug!("Sleep command successful");
        Ok(())
    }

    /// Set all LEDs to the same color
    ///
    /// # Arguments
    ///
    /// * `color` - RGB color to set
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sphero_rvr::SpheroRvr;
    /// # use sphero_rvr::api::types::Color;
    /// # let mut rvr = SpheroRvr::connect("/dev/serial0").unwrap();
    /// // Set all LEDs to red
    /// rvr.set_all_leds(Color::RED)?;
    ///
    /// // Set custom color
    /// rvr.set_all_leds(Color::new(128, 64, 255))?;
    /// # Ok::<(), sphero_rvr::error::RvrError>(())
    /// ```
    pub fn set_all_leds(&mut self, color: Color) -> Result<()> {
        tracing::debug!(
            "Setting all LEDs to RGB({}, {}, {})",
            color.r,
            color.g,
            color.b
        );

        let payload = vec![
            led_bitmask::ALL, // LED bitmask (all LEDs)
            color.r,          // Red
            color.g,          // Green
            color.b,          // Blue
        ];

        let packet = self.build_command(device::IO, io_command::SET_ALL_LEDS, payload);

        let response = self.dispatcher.send_command(packet)?;
        self.check_response(&response)?;

        tracing::debug!("Set LEDs successful");
        Ok(())
    }

    /// Set specific LEDs to a color
    ///
    /// # Arguments
    ///
    /// * `led_mask` - Bitmask of which LEDs to set (see `led_bitmask` constants)
    /// * `color` - RGB color to set
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sphero_rvr::SpheroRvr;
    /// # use sphero_rvr::api::types::Color;
    /// # use sphero_rvr::api::constants::led_bitmask;
    /// # let mut rvr = SpheroRvr::connect("/dev/serial0").unwrap();
    /// // Set only headlights to blue
    /// let headlights = led_bitmask::LEFT_HEADLIGHT | led_bitmask::RIGHT_HEADLIGHT;
    /// rvr.set_leds(headlights, Color::BLUE)?;
    /// # Ok::<(), sphero_rvr::error::RvrError>(())
    /// ```
    pub fn set_leds(&mut self, led_mask: u8, color: Color) -> Result<()> {
        tracing::debug!(
            "Setting LEDs (mask={:#04x}) to RGB({}, {}, {})",
            led_mask,
            color.r,
            color.g,
            color.b
        );

        let payload = vec![
            led_mask, // LED bitmask
            color.r,  // Red
            color.g,  // Green
            color.b,  // Blue
        ];

        let packet = self.build_command(device::IO, io_command::SET_ALL_LEDS, payload);

        let response = self.dispatcher.send_command(packet)?;
        self.check_response(&response)?;

        Ok(())
    }

    /// Get the battery percentage
    ///
    /// # Returns
    ///
    /// Battery state with percentage (0-100)
    pub fn get_battery_percentage(&mut self) -> Result<BatteryState> {
        tracing::debug!("Getting battery percentage");

        let packet =
            self.build_command(device::POWER, power_command::GET_BATTERY_PERCENTAGE, vec![]);

        let response = self.dispatcher.send_command(packet)?;
        self.check_response(&response)?;

        // Parse battery percentage from response payload
        if response.payload.is_empty() {
            return Err(RvrError::InvalidResponse(
                "Battery response has no payload".to_string(),
            ));
        }

        let percentage = response.payload[0];

        tracing::debug!("Battery percentage: {}%", percentage);
        Ok(BatteryState { percentage })
    }

    /// Reset the yaw angle to zero
    ///
    /// Useful for calibrating the robot's orientation
    pub fn reset_yaw(&mut self) -> Result<()> {
        tracing::debug!("Resetting yaw");

        let packet = self.build_command(device::DRIVE, drive_command::RESET_YAW, vec![]);

        let response = self.dispatcher.send_command(packet)?;
        self.check_response(&response)?;

        Ok(())
    }

    /// Stop all motors
    ///
    /// # Arguments
    ///
    /// * `brake` - If true, brake motors. If false, coast to stop.
    pub fn stop(&mut self, brake: bool) -> Result<()> {
        tracing::debug!("Stopping motors (brake={})", brake);

        let mode = if brake {
            drive_mode::BRAKE
        } else {
            drive_mode::COAST
        };

        let packet = self.build_command(device::DRIVE, drive_command::STOP, vec![mode]);

        let response = self.dispatcher.send_command(packet)?;
        self.check_response(&response)?;

        Ok(())
    }

    /// Take ownership of the notification receiver
    ///
    /// This allows you to receive async notifications like sensor data.
    /// Can only be called once.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use sphero_rvr::SpheroRvr;
    /// # let rvr = SpheroRvr::connect("/dev/serial0").unwrap();
    /// if let Some(rx) = rvr.take_receiver() {
    ///     std::thread::spawn(move || {
    ///         for packet in rx {
    ///             println!("Notification: {:?}", packet);
    ///         }
    ///     });
    /// }
    /// ```
    pub fn take_receiver(&self) -> Option<Receiver<Packet>> {
        self.dispatcher.take_receiver()
    }

    /// Shutdown the connection gracefully
    ///
    /// This will stop the background RX thread and close the serial port.
    /// The robot will remain in its current state (awake/asleep).
    pub fn shutdown(self) -> Result<()> {
        tracing::debug!("Shutting down SpheroRvr");
        self.dispatcher.shutdown()
    }

    // === Helper Methods ===

    /// Build a command packet with standard flags for UART board-to-board communication
    ///
    /// When communicating over the RVR's external UART expansion port, the internal
    /// routing mesh requires explicit source and target node IDs:
    /// - Target: Primary processor (Nordic MCU)
    /// - Source: UART expansion port
    ///
    /// Without these, the internal router may drop packets or return routing errors.
    fn build_command(&self, device_id: u8, command_id: u8, payload: Vec<u8>) -> Packet {
        use routing_node::{PRIMARY_PROCESSOR, UART_PORT};

        Packet {
            flags: PacketFlags {
                is_response: false,
                requests_response: true,
                is_activity: false,
                has_target_id: true, // Required for UART routing
                has_source_id: true, // Required for UART routing
                reserved: 0,
            },
            target_id: Some(PRIMARY_PROCESSOR), // Target: Primary processor (Nordic MCU)
            source_id: Some(UART_PORT),         // Source: UART expansion port
            device_id,
            command_id,
            sequence_number: 0, // Will be assigned by dispatcher
            payload,
        }
    }

    /// Check if a response indicates success or error
    fn check_response(&self, response: &Packet) -> Result<()> {
        // Response payload format: [ERROR_CODE, ...]
        // If payload is empty, assume success
        if response.payload.is_empty() {
            return Ok(());
        }

        let error_code = response.payload[0];

        match error_code {
            error_code::SUCCESS => Ok(()),
            error_code::BAD_DEVICE_ID => {
                Err(RvrError::InvalidResponse("Bad device ID".to_string()))
            }
            error_code::BAD_COMMAND_ID => {
                Err(RvrError::InvalidResponse("Bad command ID".to_string()))
            }
            error_code::NOT_YET_IMPLEMENTED => Err(RvrError::InvalidResponse(
                "Command not yet implemented".to_string(),
            )),
            error_code::RESTRICTED => Err(RvrError::InvalidResponse(
                "Command is restricted".to_string(),
            )),
            error_code::BAD_DATA_LENGTH => {
                Err(RvrError::InvalidResponse("Bad data length".to_string()))
            }
            error_code::FAILED => Err(RvrError::CommandFailed(error_code)),
            error_code::BAD_PARAMETER_VALUE => {
                Err(RvrError::InvalidResponse("Bad parameter value".to_string()))
            }
            error_code::BUSY => Err(RvrError::InvalidResponse("Device is busy".to_string())),
            code => Err(RvrError::CommandFailed(code)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command() {
        let dispatcher = Dispatcher::new("/dev/null", 115200);
        // Skip test if serial port can't open (expected on most systems)
        if dispatcher.is_err() {
            return;
        }

        let rvr = SpheroRvr {
            dispatcher: dispatcher.unwrap(),
        };

        let packet = rvr.build_command(device::POWER, power_command::WAKE, vec![]);

        assert_eq!(packet.device_id, device::POWER);
        assert_eq!(packet.command_id, power_command::WAKE);
        assert!(!packet.flags.is_response);
        assert!(packet.flags.requests_response);
        assert!(packet.payload.is_empty());

        // Verify UART routing fields are set
        assert!(packet.flags.has_target_id);
        assert!(packet.flags.has_source_id);
        assert_eq!(packet.target_id, Some(routing_node::PRIMARY_PROCESSOR));
        assert_eq!(packet.source_id, Some(routing_node::UART_PORT));
    }

    #[test]
    fn test_check_response_success() {
        let dispatcher = Dispatcher::new("/dev/null", 115200);
        if dispatcher.is_err() {
            return;
        }

        let rvr = SpheroRvr {
            dispatcher: dispatcher.unwrap(),
        };

        // Empty payload means success
        let response = Packet {
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
            device_id: device::POWER,
            command_id: power_command::WAKE,
            sequence_number: 1,
            payload: vec![],
        };

        assert!(rvr.check_response(&response).is_ok());

        // Explicit success code
        let response_with_success = Packet {
            payload: vec![error_code::SUCCESS],
            ..response
        };

        assert!(rvr.check_response(&response_with_success).is_ok());
    }

    #[test]
    fn test_check_response_error() {
        let dispatcher = Dispatcher::new("/dev/null", 115200);
        if dispatcher.is_err() {
            return;
        }

        let rvr = SpheroRvr {
            dispatcher: dispatcher.unwrap(),
        };

        let response = Packet {
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
            device_id: device::POWER,
            command_id: power_command::WAKE,
            sequence_number: 1,
            payload: vec![error_code::FAILED],
        };

        assert!(matches!(
            rvr.check_response(&response),
            Err(RvrError::CommandFailed(_))
        ));
    }
}
