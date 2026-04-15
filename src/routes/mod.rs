//! HTTP route configuration module
//!
//! Defines all web endpoints for the system monitor application.

pub mod dashboard;
pub mod metrics;

use actix_web::web;

/// Configure all application routes
///
/// # Routes
/// - `/` - Main dashboard page
/// - `/metrics/stream` - SSE endpoint for real-time metrics
/// - `/metrics/processes` - Process list endpoint
/// - `/metrics/disks` - Disk usage endpoint
/// - `/metrics/network` - Network statistics endpoint
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .route("/", web::get().to(dashboard::dashboard_page))
            .route("/metrics/stream", web::get().to(metrics::metrics_stream))
            .route("/metrics/processes", web::get().to(metrics::get_processes))
            .route("/metrics/disks", web::get().to(metrics::get_disks))
            .route("/metrics/network", web::get().to(metrics::get_network)),
    );
}
