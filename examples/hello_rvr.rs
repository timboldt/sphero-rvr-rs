//! Hello World example for Sphero RVR
//!
//! This example demonstrates the high-level API:
//! 1. Connect to the robot
//! 2. Wake it up
//! 3. Flash LEDs through different colors
//! 4. Put it to sleep
//!
//! Usage:
//!   cargo run --example hello_rvr
//!
//! Note: Requires a Sphero RVR connected to /dev/serial0

use sphero_rvr::api::types::Color;
use sphero_rvr::SpheroRvr;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("sphero_rvr=info")
        .init();

    println!("=== Sphero RVR Hello World ===\n");

    // Connect to robot
    println!("Connecting to /dev/serial0...");
    let mut rvr = SpheroRvr::connect("/dev/serial0")?;
    println!("✓ Connected!\n");

    // Wake up
    println!("Waking up robot...");
    rvr.wake()?;
    println!("✓ Awake!\n");
    thread::sleep(Duration::from_millis(500));

    // Get battery status
    println!("Checking battery...");
    match rvr.get_battery_percentage() {
        Ok(battery) => println!("✓ Battery: {}%\n", battery.percentage),
        Err(e) => println!("⚠ Could not read battery: {}\n", e),
    }

    // Color sequence
    let colors = [
        ("Red", Color::RED),
        ("Green", Color::GREEN),
        ("Blue", Color::BLUE),
        ("Yellow", Color::YELLOW),
        ("Cyan", Color::CYAN),
        ("Magenta", Color::MAGENTA),
        ("White", Color::WHITE),
    ];

    println!("LED Color Sequence:");
    for (name, color) in &colors {
        println!("  Setting LEDs to {}...", name);
        rvr.set_all_leds(*color)?;
        thread::sleep(Duration::from_millis(500));
    }
    println!("✓ Color sequence complete!\n");

    // Turn off LEDs
    println!("Turning off LEDs...");
    rvr.set_all_leds(Color::BLACK)?;
    thread::sleep(Duration::from_millis(500));
    println!("✓ LEDs off\n");

    // Sleep
    println!("Putting robot to sleep...");
    rvr.sleep()?;
    println!("✓ Sleeping!\n");

    // Shutdown
    println!("Disconnecting...");
    rvr.shutdown()?;
    println!("✓ Disconnected!\n");

    println!("=== Hello World Complete ===");

    Ok(())
}
