#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub env: String,
    pub database_url: String,
}

pub const PORT_ENV_KEY: &str = "PORT";
pub const ENV_ENV_KEY: &str = "ENV";
pub const DATABASE_URL_ENV_KEY: &str = "DATABASE_URL";

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv(); // Load .env if present

        let port = std::env::var(PORT_ENV_KEY)
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(3000);
        let env = std::env::var(ENV_ENV_KEY).unwrap_or_else(|_| "development".to_string());
        let database_url = std::env::var(DATABASE_URL_ENV_KEY)
            .unwrap_or_else(|_| "postgres://postgres@localhost:5432/ahlan_commerce".to_string());
            
        Self { port, env, database_url }
    }
}

