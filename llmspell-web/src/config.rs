use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct WebConfig {
    pub port: u16,
    pub host: String,
    pub cors_origins: Vec<String>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
            cors_origins: vec!["http://localhost:3000".to_string()],
        }
    }
}
