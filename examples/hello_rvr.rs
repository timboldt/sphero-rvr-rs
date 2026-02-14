//! Hello World example for Sphero RVR
//!
//! This example demonstrates the high-level API:
//! 1. Connect to the robot
//! 2. Spawn a notification handler thread to monitor async messages
//! 3. Wake the robot up
//! 4. Check battery status
//! 5. Flash LEDs through different colors
//! 6. Put the robot to sleep
//! 7. Clean shutdown
//!
//! The notification handler runs in the background and displays any
//! async messages from the robot (sensor data, events, etc.)
//!
//! Usage:
//!   cargo run --example hello_rvr
//!
//! Note: Requires a Sphero RVR connected to /dev/serial0

use sphero_rvr::api::types::Color;
use sphero_rvr::SpheroRvr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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

    // Spawn notification handler thread
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    let notification_thread = if let Some(rx) = rvr.take_receiver() {
        Some(thread::spawn(move || {
            println!("📡 Notification handler started\n");
            let mut notification_count = 0;

            while running_clone.load(Ordering::Relaxed) {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(packet) => {
                        notification_count += 1;
                        println!(
                            "  📨 Notification #{}: device={:#04x} cmd={:#04x} payload_len={}",
                            notification_count,
                            packet.device_id,
                            packet.command_id,
                            packet.payload.len()
                        );

                        // Show first few bytes of payload if present
                        if !packet.payload.is_empty() {
                            let preview: Vec<String> = packet
                                .payload
                                .iter()
                                .take(8)
                                .map(|b| format!("{:02x}", b))
                                .collect();
                            println!(
                                "     Data: [{}{}]",
                                preview.join(" "),
                                if packet.payload.len() > 8 { " ..." } else { "" }
                            );
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        // Normal timeout, continue
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        println!("  📡 Notification channel closed");
                        break;
                    }
                }
            }

            println!(
                "\n📡 Notification handler stopped (received {} notifications)",
                notification_count
            );
        }))
    } else {
        println!("⚠ Could not get notification receiver (already taken)\n");
        None
    };

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

    // Stop notification thread
    running.store(false, Ordering::Relaxed);
    if let Some(handle) = notification_thread {
        handle.join().expect("Failed to join notification thread");
    }

    rvr.shutdown()?;
    println!("✓ Disconnected!\n");

    println!("=== Hello World Complete ===");

    Ok(())
}
