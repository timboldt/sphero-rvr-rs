// Example demonstrating the Dispatcher layer
//
// This example shows how to use the Dispatcher directly to send commands
// and receive responses. In Phase 3, this will be wrapped by the high-level
// SpheroRvr API.
//
// Note: This requires a real Sphero RVR connected to /dev/serial0

use sphero_rvr::error::Result;
use sphero_rvr::protocol::packet::{Packet, PacketFlags};
use sphero_rvr::transport::Dispatcher;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("sphero_rvr=trace")
        .init();

    println!("Sphero RVR Dispatcher Demo");
    println!("===========================\n");

    // Create dispatcher (opens serial port and starts RX thread)
    println!("Opening serial port /dev/serial0 at 115200 baud...");
    let dispatcher = Dispatcher::new("/dev/serial0", 115200)?;
    println!("✓ Dispatcher initialized\n");

    // Example 1: Send a wake command
    println!("Example 1: Wake Command");
    println!("-----------------------");

    let wake_packet = Packet {
        flags: PacketFlags {
            is_response: false,
            requests_response: true,
            requests_only_error_response: false,
            is_activity: false,
            has_target_id: true,  // Required for UART routing
            has_source_id: true,  // Required for UART routing
            reserved: 0,
        },
        target_id: Some(0x01),    // Primary processor (Nordic MCU)
        source_id: Some(0x02),    // UART expansion port
        device_id: 0x13,          // Power device
        command_id: 0x0D,         // Wake command
        sequence_number: 0,       // Will be assigned by dispatcher
        payload: vec![],
    };

    match dispatcher.send_command(wake_packet) {
        Ok(response) => {
            println!("✓ Wake command succeeded!");
            println!("  Response seq: {}", response.sequence_number);
            println!("  Response flags: {:?}", response.flags);
        }
        Err(e) => {
            println!("✗ Wake command failed: {}", e);
        }
    }

    println!();

    // Example 2: Set all LEDs to green
    println!("Example 2: Set LEDs to Green");
    println!("----------------------------");

    let led_packet = Packet {
        flags: PacketFlags {
            is_response: false,
            requests_response: true,
            requests_only_error_response: false,
            is_activity: false,
            has_target_id: true,  // Required for UART routing
            has_source_id: true,  // Required for UART routing
            reserved: 0,
        },
        target_id: Some(0x01),    // Primary processor (Nordic MCU)
        source_id: Some(0x02),    // UART expansion port
        device_id: 0x1A,          // IO device
        command_id: 0x1A,         // Set all LEDs command
        sequence_number: 0,       // Will be assigned
        payload: vec![
            0x3F, // LED bitmask (all LEDs)
            0x00, // Red: 0
            0xFF, // Green: 255
            0x00, // Blue: 0
        ],
    };

    match dispatcher.send_command(led_packet) {
        Ok(response) => {
            println!("✓ LED command succeeded!");
            println!("  Response seq: {}", response.sequence_number);
        }
        Err(e) => {
            println!("✗ LED command failed: {}", e);
        }
    }

    println!();

    // Example 3: Monitor async notifications for 5 seconds
    println!("Example 3: Monitor Async Notifications");
    println!("---------------------------------------");
    println!("Listening for 5 seconds...");

    if let Some(rx) = dispatcher.take_receiver() {
        // Spawn dedicated notification handler thread
        let handle = thread::spawn(move || {
            let start = std::time::Instant::now();
            while start.elapsed() < Duration::from_secs(5) {
                if let Ok(packet) = rx.recv_timeout(Duration::from_millis(100)) {
                    println!(
                        "  Notification: dev={:#04x} cmd={:#04x} payload_len={}",
                        packet.device_id,
                        packet.command_id,
                        packet.payload.len()
                    );
                }
            }
        });

        handle.join().unwrap();
    }

    println!("Done listening.\n");

    // Example 4: Send sleep command
    println!("Example 4: Sleep Command");
    println!("------------------------");

    let sleep_packet = Packet {
        flags: PacketFlags {
            is_response: false,
            requests_response: true,
            requests_only_error_response: false,
            is_activity: false,
            has_target_id: true,  // Required for UART routing
            has_source_id: true,  // Required for UART routing
            reserved: 0,
        },
        target_id: Some(0x01),    // Primary processor (Nordic MCU)
        source_id: Some(0x02),    // UART expansion port
        device_id: 0x13,          // Power device
        command_id: 0x01,         // Sleep command
        sequence_number: 0,
        payload: vec![],
    };

    match dispatcher.send_command(sleep_packet) {
        Ok(response) => {
            println!("✓ Sleep command succeeded!");
            println!("  Response seq: {}", response.sequence_number);
        }
        Err(e) => {
            println!("✗ Sleep command failed: {}", e);
        }
    }

    println!();

    // Shutdown dispatcher
    println!("Shutting down dispatcher...");
    dispatcher.shutdown()?;
    println!("✓ Dispatcher shutdown complete");

    Ok(())
}
