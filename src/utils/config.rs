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
    pub lcd: LcdConfig, // إضافة جديدة
}
// إضافة في نهاية Default::default()
lcd: LcdConfig {
    enabled: true,
    auto_detect: true,
    driver: "auto".to_string(),
    interface: "spi".to_string(),
    size_inch: 3.5,
    rotation: 0,
    spi_bus: 0,
    spi_device: 0,
    spi_speed_hz: 32000000,
    gpio_cs: Some(8),
    gpio_dc: Some(24),
    gpio_rst: Some(25),
    gpio_bl: Some(18),
    touch_enabled: true,
    touch_device: "/dev/input/touchscreen".to_string(),
    calibration_matrix: vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
},

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
    // Enhanced network features
    pub backup_networks: Vec<BackupNetwork>,
    pub auto_scan_open_networks: bool,
    pub prefer_saved_networks: bool,
    pub max_connection_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupNetwork {
    pub ssid: String,
    pub password: String,
}
// إضافة بعد WebConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LcdConfig {
    pub enabled: bool,
    pub auto_detect: bool,
    pub driver: String,
    pub interface: String,
    pub size_inch: f32,
    pub rotation: u32,
    pub spi_bus: u8,
    pub spi_device: u8,
    pub spi_speed_hz: u32,
    pub gpio_cs: Option<u8>,
    pub gpio_dc: Option<u8>,
    pub gpio_rst: Option<u8>,
    pub gpio_bl: Option<u8>,
    pub touch_enabled: bool,
    pub touch_device: String,
    pub calibration_matrix: Vec<f32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub host: String,
    pub enable_cors: bool,
    pub static_files_path: String,
    // Enhanced web features
    pub auto_launch_interface: bool,
    pub show_qr_code: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootConfig {
    pub menu_timeout_seconds: u64,
    pub default_os: Option<String>,
    pub show_advanced_options: bool,
    pub gaming_mode: bool,
    // Enhanced boot features
    pub auto_web_on_timeout: bool,
    pub keyboard_interrupt_enabled: bool,
    pub show_ip_on_screen: bool,
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
                wifi_ssid: Some("YourHomeWiFi".to_string()),
                wifi_password: Some("YourWiFiPassword".to_string()),
                connection_timeout_seconds: 3,
                auto_connect: true,
                ethernet_preferred: false,
                backup_networks: vec![
                    BackupNetwork {
                        ssid: "YourPhoneHotspot".to_string(),
                        password: "hotspot123".to_string(),
                    },
                    BackupNetwork {
                        ssid: "GuestWiFi".to_string(),
                        password: "".to_string(), // Open network
                    },
                ],
                auto_scan_open_networks: true,
                prefer_saved_networks: true,
                max_connection_attempts: 3,
            },
            web: WebConfig {
                port: 8080,
                host: "0.0.0.0".to_string(),
                enable_cors: true,
                static_files_path: "assets/web".to_string(),
                auto_launch_interface: true,
                show_qr_code: true,
            },
            boot: BootConfig {
                menu_timeout_seconds: 3, // 3 seconds only
                default_os: None,
                show_advanced_options: false,
                gaming_mode: true,
                auto_web_on_timeout: true,
                keyboard_interrupt_enabled: true,
                show_ip_on_screen: true,
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