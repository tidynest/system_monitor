//! Data models for system metrics
//!
//! Defines the structures used to represent various system metrics
//! collected by the monitoring application.

pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod process;
pub mod system;

// Re-export primary model types for convenience
pub use cpu::CpuMetrics;
pub use disk::DiskMetrics;
pub use memory::MemoryMetrics;
pub use network::NetworkMetrics;
pub use process::ProcessMetrics;
