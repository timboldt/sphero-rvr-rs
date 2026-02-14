//! Transport layer for serial communication and message routing
//!
//! This module manages the physical UART connection and routes messages
//! between the synchronous API and the asynchronous serial line.
//!
//! Architecture:
//! - Owns the physical serial port (serialport crate)
//! - Manages sequence IDs and tracks pending requests
//! - Runs background RX thread to consume UART buffer
//! - Routes incoming Acks to waiting callers via oneshot channels
//! - Pushes async events/sensors to MPSC channels
//!
//! To be implemented in Phase 2:
//! - `dispatcher.rs`: Main dispatcher with thread management
