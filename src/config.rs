use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub ics: IcsConfig,
    pub google: GoogleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcsConfig {
    pub file_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                cache_ttl_seconds: 300,
            },
            ics: IcsConfig {
                file_paths: vec![],
            },
            google: GoogleConfig {
                client_id: None,
                client_secret: None,
                redirect_uri: None,
            },
        }
    }
}

impl Config {
    /// Load configuration from multiple sources in order of priority:
    /// 1. Command line arguments (highest priority)
    /// 2. Environment variables
    /// 3. Configuration file
    /// 4. Default values (lowest priority)
    pub fn load() -> Result<Self> {
        let mut config = Self::default();
        
        // Try to load from config file first
        if let Ok(file_config) = Self::load_from_file() {
            config = file_config;
        }
        
        // Override with environment variables
        config.apply_env_vars()?;
        
        // Validate required fields
        config.validate()?;
        
        Ok(config)
    }
    
    /// Load configuration from file
    /// Searches in order: ./calendar-monitor.toml, ~/.config/calendar-monitor/config.toml, /etc/calendar-monitor/config.toml
    fn load_from_file() -> Result<Self> {
        let mut possible_paths = vec![
            Some(PathBuf::from("./calendar-monitor.toml")),
        ];
        
        // Add user config directory if available
        if let Some(config_dir) = dirs::config_dir() {
            possible_paths.push(Some(config_dir.join("calendar-monitor").join("config.toml")));
        }
        
        // Add system config directory
        possible_paths.push(Some(PathBuf::from("/etc/calendar-monitor/config.toml")));
        
        for path_opt in possible_paths {
            if let Some(path) = path_opt {
                if path.exists() {
                    let contents = fs::read_to_string(&path)
                        .map_err(|e| anyhow!("Failed to read config file {}: {}", path.display(), e))?;
                    let config: Config = toml::from_str(&contents)
                        .map_err(|e| anyhow!("Failed to parse config file {}: {}", path.display(), e))?;
                    tracing::info!("Loaded configuration from: {}", path.display());
                    return Ok(config);
                }
            }
        }
        
        Err(anyhow!("No configuration file found"))
    }
    
    /// Apply environment variables to override config values
    pub fn apply_env_vars(&mut self) -> Result<()> {
        // Server configuration
        if let Ok(host) = env::var("CALENDAR_MONITOR_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("CALENDAR_MONITOR_PORT") {
            self.server.port = port.parse()
                .map_err(|e| anyhow!("Invalid CALENDAR_MONITOR_PORT: {}", e))?;
        }
        if let Ok(cache_ttl) = env::var("CALENDAR_MONITOR_CACHE_TTL") {
            self.server.cache_ttl_seconds = cache_ttl.parse()
                .map_err(|e| anyhow!("Invalid CALENDAR_MONITOR_CACHE_TTL: {}", e))?;
        }
        
        // ICS configuration
        if let Ok(ics_paths) = env::var("ICS_FILE_PATHS") {
            self.ics.file_paths = ics_paths
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        // Legacy support for single ICS file
        if let Ok(ics_path) = env::var("ICS_FILE_PATH") {
            if self.ics.file_paths.is_empty() {
                self.ics.file_paths.push(ics_path);
            }
        }
        
        // Google OAuth configuration
        if let Ok(client_id) = env::var("GOOGLE_CLIENT_ID") {
            self.google.client_id = Some(client_id);
        }
        if let Ok(client_secret) = env::var("GOOGLE_CLIENT_SECRET") {
            self.google.client_secret = Some(client_secret);
        }
        if let Ok(redirect_uri) = env::var("GOOGLE_REDIRECT_URI") {
            self.google.redirect_uri = Some(redirect_uri);
        }
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Only require ICS paths if there's no Google OAuth config either
        if self.ics.file_paths.is_empty() && self.google_oauth_config().is_none() {
            return Err(anyhow!("No ICS file paths or Google OAuth configured. Set ICS_FILE_PATHS environment variable or add paths to config file, or configure Google OAuth."));
        }
        
        // Validate Google OAuth config is complete or completely empty
        let google_fields = [
            &self.google.client_id,
            &self.google.client_secret,
            &self.google.redirect_uri,
        ];
        let filled_count = google_fields.iter().filter(|f| f.is_some()).count();
        
        if filled_count > 0 && filled_count < 3 {
            return Err(anyhow!(
                "Incomplete Google OAuth configuration. Either provide all three fields (GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET, GOOGLE_REDIRECT_URI) or none."
            ));
        }
        
        // Check for empty values in Google OAuth fields
        if let (Some(client_id), Some(client_secret), Some(redirect_uri)) = 
            (&self.google.client_id, &self.google.client_secret, &self.google.redirect_uri) {
            if client_id.trim().is_empty() {
                return Err(anyhow!("Google OAuth client_id cannot be empty"));
            }
            if client_secret.trim().is_empty() {
                return Err(anyhow!("Google OAuth client_secret cannot be empty"));
            }
            if redirect_uri.trim().is_empty() {
                return Err(anyhow!("Google OAuth redirect_uri cannot be empty"));
            }
            
            // Validate redirect_uri is a valid URL
            if !redirect_uri.starts_with("http://") && !redirect_uri.starts_with("https://") {
                return Err(anyhow!("Google OAuth redirect_uri must be a valid HTTP/HTTPS URL"));
            }
        }
        
        Ok(())
    }
    
    /// Create a sample configuration file
    pub fn create_sample_config() -> Result<String> {
        let sample_config = Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                cache_ttl_seconds: 300,
            },
            ics: IcsConfig {
                file_paths: vec![
                    "https://example.com/calendar.ics".to_string(),
                    "/path/to/local/calendar.ics".to_string(),
                ],
            },
            google: GoogleConfig {
                client_id: Some("your-google-client-id".to_string()),
                client_secret: Some("your-google-client-secret".to_string()),
                redirect_uri: Some("http://localhost:3000/auth/google/callback".to_string()),
            },
        };
        
        toml::to_string_pretty(&sample_config)
            .map_err(|e| anyhow!("Failed to serialize sample config: {}", e))
    }
    
    /// Get Google OAuth configuration if available
    pub fn google_oauth_config(&self) -> Option<(String, String, String)> {
        match (&self.google.client_id, &self.google.client_secret, &self.google.redirect_uri) {
            (Some(id), Some(secret), Some(uri)) => Some((id.clone(), secret.clone(), uri.clone())),
            _ => None,
        }
    }
    
    /// Get server bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
