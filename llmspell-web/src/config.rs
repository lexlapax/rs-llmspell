use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct WebConfig {
    pub port: u16,
    pub host: String,
    pub cors_origins: Vec<String>,
    pub auth_secret: String,
    pub api_keys: Vec<String>,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "127.0.0.1".to_string(),
            cors_origins: vec!["http://localhost:3000".to_string()],
            auth_secret: "dev_secret_do_not_use_in_prod".to_string(),
            api_keys: vec!["dev-key-123".to_string()],
        }
    }
}
