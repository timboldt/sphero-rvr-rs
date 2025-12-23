use thiserror::Error;

/// Main error type for Sphero RVR operations
#[derive(Error, Debug)]
pub enum RvrError {
    #[error("Serial port error: {0}")]
    Serial(#[from] tokio_serial::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Invalid checksum: expected {expected:#04x}, got {actual:#04x}")]
    Checksum { expected: u8, actual: u8 },

    #[error("Timeout waiting for response")]
    Timeout,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),
}

/// Convenience Result type
pub type Result<T> = std::result::Result<T, RvrError>;
