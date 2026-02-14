//! High-level synchronous API for Sphero RVR
//!
//! This module provides a clean, strongly-typed interface for controlling
//! the Sphero RVR robot. All operations are synchronous and blocking.
//!
//! The API layer has zero knowledge of transport details or byte-level
//! protocol framing. It works purely with high-level commands and responses.
