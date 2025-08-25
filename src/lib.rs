// src/lib.rs

pub mod collectors;
pub mod config;
pub mod models;
pub mod utils;

// Re-export commonly used types
pub use models::system::SystemMetrics;
pub use collectors::system::collect_system_metrics;
pub use config::Config;