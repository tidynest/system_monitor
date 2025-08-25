// src/models/mod.rs

pub mod cpu;
pub mod disk;
pub mod memory;
pub mod network;
pub mod process;
pub mod system;

// Re-export all model types
pub use cpu::CpuMetrics;
pub use disk::DiskMetrics;
pub use memory::MemoryMetrics;
pub use network::NetworkMetrics;
pub use process::ProcessMetrics;
