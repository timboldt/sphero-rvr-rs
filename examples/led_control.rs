//! LED Control Example
//!
//! Demonstrates controlling the Sphero RVR LEDs using the Stage 2 API.
//! This example cycles through different colors to show LED control.
//!
//! # Usage
//! ```bash
//! # On Raspberry Pi
//! ./deploy.sh --example led_control --run
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

    println!("Sphero RVR LED Control Example");
    println!("================================\n");

    // Configure connection
    let config = RvrConfig::default();

    // Open connection to RVR
    println!("Connecting to RVR on /dev/serial0...");
    let mut rvr = RvrConnection::open("/dev/serial0", config).await?;
    println!("Connected!\n");

    // Cycle through colors
    let colors = [
        ("Red", 255, 0, 0),
        ("Green", 0, 255, 0),
        ("Blue", 0, 0, 255),
        ("Yellow", 255, 255, 0),
        ("Cyan", 0, 255, 255),
        ("Magenta", 255, 0, 255),
        ("White", 255, 255, 255),
        ("Orange", 255, 128, 0),
        ("Purple", 128, 0, 255),
    ];

    for (name, r, g, b) in colors.iter() {
        println!("Setting LEDs to {}: RGB({}, {}, {})", name, r, g, b);
        rvr.set_all_leds(*r, *g, *b).await?;
        sleep(Duration::from_secs(1)).await;
    }

    // Turn off LEDs
    println!("\nTurning off LEDs...");
    rvr.set_all_leds(0, 0, 0).await?;

    // Close connection
    println!("Closing connection...");
    rvr.close().await?;
    println!("Done!");

    Ok(())
}
