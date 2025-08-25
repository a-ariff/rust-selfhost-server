use axum::{
    extract::Json,
    http::StatusCode,
    response::Json as ResponseJson,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
struct InfoResponse {
    service: String,
    version: String,
    description: String,
    endpoints: Vec<String>,
}

// Health check endpoint
async fn healthz() -> Result<ResponseJson<HealthResponse>, StatusCode> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        service: "rust-selfhost-server".to_string(),
        version: "0.1.0".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    info!("Health check requested");
    Ok(ResponseJson(response))
}

// Root endpoint with service info
async fn root() -> Result<ResponseJson<InfoResponse>, StatusCode> {
    let response = InfoResponse {
        service: "rust-selfhost-server".to_string(),
        version: "0.1.0".to_string(),
        description: "A Rust Axum-based HTTP server with Docker Build Cloud CI/CD integration".to_string(),
        endpoints: vec![
            "/".to_string(),
            "/healthz".to_string(),
        ],
    };
    
    info!("Root endpoint requested");
    Ok(ResponseJson(response))
}

// 404 handler
async fn handler_404() -> Result<ResponseJson<HashMap<String, String>>, StatusCode> {
    let mut response = HashMap::new();
    response.insert("error".to_string(), "Not Found".to_string());
    response.insert("message".to_string(), "The requested resource was not found".to_string());
    
    Ok(ResponseJson(response))
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_selfhost_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .fallback(handler_404)
        .layer(TraceLayer::new_for_http());

    // Get port from environment or default to 3000
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    let addr = format!("0.0.0.0:{}", port);
    info!("Starting server on {}", addr);

    // Create TCP listener
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    info!("Server listening on http://{}", addr);
    info!("Health check available at http://{}/healthz", addr);

    // Start the server
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
