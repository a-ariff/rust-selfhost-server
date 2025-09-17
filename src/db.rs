//! Database connection and pool management.
//!
//! This module handles creating and configuring the PostgreSQL connection pool
//! using sqlx with the configuration from the config module.

use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

use crate::config::Config;

/// Database connection pool manager
#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool from configuration
    ///
    /// This creates a PostgreSQL connection pool with the settings
    /// specified in the Config struct.
    pub async fn new(config: &Config) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections())
            .max_lifetime(Some(config.max_lifetime()))
            .idle_timeout(Some(config.idle_timeout()))
            .acquire_timeout(Duration::from_secs(30))
            .connect(config.database_url())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create database pool: {}", e))?;

        tracing::info!(
            "Database pool created with {} max connections",
            config.max_connections()
        );

        Ok(Database { pool })
    }

    /// Get a reference to the underlying connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Test database connectivity
    ///
    /// Attempts to acquire a connection from the pool and execute a simple query
    /// to verify the database is accessible and responsive.
    pub async fn health_check(&self) -> Result<()> {
        let mut conn = self.pool
            .acquire()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to acquire database connection: {}", e))?;

        sqlx::query("SELECT 1")
            .execute(&mut *conn)
            .await
            .map_err(|e| anyhow::anyhow!("Database health check query failed: {}", e))?;

        tracing::debug!("Database health check passed");
        Ok(())
    }

    /// Get connection pool statistics
    ///
    /// Returns information about the current state of the connection pool
    /// including active connections, idle connections, and pool capacity.
    pub fn pool_info(&self) -> PoolInfo {
        PoolInfo {
            size: self.pool.size(),
            num_idle: self.pool.num_idle(),
            is_closed: self.pool.is_closed(),
        }
    }

    /// Gracefully close the database pool
    ///
    /// This closes all connections in the pool and prevents new connections
    /// from being created. Useful for application shutdown.
    pub async fn close(&self) {
        self.pool.close().await;
        tracing::info!("Database pool closed");
    }
}

/// Information about the current state of the database pool
#[derive(Debug, Clone)]
pub struct PoolInfo {
    /// Total number of connections in the pool (active + idle)
    pub size: u32,
    /// Number of connections currently idle in the pool
    pub num_idle: usize,
    /// Whether the pool has been closed
    pub is_closed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    // Note: These tests require a running PostgreSQL instance
    // They are integration tests and may be skipped in CI without DB setup

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_database_creation() {
        std::env::set_var("DATABASE_URL", "postgres://test:test@localhost/testdb");

        let config = Config::from_env().unwrap();
        let db = Database::new(&config).await;

        assert!(db.is_ok());

        std::env::remove_var("DATABASE_URL");
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_health_check() {
        std::env::set_var("DATABASE_URL", "postgres://test:test@localhost/testdb");

        let config = Config::from_env().unwrap();
        let db = Database::new(&config).await.unwrap();

        let health = db.health_check().await;
        assert!(health.is_ok());

        std::env::remove_var("DATABASE_URL");
    }
}
