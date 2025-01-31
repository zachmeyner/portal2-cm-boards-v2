use config::ConfigError;
use serde::Deserialize;

/// Server hosting information for mounting the webserver.
#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
}
/// The proof standards, update based on the mod tools desired.
#[derive(Deserialize, Debug, Clone)]
pub struct ProofConfig {
    pub results: i32,
    pub demo: i32,
    pub video: i32,
}
#[derive(Deserialize, Debug, Clone)]
pub struct BackBlazeConfig {
    pub keyid: String,
    pub key: String,
    pub bucket: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server: ServerConfig,
    pub proof: ProofConfig,
    pub backblaze: BackBlazeConfig,
}
// Extracts the environment variables from .env
impl Config {
    /// The function fall that attempts to parse the `.env`
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        cfg.try_into()
    }
}
