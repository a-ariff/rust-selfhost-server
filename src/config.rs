//! Configuration module for database settings.
//!
//! This module handles loading configuration from environment variables.
//! In development, it uses dotenvy to load from .env files.

use anyhow::Result;
use std::time::Duration;

/// Database configuration settings
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub db_max_connections: u32,
    pub db_max_lifetime: Duration,
    pub db_idle_timeout: Duration,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// In development, this will attempt to load from .env files first.
    pub fn from_env() -> Result<Self> {
        // Load from .env file in development (will silently fail in production)
        #[cfg(debug_assertions)]
        let _ = dotenvy::dotenv();

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

        let db_max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|e| anyhow::anyhow!("Invalid DB_MAX_CONNECTIONS: {}", e))?;

        let db_max_lifetime = std::env::var("DB_MAX_LIFETIME")
            .unwrap_or_else(|_| "3600".to_string()) // 1 hour default
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid DB_MAX_LIFETIME: {}", e))
            .map(Duration::from_secs)?;

        let db_idle_timeout = std::env::var("DB_IDLE_TIMEOUT")
            .unwrap_or_else(|_| "600".to_string()) // 10 minutes default
            .parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid DB_IDLE_TIMEOUT: {}", e))
            .map(Duration::from_secs)?;

        Ok(Config {
            database_url,
            db_max_connections,
            db_max_lifetime,
            db_idle_timeout,
        })
    }

    /// Get the database URL
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    /// Get maximum database connections
    pub fn max_connections(&self) -> u32 {
        self.db_max_connections
    }

    /// Get database connection max lifetime
    pub fn max_lifetime(&self) -> Duration {
        self.db_max_lifetime
    }

    /// Get database connection idle timeout
    pub fn idle_timeout(&self) -> Duration {
        self.db_idle_timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        // Test that defaults are applied when env vars are not set
        std::env::remove_var("DB_MAX_CONNECTIONS");
        std::env::remove_var("DB_MAX_LIFETIME");
        std::env::remove_var("DB_IDLE_TIMEOUT");
        
        std::env::set_var("DATABASE_URL", "postgres://test:test@localhost/testdb");
        
        let config = Config::from_env().unwrap();
        assert_eq!(config.max_connections(), 10);
        assert_eq!(config.max_lifetime(), Duration::from_secs(3600));
        assert_eq!(config.idle_timeout(), Duration::from_secs(600));
        
        std::env::remove_var("DATABASE_URL");
    }
}
