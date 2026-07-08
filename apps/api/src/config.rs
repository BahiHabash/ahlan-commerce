#[derive(Debug, Clone)]
pub struct Config {
    pub api_bind_addr: String,
    pub env: String,
    pub database_url: String,
    pub redis_url: String,
}

pub const API_BIND_ADDR_ENV_KEY: &str = "API_BIND_ADDR";
pub const ENV_ENV_KEY: &str = "ENV";
pub const DATABASE_URL_ENV_KEY: &str = "DATABASE_URL";
pub const REDIS_URL_ENV_KEY: &str = "REDIS_URL";

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv(); // Load .env if present

        let api_bind_addr = std::env::var(API_BIND_ADDR_ENV_KEY)
            .expect("API_BIND_ADDR must be set");
        let env = std::env::var(ENV_ENV_KEY).unwrap_or_else(|_| "development".to_string());
        let database_url = std::env::var(DATABASE_URL_ENV_KEY)
            .expect("DATABASE_URL must be set");
        let redis_url = std::env::var(REDIS_URL_ENV_KEY)
            .expect("REDIS_URL must be set");
            
        Self { api_bind_addr, env, database_url, redis_url }
    }
}

