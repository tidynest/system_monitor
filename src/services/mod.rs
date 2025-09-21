//! Service layer module
//!
//! Provides abstractions for metrics collection to enable testing and modularity.

pub mod metrics_service;

// Re-export all public items from metrics_service
pub use metrics_service::*;