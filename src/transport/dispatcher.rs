use crate::error::{Result, RvrError};
use crate::protocol::framing::{encode_bytes, EOP, SOP};
use crate::protocol::packet::Packet;
use crate::protocol::parser::SpheroParser;
use serialport::SerialPort;
use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Response channel for a single request
type ResponseSender = Sender<Packet>;

/// Dispatcher manages serial communication and routes messages
///
/// Architecture:
/// - Owns the serial port connection
/// - Assigns sequence numbers to outgoing packets
/// - Tracks pending requests in a HashMap (seq_num -> oneshot channel)
/// - Runs background RX thread that:
///   - Reads bytes from serial port
///   - Feeds to SpheroParser
///   - Routes responses to pending request channels
///   - Routes async notifications to notification channel
///
/// # Thread Safety
///
/// The Dispatcher is designed to be wrapped in Arc and shared between threads:
/// - Serial port is protected by Mutex
/// - Sequence counter uses AtomicU8
/// - Pending requests map is protected by Mutex
/// - RX thread owns the read half of the serial port
pub struct Dispatcher {
    /// Shared serial port (for writing)
    serial_port: Arc<Mutex<Box<dyn SerialPort>>>,

    /// Sequence number counter (wraps at 255)
    next_sequence: AtomicU8,

    /// Pending requests waiting for responses
    /// Maps sequence_number -> oneshot sender
    pending_requests: Arc<Mutex<HashMap<u8, ResponseSender>>>,

    /// Channel for async notifications (sensor data, events)
    notification_tx: Sender<Packet>,

    /// Receiver for async notifications (exposed to API layer via take_receiver)
    /// Wrapped in Option to allow transfer of ownership
    notification_rx: Mutex<Option<Receiver<Packet>>>,

    /// RX thread handle
    rx_thread: Mutex<Option<JoinHandle<()>>>,

    /// Shutdown flag for RX thread
    shutdown: Arc<AtomicBool>,
}

impl Dispatcher {
    /// Create a new Dispatcher and start background RX thread
    ///
    /// # Arguments
    ///
    /// * `port_name` - Serial port path (e.g., "/dev/serial0")
    /// * `baud_rate` - Baud rate (typically 115200 for Sphero RVR)
    ///
    /// # Returns
    ///
    /// Returns `Dispatcher` instance with RX thread running
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self> {
        // Open serial port
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(100))
            .open()?;

        let serial_port = Arc::new(Mutex::new(port));
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let shutdown = Arc::new(AtomicBool::new(false));

        // Create notification channel
        let (notification_tx, notification_rx) = mpsc::channel();

        // Clone serial port for RX thread
        let rx_serial = Arc::clone(&serial_port);
        let rx_pending = Arc::clone(&pending_requests);
        let rx_shutdown = Arc::clone(&shutdown);
        let rx_notif_tx = notification_tx.clone();

        // Spawn RX thread
        let rx_thread = thread::spawn(move || {
            Self::rx_thread_loop(rx_serial, rx_pending, rx_notif_tx, rx_shutdown);
        });

        Ok(Self {
            serial_port,
            next_sequence: AtomicU8::new(0),
            pending_requests,
            notification_tx,
            notification_rx: Mutex::new(Some(notification_rx)),
            rx_thread: Mutex::new(Some(rx_thread)),
            shutdown,
        })
    }

    /// Send a command packet and wait for response
    ///
    /// This method:
    /// 1. Assigns a sequence number
    /// 2. Registers a oneshot channel for the response
    /// 3. Serializes and sends the packet
    /// 4. Blocks waiting for response
    ///
    /// # Arguments
    ///
    /// * `packet` - Packet to send (sequence_number will be overwritten)
    ///
    /// # Returns
    ///
    /// Returns the response packet or timeout error
    pub fn send_command(&self, mut packet: Packet) -> Result<Packet> {
        // Assign sequence number
        let seq = self.next_sequence.fetch_add(1, Ordering::SeqCst);
        packet.sequence_number = seq;

        // Create response channel
        let (tx, rx) = mpsc::channel();

        // Register pending request
        {
            let mut pending = self.pending_requests.lock().unwrap();
            pending.insert(seq, tx);
        }

        // Send packet
        self.send_packet_internal(&packet)?;

        // Wait for response (with timeout)
        match rx.recv_timeout(Duration::from_secs(2)) {
            Ok(response) => Ok(response),
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Clean up pending request
                let mut pending = self.pending_requests.lock().unwrap();
                pending.remove(&seq);
                Err(RvrError::Timeout)
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(RvrError::Protocol(
                "Response channel disconnected".to_string(),
            )),
        }
    }

    /// Send a packet without waiting for response
    ///
    /// Useful for packets that don't expect a response
    pub fn send_packet_no_response(&self, packet: &Packet) -> Result<()> {
        self.send_packet_internal(packet)
    }

    /// Internal packet sending logic
    ///
    /// Serializes packet, applies SLIP encoding, adds framing, and writes to serial port
    fn send_packet_internal(&self, packet: &Packet) -> Result<()> {
        // Serialize packet to unescaped bytes
        let unescaped = packet.to_bytes();

        // Apply SLIP encoding
        let escaped = encode_bytes(&unescaped);

        // Build framed packet: SOP + escaped + EOP
        let mut framed = Vec::with_capacity(escaped.len() + 2);
        framed.push(SOP);
        framed.extend_from_slice(&escaped);
        framed.push(EOP);

        // Write to serial port
        let mut port = self.serial_port.lock().unwrap();
        port.write_all(&framed)?;
        port.flush()?;

        tracing::trace!(
            "TX: seq={} dev={:#04x} cmd={:#04x} len={}",
            packet.sequence_number,
            packet.device_id,
            packet.command_id,
            framed.len()
        );

        Ok(())
    }

    /// Background RX thread loop
    ///
    /// Continuously reads bytes from serial port, parses packets, and routes them
    ///
    /// Performance: Reads chunks of 1024 bytes at a time to minimize syscalls
    /// and mutex contention. At 115200 baud, bytes arrive ~every 86μs, so
    /// single-byte reads would cause severe CPU thrashing.
    fn rx_thread_loop(
        serial_port: Arc<Mutex<Box<dyn SerialPort>>>,
        pending_requests: Arc<Mutex<HashMap<u8, ResponseSender>>>,
        notification_tx: Sender<Packet>,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut parser = SpheroParser::new();
        let mut buffer = [0u8; 1024]; // Read chunks to minimize syscalls

        tracing::debug!("RX thread started");

        loop {
            // Check shutdown flag
            if shutdown.load(Ordering::Relaxed) {
                tracing::debug!("RX thread shutting down");
                break;
            }

            // Read chunk from serial port (single syscall + mutex lock)
            let bytes_read = {
                let mut port = serial_port.lock().unwrap();
                match port.read(&mut buffer) {
                    Ok(0) => continue, // No data available
                    Ok(n) => n,
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                        // Timeout is expected with non-blocking reads
                        continue;
                    }
                    Err(e) => {
                        tracing::error!("Serial read error: {}", e);
                        continue;
                    }
                }
            };

            // Feed chunk to parser (no mutex held here)
            for &byte in &buffer[..bytes_read] {
                match parser.feed(byte) {
                    Ok(Some(packet)) => {
                        tracing::trace!(
                            "RX: seq={} dev={:#04x} cmd={:#04x} is_resp={} payload_len={}",
                            packet.sequence_number,
                            packet.device_id,
                            packet.command_id,
                            packet.flags.is_response,
                            packet.payload.len()
                        );

                        // Route packet based on type
                        if packet.flags.is_response {
                            // This is a response to a command - route to pending request
                            let seq = packet.sequence_number;
                            let mut pending = pending_requests.lock().unwrap();
                            if let Some(sender) = pending.remove(&seq) {
                                if sender.send(packet).is_err() {
                                    tracing::warn!("Failed to send response for seq={}", seq);
                                }
                            } else {
                                tracing::warn!("Received response for unknown sequence: {}", seq);
                            }
                        } else {
                            // This is an async notification (sensor data, event)
                            if notification_tx.send(packet).is_err() {
                                tracing::warn!("Notification channel closed");
                            }
                        }
                    }
                    Ok(None) => {
                        // Still accumulating bytes
                    }
                    Err(e) => {
                        // Parser error (bad checksum, resync, etc.)
                        // This is expected on noisy lines - just log and continue
                        tracing::warn!("Parser error: {}", e);
                    }
                }
            }
        }

        tracing::debug!("RX thread exited");
    }

    /// Take ownership of the notification receiver
    ///
    /// This receiver gets async notifications like sensor data and events
    /// that arrive without being requested.
    ///
    /// Can only be called once - subsequent calls return None.
    ///
    /// # Usage
    ///
    /// The caller should spawn a dedicated thread to handle notifications:
    ///
    /// ```no_run
    /// # use sphero_rvr::transport::Dispatcher;
    /// # let dispatcher = Dispatcher::new("/dev/serial0", 115200).unwrap();
    /// if let Some(rx) = dispatcher.take_receiver() {
    ///     std::thread::spawn(move || {
    ///         for packet in rx {
    ///             println!("Notification: {:?}", packet);
    ///         }
    ///     });
    /// }
    /// ```
    pub fn take_receiver(&self) -> Option<Receiver<Packet>> {
        self.notification_rx.lock().unwrap().take()
    }

    /// Shutdown the dispatcher and wait for RX thread to exit
    pub fn shutdown(&self) -> Result<()> {
        tracing::debug!("Shutting down dispatcher");

        // Signal shutdown
        self.shutdown.store(true, Ordering::SeqCst);

        // Wait for RX thread to exit
        if let Some(handle) = self.rx_thread.lock().unwrap().take() {
            handle
                .join()
                .map_err(|_| RvrError::Protocol("Failed to join RX thread".to_string()))?;
        }

        tracing::debug!("Dispatcher shutdown complete");
        Ok(())
    }
}

impl Drop for Dispatcher {
    fn drop(&mut self) {
        // Best-effort shutdown
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a real serial port or mock.
    // For now, we'll test the packet routing logic in isolation.

    #[test]
    fn test_sequence_number_wrapping() {
        // Verify sequence numbers wrap correctly
        let seq = AtomicU8::new(254);
        assert_eq!(seq.fetch_add(1, Ordering::SeqCst), 254);
        assert_eq!(seq.fetch_add(1, Ordering::SeqCst), 255);
        assert_eq!(seq.fetch_add(1, Ordering::SeqCst), 0); // Wraps to 0
    }

    #[test]
    fn test_pending_requests_cleanup() {
        let pending: Arc<Mutex<HashMap<u8, ResponseSender>>> = Arc::new(Mutex::new(HashMap::new()));

        let (tx, _rx) = mpsc::channel();

        // Insert request
        {
            let mut map = pending.lock().unwrap();
            map.insert(42, tx);
            assert_eq!(map.len(), 1);
        }

        // Remove request
        {
            let mut map = pending.lock().unwrap();
            map.remove(&42);
            assert_eq!(map.len(), 0);
        }
    }
}
