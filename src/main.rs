use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tracing_subscriber;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
}

async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

async fn root_handler() -> Html<&'static str> {
    Html(r#"
    <html>
    <head><title>Rust Self-Host Server</title></head>
    <body style="font-family: Arial, sans-serif; margin: 2rem; background: #f5f5f5;">
        <div style="max-width: 800px; margin: 0 auto; background: white; padding: 2rem; border-radius: 8px;">
            <h1>🚀 Rust Self-Host Server</h1>
            <div style="background: #d4edda; color: #155724; padding: 1rem; border-radius: 4px; margin: 1rem 0;">
                ✅ Server is running successfully!
            </div>
            <h2>Available Endpoints:</h2>
            <ul>
                <li><strong>GET /</strong> - This welcome page</li>
                <li><strong>GET /health</strong> - Health check endpoint</li>
            </ul>
            <p style="text-align: center; color: #666;">Built with ❤️ using Rust and Axum</p>
        </div>
    </body>
    </html>
    "#)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("Received terminate signal");
        },
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::init();

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive());

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Server starting on http://0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    info!("✅ Server is ready to accept connections");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("🛑 Server shutdown complete");
}
