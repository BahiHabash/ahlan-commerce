#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub env: String,
}

pub const PORT_ENV_KEY: &str = "PORT";
pub const ENV_ENV_KEY: &str = "ENV";

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv(); // Load .env if present

        let port = std::env::var(PORT_ENV_KEY)
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(3000);
        let env = std::env::var(ENV_ENV_KEY).unwrap_or_else(|_| "development".to_string());
        Self { port, env }
    }
}

