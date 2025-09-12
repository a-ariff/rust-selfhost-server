use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};
use tracing_subscriber;

// Import our modules
mod config;
mod db;

use config::Config;
use db::Database;

// Application state containing shared resources
#[derive(Clone)]
struct AppState {
    db: Database,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
}

#[derive(Serialize)]
struct DatabaseHealthResponse {
    status: String,
    timestamp: String,
    database: String,
    pool_size: u32,
    idle_connections: usize,
}

// Basic health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

// Database health check endpoint
async fn db_health_check(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.health_check().await {
        Ok(_) => {
            let pool_info = state.db.pool_info();
            let response = DatabaseHealthResponse {
                status: "healthy".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                database: "connected".to_string(),
                pool_size: pool_info.size,
                idle_connections: pool_info.num_idle,
            };
            (StatusCode::OK, Json(response))
        },
        Err(e) => {
            error!("Database health check failed: {}", e);
            let response = DatabaseHealthResponse {
                status: "unhealthy".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                database: "disconnected".to_string(),
                pool_size: 0,
                idle_connections: 0,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response))
        }
    }
}

async fn root_handler() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Rust Self-Host Server</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        </style>
    </head>
    <body>
        <div style="max-width: 800px; margin: 0 auto; background: white; padding: 2rem; border-radius: 8px;">
            <h1>🚀 Rust Self-Host Server</h1>
            <div style="background: #d4edda; color: #155724; padding: 1rem; border-radius: 4px; margin: 1rem 0;">
                ✅ Server is running successfully!
            </div>
            <h3>Available Endpoints:</h3>
            <ul>
                <li><strong>GET /</strong> - This welcome page</li>
                <li><strong>GET /health</strong> - Basic health check endpoint</li>
                <li><strong>GET /health/db</strong> - Database health check with connection pool info</li>
            </ul>
            <p style="text-align: center; color: #666;">Built with ❤️ using Rust, Axum, and PostgreSQL</p>
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
    // Initialize tracing
    tracing_subscriber::init();

    info!("🔧 Loading configuration...");
    let config = match Config::from_env() {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            config
        },
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    info!("🗄️  Initializing database connection...");
    let database = match Database::new(&config).await {
        Ok(db) => {
            info!("✅ Database connection established");
            db
        },
        Err(e) => {
            error!("❌ Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // Create application state
    let app_state = AppState {
        db: database,
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
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    info!("🛑 Server shutdown complete");
}
