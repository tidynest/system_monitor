//! Metrics service implementation
//!
//! Provides an abstraction layer for system metrics collection,
//! enabling dependency injection and easier testing.

use crate::models::system::SystemMetrics;
use std::sync::Arc;

/// Trait for metrics collection services
///
/// Implement this trait to provide custom metrics collection
/// (e.g., for testing with mock data).
pub trait MetricsService: Send + Sync {
    /// Collect current system metrics
    fn collect(&self) -> SystemMetrics;
}

/// Production implementation of MetricsService
///
/// Uses the actual system collectors to gather real-time metrics.
pub struct RealMetricsService;

impl MetricsService for RealMetricsService {
    fn collect(&self) -> SystemMetrics {
        crate::collectors::system::collect_system_metrics()
    }
}

/// Type alias for a reference-counted metrics service
pub type MetricsServiceRef = Arc<dyn MetricsService>;
