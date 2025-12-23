use crate::error::{Result, RvrError};
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

    /// Close the connection (explicit shutdown)
    pub async fn close(self) -> Result<()> {
        tracing::info!("Closing RVR connection");
        // SerialStream is dropped automatically, no explicit shutdown needed
        Ok(())
    }
}

// Stage 2/3 will add:
// - send_command() method
// - receive_response() method
// - Background task for unsolicited responses
// - Command/response matching via sequence numbers

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
        let config = RvrConfig::default();
        // We can't actually open a serial port in tests, so we'll skip the connection test
        // This would require a mock serial port
    }
}
