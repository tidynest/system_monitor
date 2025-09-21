//! System Monitor Dashboard Library
//!
//! This library provides real-time system monitoring capabilities including:
//! - CPU, memory, disk, and network metrics collection
//! - Process monitoring and tracking
//! - Web-based dashboard with Server-Sent Events (SSE) for real-time updates

pub mod collectors;
pub mod config;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

// Re-export commonly used types for external consumers
pub use collectors::system::collect_system_metrics;
pub use config::Config;
pub use models::system::SystemMetrics;