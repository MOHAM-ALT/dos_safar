use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub system: SystemConfig,
    pub hardware: HardwareConfig,
    pub network: NetworkConfig,
    pub web: WebConfig,
    pub boot: BootConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub device_type: String,
    pub log_level: String,
    pub config_persist_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareConfig {
    pub test_timeout_seconds: u64,
    pub display_test_enabled: bool,
    pub input_test_enabled: bool,
    pub network_test_enabled: bool,
    pub auto_save_working_config: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub wifi_ssid: Option<String>,
    pub wifi_password: Option<String>,
    pub connection_timeout_seconds: u64,
    pub auto_connect: bool,
    pub ethernet_preferred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub host: String,
    pub enable_cors: bool,
    pub static_files_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootConfig {
    pub menu_timeout_seconds: u64,
    pub default_os: Option<String>,
    pub show_advanced_options: bool,
    pub gaming_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            system: SystemConfig {
                device_type: "auto".to_string(),
                log_level: "info".to_string(),
                config_persist_path: "/boot/dos_safar/".to_string(),
            },
            hardware: HardwareConfig {
                test_timeout_seconds: 30,
                display_test_enabled: true,
                input_test_enabled: true,
                network_test_enabled: true,
                auto_save_working_config: true,
            },
            network: NetworkConfig {
                wifi_ssid: None,
                wifi_password: None,
                connection_timeout_seconds: 3,
                auto_connect: true,
                ethernet_preferred: true,
            },
            web: WebConfig {
                port: 8080,
                host: "0.0.0.0".to_string(),
                enable_cors: true,
                static_files_path: "assets/web".to_string(),
            },
            boot: BootConfig {
                menu_timeout_seconds: 10,
                default_os: None,
                show_advanced_options: false,
                gaming_mode: true,
            },
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            // Create default config if it doesn't exist
            let default_config = Config::default();
            default_config.save(path)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        Ok(())
    }

    pub fn get_persistent_config_path(&self) -> &str {
        &self.system.config_persist_path
    }

    pub fn is_gaming_mode(&self) -> bool {
        self.boot.gaming_mode
    }

    pub fn get_web_url(&self) -> String {
        if self.web.host == "0.0.0.0" {
            format!("http://localhost:{}", self.web.port)
        } else {
            format!("http://{}:{}", self.web.host, self.web.port)
        }
    }
}