// Integration tests for Dispatcher
//
// Note: These tests require mocking or a loopback serial connection.
// For now, we document the expected behavior and test components in isolation.

use sphero_rvr::protocol::packet::{Packet, PacketFlags};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[test]
fn test_sequence_assignment() {
    // Verify that sequence numbers are assigned correctly and wrap at 256
    let seq = AtomicU8::new(0);

    // Assign 300 sequence numbers
    let mut numbers = Vec::new();
    for _ in 0..300 {
        numbers.push(seq.fetch_add(1, Ordering::SeqCst));
    }

    // First 256 should be 0..255
    assert_eq!(numbers[0], 0);
    assert_eq!(numbers[255], 255);

    // Should wrap to 0
    assert_eq!(numbers[256], 0);
    assert_eq!(numbers[299], 43);
}

#[test]
fn test_packet_routing_logic() {
    // Simulate the routing logic used in the RX thread

    type ResponseSender = mpsc::Sender<Packet>;
    let pending: Arc<Mutex<HashMap<u8, ResponseSender>>> = Arc::new(Mutex::new(HashMap::new()));
    let (notif_tx, notif_rx) = mpsc::channel();

    // Create a response packet (seq 42)
    let response_packet = Packet {
        flags: PacketFlags {
            is_response: true,
            requests_response: false,
            requests_only_error_response: false,
            is_activity: false,
            has_target_id: false,
            has_source_id: false,
            reserved: 0,
        },
        target_id: None,
        source_id: None,
        device_id: 0x13,
        command_id: 0x0D,
        sequence_number: 42,
        payload: vec![],
    };

    // Create an async notification
    let notification_packet = Packet {
        flags: PacketFlags {
            is_response: false,
            requests_response: false,
            requests_only_error_response: false,
            is_activity: true,
            has_target_id: false,
            has_source_id: false,
            reserved: 0,
        },
        target_id: None,
        source_id: None,
        device_id: 0x18,
        command_id: 0x25,
        sequence_number: 0,
        payload: vec![0x01, 0x02, 0x03],
    };

    // Register a pending request for seq 42
    let (resp_tx, resp_rx) = mpsc::channel();
    {
        let mut map = pending.lock().unwrap();
        map.insert(42, resp_tx);
    }

    // Route the response packet (simulating RX thread logic)
    if response_packet.flags.is_response {
        let seq = response_packet.sequence_number;
        let mut map = pending.lock().unwrap();
        if let Some(sender) = map.remove(&seq) {
            sender.send(response_packet.clone()).unwrap();
        }
    }

    // Verify response was received
    let received = resp_rx
        .recv_timeout(std::time::Duration::from_millis(100))
        .unwrap();
    assert_eq!(received.sequence_number, 42);
    assert!(received.flags.is_response);

    // Route the notification packet
    if !notification_packet.flags.is_response {
        notif_tx.send(notification_packet.clone()).unwrap();
    }

    // Verify notification was received
    let received = notif_rx
        .recv_timeout(std::time::Duration::from_millis(100))
        .unwrap();
    assert_eq!(received.device_id, 0x18);
    assert_eq!(received.payload, vec![0x01, 0x02, 0x03]);
    assert!(received.flags.is_activity);
}

#[test]
fn test_pending_request_timeout_cleanup() {
    // Verify that timed-out requests are properly cleaned up

    type ResponseSender = mpsc::Sender<Packet>;
    let pending: Arc<Mutex<HashMap<u8, ResponseSender>>> = Arc::new(Mutex::new(HashMap::new()));

    let (tx, rx) = mpsc::channel();

    // Register pending request
    {
        let mut map = pending.lock().unwrap();
        map.insert(100, tx);
        assert_eq!(map.len(), 1);
    }

    // Simulate timeout - try to receive with timeout
    let result = rx.recv_timeout(std::time::Duration::from_millis(50));
    assert!(result.is_err());

    // Clean up pending request (simulating what Dispatcher does)
    {
        let mut map = pending.lock().unwrap();
        map.remove(&100);
        assert_eq!(map.len(), 0);
    }
}

#[test]
fn test_packet_serialization_roundtrip() {
    // Verify packet serialization used in TX path
    use sphero_rvr::protocol::framing::{encode_bytes, EOP, SOP};

    let packet = Packet::new_command(0x13, 0x0D, 42, vec![0xAA, 0xBB]);

    // Simulate TX path
    let unescaped = packet.to_bytes();
    let escaped = encode_bytes(&unescaped);

    let mut framed = vec![SOP];
    framed.extend_from_slice(&escaped);
    framed.push(EOP);

    // Verify framing
    assert_eq!(framed[0], SOP);
    assert_eq!(framed[framed.len() - 1], EOP);

    // Verify we can parse it back
    use sphero_rvr::protocol::parser::SpheroParser;
    let mut parser = SpheroParser::new();

    let mut result = None;
    for &byte in &framed {
        if let Ok(Some(parsed)) = parser.feed(byte) {
            result = Some(parsed);
        }
    }

    let parsed = result.expect("Failed to parse packet");
    assert_eq!(parsed.device_id, packet.device_id);
    assert_eq!(parsed.command_id, packet.command_id);
    assert_eq!(parsed.payload, packet.payload);
}
