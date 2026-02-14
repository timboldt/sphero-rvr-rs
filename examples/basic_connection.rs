//! Basic connection example (PLACEHOLDER - to be implemented in Phase 3)
//!
//! This example will demonstrate:
//! - Opening a serial connection to RVR
//! - Sending basic commands
//! - Graceful shutdown
//!
//! Usage (once implemented):
//!   cargo run --example basic_connection
//!
//! On Raspberry Pi (after deployment):
//!   ./basic_connection

// This is a placeholder example that will be implemented in Phase 3
// after the SpheroRvr client API is built.

fn main() {
    println!("Basic connection example - to be implemented in Phase 3");
    println!();
    println!("The new synchronous API will look like:");
    println!();
    println!("  use sphero_rvr::SpheroRvr;");
    println!();
    println!("  fn main() -> Result<(), Box<dyn std::error::Error>> {{");
    println!("      let mut rvr = SpheroRvr::connect(\"/dev/serial0\")?;");
    println!("      rvr.wake()?;");
    println!("      rvr.set_all_leds(0, 255, 0)?; // Green");
    println!("      rvr.sleep()?;");
    println!("      Ok(())");
    println!("  }}");
    println!();
    println!("See CLAUDE.md for implementation phases and architecture details.");
}

// Future implementation will use the synchronous API:
/*
use sphero_rvr::{Result, SpheroRvr};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Sphero RVR Basic Connection Example");

    // Determine serial port based on platform
    let port = if cfg!(target_os = "linux") {
        "/dev/serial0" // Raspberry Pi UART
    } else {
        "/dev/ttyUSB0" // Generic fallback
    };

    tracing::info!("Attempting to connect to RVR on {}", port);

    // Open connection (synchronous)
    let mut rvr = SpheroRvr::connect(port)?;
    tracing::info!("Successfully connected to RVR!");

    // Wake the robot
    rvr.wake()?;

    // Set LEDs to green
    rvr.set_all_leds(0, 255, 0)?;

    // Sleep for a moment
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Put robot to sleep
    rvr.sleep()?;

    tracing::info!("Connection closed successfully");
    Ok(())
}
*/
