//! Battery Status Example
//!
//! Demonstrates querying battery status information from the Sphero RVR.
//! Shows battery percentage and voltage state.
//!
//! # Usage
//! ```bash
//! # On Raspberry Pi
//! ./deploy.sh --example battery_status --run
//! ```

use sphero_rvr::{RvrConfig, RvrConnection};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    println!("Sphero RVR Battery Status Example");
    println!("===================================\n");

    // Configure connection
    let config = RvrConfig::default();

    // Open connection to RVR
    println!("Connecting to RVR on /dev/serial0...");
    let mut rvr = RvrConnection::open("/dev/serial0", config).await?;
    println!("Connected!\n");

    // Poll battery status every 5 seconds for 30 seconds
    for i in 1..=6 {
        println!("Battery Status Check #{}", i);
        println!("--------------------------");

        // Get battery percentage
        match rvr.get_battery_percentage().await {
            Ok(percentage) => {
                println!("  Battery Level:  {}%", percentage);

                // Visual battery bar
                let bars = (percentage / 10) as usize;
                let bar = "█".repeat(bars) + &"░".repeat(10 - bars);
                println!("  Battery Bar:    [{}]", bar);
            }
            Err(e) => println!("  Error getting battery percentage: {}", e),
        }

        // Get battery voltage state
        match rvr.get_battery_voltage_state().await {
            Ok(state) => {
                let state_str = match state {
                    0 => "Unknown",
                    1 => "OK ✓",
                    2 => "Low ⚠",
                    3 => "Critical ⚠⚠",
                    _ => "Invalid",
                };
                println!("  Voltage State:  {}", state_str);
            }
            Err(e) => println!("  Error getting voltage state: {}", e),
        }

        println!();

        if i < 6 {
            sleep(Duration::from_secs(5)).await;
        }
    }

    // Close connection
    println!("Closing connection...");
    rvr.close().await?;
    println!("Done!");

    Ok(())
}
