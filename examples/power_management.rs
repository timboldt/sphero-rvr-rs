//! Power Management Example
//!
//! Demonstrates wake and sleep commands for the Sphero RVR.
//! This example shows how to control the power state of the robot.
//!
//! # Usage
//! ```bash
//! # On Raspberry Pi
//! ./deploy.sh --example power_management --run
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

    println!("Sphero RVR Power Management Example");
    println!("====================================\n");

    // Configure connection
    let config = RvrConfig::default();

    // Open connection to RVR
    println!("Connecting to RVR on /dev/serial0...");
    let mut rvr = RvrConnection::open("/dev/serial0", config).await?;
    println!("Connected!\n");

    // Ensure RVR is awake
    println!("1. Sending wake command...");
    rvr.wake().await?;
    println!("   RVR is now awake");
    sleep(Duration::from_secs(2)).await;

    // Set LEDs to green to confirm awake state
    println!("\n2. Setting LEDs to green (awake indicator)...");
    rvr.set_all_leds(0, 255, 0).await?;
    sleep(Duration::from_secs(2)).await;

    // Put RVR to sleep
    println!("\n3. Putting RVR to sleep...");
    println!("   (LEDs and sensors will turn off)");
    rvr.sleep().await?;
    println!("   RVR is now sleeping");
    sleep(Duration::from_secs(3)).await;

    // Wake RVR back up
    println!("\n4. Waking RVR back up...");
    rvr.wake().await?;
    println!("   RVR is awake again");
    sleep(Duration::from_secs(2)).await;

    // Set LEDs to blue to confirm wake
    println!("\n5. Setting LEDs to blue (awake confirmation)...");
    rvr.set_all_leds(0, 0, 255).await?;
    sleep(Duration::from_secs(2)).await;

    // Turn off LEDs and close
    println!("\n6. Turning off LEDs...");
    rvr.set_all_leds(0, 0, 0).await?;

    println!("\nClosing connection...");
    rvr.close().await?;
    println!("Done!");

    Ok(())
}
