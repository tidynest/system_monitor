//! Dashboard route handlers
//!
//! Serves the main HTML dashboard interface.

use actix_web::{HttpResponse, Result};

/// Serves the main dashboard HTML page
///
/// Returns the dashboard.html template with appropriate content type headers.
pub async fn dashboard_page() -> Result<HttpResponse> {
    // Embed the dashboard HTML at compile time for better performance
    const DASHBOARD_HTML: &str = include_str!("../../static/html/dashboard.html");

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(DASHBOARD_HTML))
}