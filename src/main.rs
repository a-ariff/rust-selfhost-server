use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
mod config;
mod db;
use config::Config;
use db::Database;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt().init();
    info!("🔧 Loading configuration...");
    let config = match Config::from_env() {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };
    info!("🗄️ Initializing database connection...");
    let database = match Database::new(&config).await {
        Ok(db) => {
            info!("✅ Database connection established");
            db
        }
        Err(e) => {
            error!("❌ Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };
    // Create application state
    let app_state = AppState {
        db: database.clone(),
    };
    // Build our application with routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_check))
        .route("/health/db", get(db_health_check))
        .layer(CorsLayer::permissive())
        .with_state(app_state);
    // Get port from environment or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid number");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("🚀 Server starting on http://0.0.0.0:{}", port);
    // Create listener
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    info!("✅ Server is ready to accept connections");
    // Start server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(database))
        .await
        .unwrap();
    info!("🛑 Server shutdown complete");
}

async fn root_handler() -> Json<Value> {
    Json(json!({
        "message": "Rust Self-Host Server",
        "status": "running"
    }))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn db_health_check(State(state): State<AppState>) -> StatusCode {
    match state.db.health_check().await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

async fn shutdown_signal(db: Database) {
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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("🛑 Shutdown signal received");
    db.close().await;
    info!("✅ Pending processes have been cancelled");
}
