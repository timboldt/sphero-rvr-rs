//! Basic connection example
//!
//! Demonstrates:
//! - Opening a serial connection to RVR
//! - Configuring connection parameters
//! - Graceful shutdown
//!
//! Usage:
//!   cargo run --example basic_connection
//!
//! On Raspberry Pi (after deployment):
//!   ./basic_connection

use sphero_rvr::{Result, RvrConfig, RvrConnection};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Sphero RVR Basic Connection Example");

    // Configure connection
    let config = RvrConfig::default();
    tracing::info!("Using configuration: {:?}", config);

    // Determine serial port based on platform
    let port = if cfg!(target_os = "linux") {
        "/dev/serial0" // Raspberry Pi UART
    } else {
        "/dev/ttyUSB0" // Generic fallback
    };

    tracing::info!("Attempting to connect to RVR on {}", port);

    // Open connection
    match RvrConnection::open(port, config).await {
        Ok(rvr) => {
            tracing::info!("Successfully connected to RVR!");

            // Stage 2 will add:
            // - Sending a ping/wake command
            // - Reading firmware version
            // - LED blink test

            // Keep connection open briefly
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            // Graceful shutdown
            rvr.close().await?;
            tracing::info!("Connection closed successfully");
        }
        Err(e) => {
            tracing::error!("Failed to connect to RVR: {}", e);
            return Err(e);
        }
    }

    Ok(())
}
