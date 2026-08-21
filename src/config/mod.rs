// src/config/mod.rs
//
// WHY THIS EXISTS:
//   Configuration should be centralized, not scattered across the codebase.
//   At scale, you will have 10+ config values. Loading them in main.rs creates clutter.
//   This module owns "how does the app know what to connect to?"
//
// WHAT IT DOES:
//   Reads environment variables at startup and panics fast if critical ones are missing.
//   Panic-on-startup is intentional — it is better to crash immediately than to
//   fail mysteriously after accepting connections.

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub app_port: u16,
    pub app_env: String,
}

impl AppConfig {
    /// Load configuration from environment variables.
    ///
    /// Call this once at startup, before creating any connections.
    /// The result is cloned into AppState and shared across all request handlers.
    pub fn from_env() -> Self {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in environment");

        let app_port = std::env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("APP_PORT must be a valid port number (1-65535)");

        let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        AppConfig {
            database_url,
            app_port,
            app_env,
        }
    }

    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }
}
