//! System Monitor Dashboard Server
//!
//! Main entry point for the system monitoring web application.
//! Provides real-time system metrics via a web dashboard using SSE.

use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use std::sync::Arc;

mod collectors;
mod config;
mod models;
mod routes;
mod services;
mod utils;

/// Application entry point
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialise logging with production defaults
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn,system_monitor=info")
    ).init();

    // Load configuration from environment
    let cfg = config::Config::from_env();
    let bind_addr = cfg.bind_address();

    // Create the metrics service
    let metrics_service: services::MetricsServiceRef = Arc::new(services::RealMetricsService);

    // Log startup information
    log::info!("System Monitor Dashboard starting");
    log::info!("Binding to: {}", bind_addr);

    HttpServer::new(move || {
        App::new()
            // Add logging middleware (production-appropriate)
            .wrap(
                middleware::Logger::new("%a \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %T")
                    .exclude("/metrics/stream")  // Don't log SSE connections
            )
            // Add compression middleware for better performance
            .wrap(middleware::Compress::default())
            // Register the metrics service
            .app_data(web::Data::new(metrics_service.clone()))
            // Configure all routes
            .configure(routes::configure)
            // Serve static files with caching headers
            .service(
                Files::new("/static", "./static")
                    .use_etag(true)
                    .use_last_modified(true)
            )
    })
        .bind(bind_addr)?
        .run()
        .await
}