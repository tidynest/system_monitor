//! Metrics API endpoints
//!
//! Provides HTTP endpoints for system metrics retrieval including:
//! - Server-Sent Events (SSE) stream for real-time updates
//! - Process information endpoints
//! - Disk usage endpoints
//! - Network statistics endpoints

use actix_web::web::Bytes;
use actix_web::{web, HttpResponse, Result};
use actix_web::http::header::ContentEncoding;  // Add this import
use serde::Deserialize;
use std::time::Duration;
use tokio::time::interval;

pub use crate::services::MetricsServiceRef;
use crate::utils;

/// Query parameters for process endpoint
#[derive(Debug, Deserialize)]
pub struct ProcessQuery {
    /// Type of processes to retrieve: "memory" for top memory consumers, otherwise top CPU
    #[serde(default)]
    pub r#type: Option<String>,
}

/// SSE endpoint for streaming real-time metrics updates
///
/// Establishes a Server-Sent Events connection that pushes system metrics
/// to the client every second.
pub async fn metrics_stream(
    metrics_service: web::Data<MetricsServiceRef>,
) -> Result<HttpResponse> {
    let stream = async_stream::stream! {
        let mut interval = interval(Duration::from_secs(1));

        // IMPORTANT: Skip the first tick to avoid immediate send
        interval.tick().await;

        log::info!("SSE client connected, starting metrics stream");

        loop {
            interval.tick().await;

            // Collect metrics using the service
            let metrics = metrics_service.collect();

            // Create the SSE update with proper formatting
            match format_sse_update(&metrics) {
                Ok(update) => {
                    log::debug!("Sending SSE update: {} bytes", update.len());
                    yield Ok::<_, actix_web::Error>(Bytes::from(update));
                }
                Err(e) => {
                    log::error!("Failed to format SSE update: {}", e);
                    yield Ok::<_, actix_web::Error>(Bytes::from(": heartbeat\n\n"));
                }
            }
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream; charset=utf-8")
        .insert_header(("Cache-Control", "no-cache, no-transform"))
        .insert_header(("X-Accel-Buffering", "no"))
        .insert_header(("Connection", "keep-alive"))
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(ContentEncoding::Identity)  // CRITICAL: Bypass compression middleware
        .streaming(Box::pin(stream)))
}

/// Get top processes by CPU or memory usage
///
/// Returns an HTML fragment containing the top 5 processes sorted by
/// either CPU usage (default) or memory usage (if type=memory).
pub async fn get_processes(
    metrics_service: web::Data<MetricsServiceRef>,
    query: web::Query<ProcessQuery>,
) -> Result<HttpResponse> {
    let metrics = metrics_service.collect();
    let show_memory = matches!(query.r#type.as_deref(), Some("memory"));

    let processes = if show_memory {
        &metrics.process.top_memory
    } else {
        &metrics.process.top_cpu
    };

    let html = if processes.is_empty() {
        r#"<div class="loading">No active processes</div>"#.to_string()
    } else {
        render_process_list(processes, show_memory)
    };

    Ok(html_response(html))
}

/// Get disk usage information
///
/// Returns an HTML fragment containing disk usage statistics for
/// up to 3 mounted filesystems.
pub async fn get_disks(metrics_service: web::Data<MetricsServiceRef>) -> Result<HttpResponse> {
    let metrics = metrics_service.collect();

    let html = if metrics.disk.is_empty() {
        r#"<div class="loading">No disks found</div>"#.to_string()
    } else {
        render_disk_list(&metrics.disk)
    };

    Ok(html_response(html))
}

/// Get network interface statistics
///
/// Returns an HTML fragment containing total network traffic and
/// active interface count.
pub async fn get_network(metrics_service: web::Data<MetricsServiceRef>) -> Result<HttpResponse> {
    let metrics = metrics_service.collect();
    let html = render_network_stats(&metrics.network);
    Ok(html_response(html))
}

// --- Helper functions ---

/// Create an HTML response with appropriate headers
fn html_response(body: impl Into<String>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body.into())
}

/// Render process list as HTML
fn render_process_list(
    processes: &[crate::models::process::ProcessInfo],
    show_memory: bool,
) -> String {
    let mut html = String::with_capacity(1024);

    for process in processes.iter().take(5) {
        let value = if show_memory {
            format!("{:.1} MB", process.memory_mb)
        } else {
            format!("{:.1}%", process.cpu_usage)
        };

        html.push_str(&format!(
            r#"<div class="process-row">
                <span>{}</span>
                <span style="color: #667eea; font-weight: bold;">{}</span>
            </div>"#,
            html_escape(&process.name),
            value
        ));
    }

    html
}

/// Render disk list as HTML
fn render_disk_list(disks: &[crate::models::disk::DiskMetrics]) -> String {
    let mut html = String::with_capacity(2048);

    for disk in disks.iter().take(3) {
        let used_gb = disk.total_gb - disk.available_gb;

        html.push_str(&format!(
            r#"<div class="disk-item">
                <div style="display: flex; justify-content: space-between; margin-bottom: 5px">
                    <span>{}</span>
                    <span>{:.1}%</span>
                </div>
                <div class="progress-bar" style="height: 10px;">
                    <div class="progress-fill" style="width: {:.1}%"></div>
                </div>
                <div style="font-size: 0.8rem; color: #666; margin-top: 5px;">
                    {:.1} GB / {:.1} GB
                </div>
            </div>"#,
            html_escape(&disk.mount_point),
            disk.usage_percent,
            disk.usage_percent,
            used_gb,
            disk.total_gb,
        ));
    }

    html
}

/// Render network statistics as HTML
fn render_network_stats(network: &crate::models::network::NetworkMetrics) -> String {
    format!(
        r#"<div class="network-item">
            <div style="margin-bottom: 10px;">
                <strong>Total Traffic</strong>
            </div>
            <div style="display: flex; justify-content: space-between;">
                <span>↓ Received:</span>
                <span style="color: #667eea; font-weight: bold;">{:.1} MB</span>
            </div>
            <div style="display:flex; justify-content: space-between;">
                <span>↑ Transmitted:</span>
                <span style="color: #764ba2; font-weight: bold;">{:.1} MB</span>
            </div>
        </div>
        <div style="margin-top: 10px; font-size: 0.8rem; color: #666;">
            Active Interfaces: {}
        </div>"#,
        network.total_received_mb,
        network.total_transmitted_mb,
        network.interfaces.len()
    )
}

/// Format metrics as Server-Sent Event data
/// CRITICAL: Each SSE message MUST end with two newlines (\n\n) to be valid
fn format_sse_update(
    metrics: &crate::models::system::SystemMetrics,
) -> Result<String, serde_json::Error> {
    let data = serde_json::json!({
        "cpu_usage": format!("{:.1}%", metrics.cpu.usage_percent),
        "cpu_percent": metrics.cpu.usage_percent,
        "cpu_cores": metrics.cpu.core_count,
        "cpu_freq": metrics.cpu.frequency,
        "memory_used": metrics.memory.used_gb,
        "memory_total": metrics.memory.total_gb,
        "memory_percent": metrics.memory.usage_percent,
        "memory_available": metrics.memory.available_gb,
        "swap_used": metrics.memory.swap_used_gb,
        "swap_total": metrics.memory.swap_total_gb,
        "timestamp": &metrics.timestamp,
        "uptime": utils::format_uptime(metrics.uptime),
    });

    // IMPORTANT: SSE format requires:
    // 1. "data: " prefix
    // 2. The actual data
    // 3. Two newlines to terminate the message
    let json_string = serde_json::to_string(&data)?;

    // Explicitly format with double newline for proper SSE termination
    Ok(format!("data: {}\n\n", json_string))
}

/// Basic HTML escape for user-provided content
fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}