use crate::error::{Result, RvrError};
use crate::protocol::{checksum, encoding, packet::Packet};
use crate::response::Response;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

/// Configuration for RVR connection
#[derive(Debug, Clone)]
pub struct RvrConfig {
    pub baud_rate: u32,
    pub timeout_ms: u64,
}

impl Default for RvrConfig {
    fn default() -> Self {
        Self {
            baud_rate: 115_200, // RVR UART specification
            timeout_ms: 1000,
        }
    }
}

/// Main connection handle to Sphero RVR
pub struct RvrConnection {
    port: SerialStream,
    config: RvrConfig,
    sequence_number: u8,
}

impl RvrConnection {
    /// Open a connection to the RVR on the specified serial port
    pub async fn open(port_path: &str, config: RvrConfig) -> Result<Self> {
        tracing::info!("Opening connection to RVR on {}", port_path);

        let port = tokio_serial::new(port_path, config.baud_rate)
            .open_native_async()
            .map_err(RvrError::Serial)?;

        tracing::info!("Serial port opened successfully");

        Ok(Self {
            port,
            config,
            sequence_number: 0,
        })
    }

    /// Get the next sequence number for commands
    fn next_sequence(&mut self) -> u8 {
        let seq = self.sequence_number;
        self.sequence_number = self.sequence_number.wrapping_add(1);
        seq
    }

    /// Send a command packet to the RVR
    pub async fn send_command(&mut self, packet: &Packet) -> Result<()> {
        // Serialize packet (without SOP/checksum/EOP)
        let packet_bytes = packet.to_bytes();

        // Calculate checksum
        let checksum = checksum::calculate_checksum(&packet_bytes);

        // Encode packet with SLIP encoding (escaping special bytes)
        let encoded = encoding::encode_bytes(&packet_bytes);

        // Build complete frame: SOP + encoded_data + checksum + EOP
        let mut frame = BytesMut::new();
        frame.extend_from_slice(&[encoding::SOP]);
        frame.extend_from_slice(&encoded);
        frame.extend_from_slice(&[checksum, encoding::EOP]);

        tracing::debug!(
            "Sending packet: device={:02X}, command={:02X}, seq={}, payload_len={}",
            packet.device_id,
            packet.command_id,
            packet.sequence_number,
            packet.payload.len()
        );
        tracing::trace!("Frame bytes: {:02X?}", frame.as_ref());

        // Write to serial port
        self.port.write_all(&frame).await.map_err(RvrError::Io)?;
        self.port.flush().await.map_err(RvrError::Io)?;

        Ok(())
    }

    /// Receive a response packet from the RVR
    ///
    /// This is a blocking read that will wait for a complete packet
    /// Stage 2: Basic implementation
    /// Stage 3: Will add timeout, response matching, and async background processing
    pub async fn receive_response(&mut self) -> Result<Response> {
        // Read until we get SOP
        loop {
            let byte = self.read_byte().await?;
            if byte == encoding::SOP {
                break;
            }
        }

        // Read until EOP
        let mut packet_data = Vec::new();
        loop {
            let byte = self.read_byte().await?;
            if byte == encoding::EOP {
                break;
            }
            packet_data.push(byte);
        }

        // Last byte before EOP is checksum
        if packet_data.is_empty() {
            return Err(RvrError::Protocol("Empty packet".to_string()));
        }
        let checksum = packet_data.pop().unwrap();

        // Decode SLIP encoding
        let decoded = encoding::decode_bytes(&packet_data)?;

        // Verify checksum
        if !checksum::verify_checksum(&decoded, checksum) {
            tracing::warn!("Checksum mismatch");
            return Err(RvrError::Protocol("Checksum mismatch".to_string()));
        }

        // Parse packet
        let packet = Packet::from_bytes(&decoded)?;
        tracing::debug!(
            "Received packet: device={:02X}, command={:02X}, seq={}",
            packet.device_id,
            packet.command_id,
            packet.sequence_number
        );

        // Convert to response
        Response::from_packet(packet)
    }

    /// Helper to read a single byte
    async fn read_byte(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.port.read_exact(&mut buf).await.map_err(RvrError::Io)?;
        Ok(buf[0])
    }

    /// Send a command and wait for response
    pub async fn send_command_with_response(&mut self, packet: Packet) -> Result<Response> {
        self.send_command(&packet).await?;
        self.receive_response().await
    }

    // ========== High-Level API Methods (Stage 2) ==========

    /// Set all LEDs to the specified RGB color
    ///
    /// # Arguments
    /// * `red` - Red component (0-255)
    /// * `green` - Green component (0-255)
    /// * `blue` - Blue component (0-255)
    ///
    /// # Example
    /// ```no_run
    /// # use sphero_rvr::{RvrConnection, RvrConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut rvr = RvrConnection::open("/dev/serial0", RvrConfig::default()).await?;
    /// // Set LEDs to red
    /// rvr.set_all_leds(255, 0, 0).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_all_leds(&mut self, red: u8, green: u8, blue: u8) -> Result<()> {
        use crate::commands::{CMD_SET_ALL_LEDS, DEVICE_IO};

        tracing::info!("Setting all LEDs to RGB({}, {}, {})", red, green, blue);

        // RVR has 10 RGB LEDs, so we need 30 bytes (10 * 3) plus a 4-byte LED mask
        // LED mask: 0x3F, 0xFF, 0xFF, 0xFF enables all LEDs
        let mut payload = vec![0x3F, 0xFF, 0xFF, 0xFF];

        // Add RGB triplets for all 10 LEDs
        for _ in 0..10 {
            payload.push(red);
            payload.push(green);
            payload.push(blue);
        }

        let seq = self.next_sequence();
        let packet = Packet::new_command(DEVICE_IO, CMD_SET_ALL_LEDS, seq, payload);

        let response = self.send_command_with_response(packet).await?;
        if !response.is_success() {
            return Err(RvrError::CommandFailed(format!(
                "LED command failed with error code {}",
                response.error_code
            )));
        }

        Ok(())
    }

    /// Get battery percentage (0-100)
    ///
    /// Returns the estimated battery charge remaining as a percentage
    pub async fn get_battery_percentage(&mut self) -> Result<u8> {
        use crate::commands::{CMD_GET_BATTERY_PERCENTAGE, DEVICE_POWER};

        tracing::info!("Querying battery percentage");

        let seq = self.next_sequence();
        let packet = Packet::new_command(DEVICE_POWER, CMD_GET_BATTERY_PERCENTAGE, seq, vec![]);

        let response = self.send_command_with_response(packet).await?;
        if !response.is_success() {
            return Err(RvrError::CommandFailed(format!(
                "Battery query failed with error code {}",
                response.error_code
            )));
        }

        // Extract percentage from payload (first byte)
        let percentage = response.payload.first().copied().ok_or_else(|| {
            RvrError::Protocol("Battery response missing percentage data".to_string())
        })?;

        tracing::info!("Battery percentage: {}%", percentage);
        Ok(percentage)
    }

    /// Get battery voltage state
    ///
    /// Returns:
    /// - 0: Unknown
    /// - 1: OK
    /// - 2: Low
    /// - 3: Critical
    pub async fn get_battery_voltage_state(&mut self) -> Result<u8> {
        use crate::commands::{CMD_GET_BATTERY_VOLTAGE_STATE, DEVICE_POWER};

        tracing::info!("Querying battery voltage state");

        let seq = self.next_sequence();
        let packet = Packet::new_command(DEVICE_POWER, CMD_GET_BATTERY_VOLTAGE_STATE, seq, vec![]);

        let response = self.send_command_with_response(packet).await?;
        if !response.is_success() {
            return Err(RvrError::CommandFailed(format!(
                "Battery state query failed with error code {}",
                response.error_code
            )));
        }

        let state = response.payload.first().copied().ok_or_else(|| {
            RvrError::Protocol("Battery state response missing data".to_string())
        })?;

        let state_str = match state {
            0 => "Unknown",
            1 => "OK",
            2 => "Low",
            3 => "Critical",
            _ => "Invalid",
        };
        tracing::info!("Battery voltage state: {} ({})", state, state_str);

        Ok(state)
    }

    /// Wake the RVR from sleep mode
    pub async fn wake(&mut self) -> Result<()> {
        use crate::commands::{CMD_WAKE, DEVICE_POWER};

        tracing::info!("Sending wake command");

        let seq = self.next_sequence();
        let packet = Packet::new_command(DEVICE_POWER, CMD_WAKE, seq, vec![]);

        let response = self.send_command_with_response(packet).await?;
        if !response.is_success() {
            return Err(RvrError::CommandFailed(format!(
                "Wake command failed with error code {}",
                response.error_code
            )));
        }

        tracing::info!("RVR awake");
        Ok(())
    }

    /// Put the RVR into sleep mode
    ///
    /// This disables driving, LEDs, and sensors to conserve power
    pub async fn sleep(&mut self) -> Result<()> {
        use crate::commands::{CMD_SLEEP, DEVICE_POWER};

        tracing::info!("Sending sleep command");

        let seq = self.next_sequence();
        let packet = Packet::new_command(DEVICE_POWER, CMD_SLEEP, seq, vec![]);

        let response = self.send_command_with_response(packet).await?;
        if !response.is_success() {
            return Err(RvrError::CommandFailed(format!(
                "Sleep command failed with error code {}",
                response.error_code
            )));
        }

        tracing::info!("RVR sleeping");
        Ok(())
    }

    /// Close the connection (explicit shutdown)
    pub async fn close(self) -> Result<()> {
        tracing::info!("Closing RVR connection");
        // SerialStream is dropped automatically, no explicit shutdown needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RvrConfig::default();
        assert_eq!(config.baud_rate, 115_200);
        assert_eq!(config.timeout_ms, 1000);
    }

    #[test]
    fn test_sequence_number_wrapping() {
        // We can't actually open a serial port in tests, so we'll skip the connection test
        // This would require a mock serial port
        // Testing sequence number wrapping would need a mock RvrConnection
    }
}
