// src/main.rs

use actix_web::{web, App, HttpResponse, HttpServer, Result, middleware};
use actix_files::Files;
use tokio::time::interval;
use std::time::Duration;
use futures::stream::StreamExt;

mod collectors;
mod config;
mod models;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment logger for better debugging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Use Config module
    let config = config::Config::from_env();

    // Print initial startup messages
    println!("System Monitor Dashboard (HTMX Edition)");
    println!("Starting server on http://127.0.0.1:8080");
    println!("🌐 Open your browser to view the dashboard");
    println!("Press Ctrl+C to stop\n");

    let bind_addr = config.bind_address();

    HttpServer::new(|| {
        App::new()
            // Add logging middleware for debugging
            .wrap(middleware::Logger::default())
            // Serve the main dashboard page
            .route("/", web::get().to(dashboard_page))
            // SSE endpoint for real-time metrics
            .route("/metrics/stream", web::get().to(metrics_stream))
            // Individual metric endpoints for HTMX updates
            .route("/metrics/processes", web::get().to(get_processes))
            .route("/metrics/disks", web::get().to(get_disks))
            .route("/metrics/network", web::get().to(get_network))
            // Serve CSS and any remaining static assets
            .service(Files::new("/static", "./static"))
    })
        .bind(bind_addr)?
        .run()
        .await
}

// Serves the main dashboard HTML page
async fn dashboard_page() -> Result<HttpResponse> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>System Monitor</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <script src="https://unpkg.com/htmx.org@1.9.10/dist/ext/sse.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
        }

        .header {
            text-align: center;
            color: white;
            margin-bottom: 30px;
        }

        .header h1 {
            font-size: 2.5rem;
            margin-bottom: 10px;
        }

        .status-bar {
            display: flex;
            justify-content: space-between;
            color: rgba(255, 255, 255, 0.9);
            margin-bottom: 20px;
            padding: 10px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 8px;
        }

        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }

        .metric-card {
            background: rgba(255, 255, 255, 0.95);
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
            transition: transform 0.2s;
        }

        .metric-card:hover {
            transform: translateY(-5px);
        }

        .metric-header {
            display: flex;
            align-items: center;
            margin-bottom: 15px;
        }

        .metric-icon {
            font-size: 1.5rem;
            margin-right: 10px;
        }

        .metric-title {
            font-size: 1.2rem;
            font-weight: 600;
            color: #333;
        }

        .metric-value {
            font-size: 2rem;
            font-weight: bold;
            color: #667eea;
            margin-bottom: 10px;
        }

        .progress-bar {
            width: 100%;
            height: 20px;
            background: #e0e0e0;
            border-radius: 10px;
            overflow: hidden;
            position: relative;
        }

        .progress-fill {
            height: 100%;
            background: linear-gradient(90deg, #667eea, #764ba2);
            border-radius: 10px;
            transition: width 0.3s ease;
        }

        .process-table {
            background: rgba(255, 255, 255, 0.95);
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
            margin-bottom: 20px;
        }

        .process-row {
            display: flex;
            justify-content: space-between;
            padding: 8px 0;
            border-bottom: 1px solid #e0e0e0;
        }

        .process-row:last-child {
            border-bottom: none;
        }

        .loading {
            color: #999;
            font-style: italic;
        }

        .grid-container {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
        }

        .disk-item, .network-item {
            background: #f5f5f5;
            padding: 10px;
            border-radius: 8px;
            margin-bottom: 10px;
        }

        .pulse {
            animation: pulse 2s infinite;
        }

        @keyframes pulse {
            0% { opacity: 1; }
            50% { opacity: 0.5; }
            100% { opacity: 1; }
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>System Monitor</h1>
        </div>

        <div class="status-bar">
            <div id="connection-status">
                <span class="pulse">⚡</span> Connecting...
            </div>
            <div id="timestamp">Waiting for data...</div>
            <div>Uptime: <span id="uptime">--</span></div>
        </div>

        <!-- Metrics Grid - No SSE wrapper here, we'll handle updates differently -->
        <div class="metrics-grid">
            <!-- CPU Usage Card -->
            <div class="metric-card">
                <div class="metric-header">
                    <span class="metric-icon">⚡</span>
                    <span class="metric-title">CPU Usage</span>
                </div>
                <div class="metric-value" id="cpu-value">--%</div>
                <div class="progress-bar">
                    <div id="cpu-bar" class="progress-fill" style="width: 0%"></div>
                </div>
                <div id="cpu-details" style="margin-top: 10px; font-size: 0.9rem; color: #666;">
                    Cores: <span id="cpu-cores">--</span> |
                    Freq: <span id="cpu-freq">--</span> MHz
                </div>
            </div>

            <!-- Memory Card -->
            <div class="metric-card">
                <div class="metric-header">
                    <span class="metric-icon">💾</span>
                    <span class="metric-title">Memory</span>
                </div>
                <div class="metric-value" id="memory-value">-- GB</div>
                <div class="progress-bar">
                    <div id="memory-bar" class="progress-fill" style="width: 0%"></div>
                </div>
                <div id="memory-details" style="margin-top: 10px; font-size: 0.9rem; color: #666;">
                    Available: <span id="memory-available">--</span> GB |
                    Swap: <span id="swap-usage">--</span> GB
                </div>
            </div>

            <!-- Network Card -->
            <div class="metric-card">
                <div class="metric-header">
                    <span class="metric-icon">🌐</span>
                    <span class="metric-title">Network</span>
                </div>
                <div id="network-info" hx-get="/metrics/network"
                     hx-trigger="load, every 2s"
                     hx-swap="innerHTML">
                    <div class="loading">Loading...</div>
                </div>
            </div>

            <!-- Disk Usage Card -->
            <div class="metric-card">
                <div class="metric-header">
                    <span class="metric-icon">💿</span>
                    <span class="metric-title">Disk Usage</span>
                </div>
                <div id="disk-info" hx-get="/metrics/disks"
                     hx-trigger="load, every 5s"
                     hx-swap="innerHTML">
                    <div class="loading">Loading...</div>
                </div>
            </div>
        </div>

        <!-- Process Tables -->
        <div class="grid-container">
            <div class="process-table">
                <h3 style="margin-bottom: 15px;">Top CPU Processes</h3>
                <div id="cpu-processes" hx-get="/metrics/processes"
                     hx-trigger="load, every 2s"
                     hx-swap="innerHTML">
                    <div class="loading">Loading...</div>
                </div>
            </div>

            <div class="process-table">
                <h3 style="margin-bottom: 15px;">Top Memory Processes</h3>
                <div id="memory-processes" hx-get="/metrics/processes?type=memory"
                     hx-trigger="load, every 2s"
                     hx-swap="innerHTML">
                    <div class="loading">Loading...</div>
                </div>
            </div>
        </div>
    </div>

    <!-- SSE connection handled separately to avoid wrapping issues -->
    <div id="sse-container" hx-ext="sse" sse-connect="/metrics/stream" sse-swap="message" style="display: none;"></div>

    <script>
        // Manual SSE handling for better control
        const evtSource = new EventSource('/metrics/stream');

        evtSource.onopen = function(e) {
            document.getElementById('connection-status').innerHTML = '<span class="pulse">⚡</span> Connected';
        };

        evtSource.onerror = function(e) {
            document.getElementById('connection-status').innerHTML = '<span style="color: red;">⚠️</span> Reconnecting...';
        };

        evtSource.onmessage = function(e) {
            // Parse the SSE data and update elements manually
            const lines = e.data.split('\n');
            lines.forEach(line => {
                if (line.includes('hx-swap-oob')) {
                    // Extract the element and update it
                    const parser = new DOMParser();
                    const doc = parser.parseFromString(line, 'text/html');
                    const element = doc.body.firstChild;
                    if (element && element.id) {
                        const target = document.getElementById(element.id);
                        if (target) {
                            if (element.id.includes('-bar')) {
                                // For progress bars, update the style
                                target.style.width = element.style.width;
                            } else {
                                // For other elements, update innerHTML
                                target.innerHTML = element.innerHTML;
                            }
                        }
                    }
                }
            });
        };
    </script>
</body>
</html>"#;

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

// SSE endpoint that streams metric updates
async fn metrics_stream() -> Result<HttpResponse> {
    use actix_web::web::Bytes;

    let stream = async_stream::stream! {
        let mut interval = interval(Duration::from_secs(1));

        // Send initial connection event
        yield Ok::<_, actix_web::Error>(Bytes::from("event: connected\ndata: connected\n\n"));

        loop {
            interval.tick().await;

            // Collect metrics using collectors
            let metrics = collectors::system::collect_system_metrics();

            // Create the SSE update with proper formatting
            let update = format_sse_update(&metrics);

            // Yield the formatted SSE data
            yield Ok::<_, actix_web::Error>(Bytes::from(update));
        }
    };

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))  // Disable Nginx buffering
        .streaming(Box::pin(stream)))
}

// Helper function to format metrics as SSE with HTMX swaps
fn format_sse_update(metrics: &models::system::SystemMetrics) -> String {
    // Build the SSE message with proper formatting
    // Each element gets its own update with hx-swap-oob
    let mut update = String::new();

    // Start the SSE event
    update.push_str("event: message\n");

    // CPU updates
    update.push_str(&format!(
        "data: <div id=\"cpu-value\" hx-swap-oob=\"innerHTML\">{:.1}%</div>\n",
        metrics.cpu.usage_percent
    ));

    update.push_str(&format!(
        "data: <div id=\"cpu-bar\" hx-swap-oob=\"outerHTML\"><div id=\"cpu-bar\" class=\"progress-fill\" style=\"width: {:.1}%\"></div></div>\n",
        metrics.cpu.usage_percent
    ));

    update.push_str(&format!(
        "data: <span id=\"cpu-cores\" hx-swap-oob=\"innerHTML\">{}</span>\n",
        metrics.cpu.core_count
    ));

    update.push_str(&format!(
        "data: <span id=\"cpu-freq\" hx-swap-oob=\"innerHTML\">{}</span>\n",
        metrics.cpu.frequency
    ));

    // Memory updates
    update.push_str(&format!(
        "data: <div id=\"memory-value\" hx-swap-oob=\"innerHTML\">{:.1} / {:.1} GB</div>\n",
        metrics.memory.used_gb,
        metrics.memory.total_gb
    ));

    update.push_str(&format!(
        "data: <div id=\"memory-bar\" hx-swap-oob=\"outerHTML\"><div id=\"memory-bar\" class=\"progress-fill\" style=\"width: {:.1}%\"></div></div>\n",
        metrics.memory.usage_percent
    ));

    update.push_str(&format!(
        "data: <span id=\"memory-available\" hx-swap-oob=\"innerHTML\">{:.1}</span>\n",
        metrics.memory.available_gb
    ));

    update.push_str(&format!(
        "data: <span id=\"swap-usage\" hx-swap-oob=\"innerHTML\">{:.1}/{:.1}</span>\n",
        metrics.memory.swap_used_gb,
        metrics.memory.swap_total_gb
    ));

    // System info updates
    update.push_str(&format!(
        "data: <div id=\"timestamp\" hx-swap-oob=\"innerHTML\">{}</div>\n",
        metrics.timestamp
    ));

    update.push_str(&format!(
        "data: <span id=\"uptime\" hx-swap-oob=\"innerHTML\">{}</span>\n",
        utils::format_uptime(metrics.uptime)
    ));

    // End the SSE message with double newline
    update.push_str("\n");

    update
}

// Endpoint for process list updates
async fn get_processes(req: actix_web::HttpRequest) -> Result<HttpResponse> {
    let metrics = collectors::system::collect_system_metrics();

    // Check query parameter to determine which processes to show
    let query_string = req.query_string();
    let show_memory = query_string.contains("type=memory");

    let mut html = String::new();

    let processes = if show_memory {
        &metrics.process.top_memory
    } else {
        &metrics.process.top_cpu
    };

    // Display the appropriate processes
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
            process.name, value
        ));
    }

    // If no processes found
    if html.is_empty() {
        html.push_str(r#"<div class="loading">No active processes</div>"#);
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html))
}

// New endpoint for disk information
async fn get_disks() -> Result<HttpResponse> {
    let metrics = collectors::system::collect_system_metrics();
    let mut html = String::new();

    for disk in metrics.disk.iter().take(3) {
        html.push_str(&format!(
            r#"<div class="disk-item">
                <div style="display: flex; justify-content: space-between; margin-bottom: 5px;">
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
            disk.mount_point,
            disk.usage_percent,
            disk.usage_percent,
            disk.total_gb - disk.available_gb,
            disk.total_gb
        ));
    }

    if html.is_empty() {
        html.push_str(r#"<div class="loading">No disks found</div>"#);
    }

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html))
}

// New endpoint for network information
async fn get_network() -> Result<HttpResponse> {
    let metrics = collectors::system::collect_system_metrics();

    let html = format!(
        r#"<div class="network-item">
            <div style="margin-bottom: 10px;">
                <strong>Total Traffic</strong>
            </div>
            <div style="display: flex; justify-content: space-between;">
                <span>↓ Received:</span>
                <span style="color: #667eea; font-weight: bold;">{:.1} MB</span>
            </div>
            <div style="display: flex; justify-content: space-between;">
                <span>↑ Transmitted:</span>
                <span style="color: #764ba2; font-weight: bold;">{:.1} MB</span>
            </div>
        </div>
        <div style="margin-top: 10px; font-size: 0.8rem; color: #666;">
            Active Interfaces: {}
        </div>"#,
        metrics.network.total_received_mb,
        metrics.network.total_transmitted_mb,
        metrics.network.interfaces.len()
    );

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html))
}