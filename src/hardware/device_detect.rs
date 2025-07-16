use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_type: DeviceType,
    pub model: String,
    pub architecture: String,
    pub cpu_info: CpuInfo,
    pub memory_mb: u64,
    pub has_gpio: bool,
    pub has_camera: bool,
    pub display_type: DisplayType,
    pub gaming_features: GamingFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    RaspberryPi,
    Anbernic,
    OrangePi,
    BananaPi,
    RockPi,
    Odroid,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: u32,
    pub frequency_mhz: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayType {
    HDMI,
    DSI,
    LCD,
    OLED,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingFeatures {
    pub has_dpad: bool,
    pub has_analog_sticks: bool,
    pub has_shoulder_buttons: bool,
    pub has_built_in_screen: bool,
    pub has_battery: bool,
    pub screen_size_inches: Option<f32>,
    pub native_resolution: Option<(u32, u32)>,
}

pub struct DeviceDetector;

impl DeviceDetector {
    pub fn new() -> Self {
        DeviceDetector
    }

    pub async fn detect_device(&self) -> Result<DeviceInfo> {
        info!("Starting device detection...");

        // Get basic system information
        let architecture = self.get_architecture()?;
        let memory_mb = self.get_memory_mb()?;
        
        // Detect device type
        let device_type = self.detect_device_type().await?;
        let model = self.get_device_model(&device_type)?;
        
        // Get CPU information
        let cpu_info = self.get_cpu_info()?;
        
        // Detect hardware features
        let has_gpio = self.has_gpio_support(&device_type);
        let has_camera = self.detect_camera().await;
        let display_type = self.detect_display_type(&device_type).await;
        let gaming_features = self.detect_gaming_features(&device_type).await;

        let device_info = DeviceInfo {
            device_type,
            model,
            architecture,
            cpu_info,
            memory_mb,
            has_gpio,
            has_camera,
            display_type,
            gaming_features,
        };

        info!("Device detection completed: {}", device_info.model);
        debug!("Device info: {:?}", device_info);

        Ok(device_info)
    }

    fn get_architecture(&self) -> Result<String> {
        std::env::consts::ARCH.to_string().into()
    }

    fn get_memory_mb(&self) -> Result<u64> {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return Ok(kb / 1024); // Convert KB to MB
                        }
                    }
                }
            }
        }
        
        // Fallback: Use system info
        Ok(1024) // Default 1GB
    }

    async fn detect_device_type(&self) -> Result<DeviceType> {
        // Check for Raspberry Pi
        if self.is_raspberry_pi() {
            return Ok(DeviceType::RaspberryPi);
        }

        // Check for Anbernic devices
        if self.is_anbernic_device().await {
            return Ok(DeviceType::Anbernic);
        }

        // Check for Orange Pi
        if self.is_orange_pi() {
            return Ok(DeviceType::OrangePi);
        }

        // Check for other ARM boards
        if self.is_banana_pi() {
            return Ok(DeviceType::BananaPi);
        }

        if self.is_rock_pi() {
            return Ok(DeviceType::RockPi);
        }

        if self.is_odroid() {
            return Ok(DeviceType::Odroid);
        }

        // Default to generic ARM device
        Ok(DeviceType::Generic)
    }

    fn is_raspberry_pi(&self) -> bool {
        // Check device tree model
        if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
            return content.to_lowercase().contains("raspberry pi");
        }

        // Check cpuinfo
        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            return content.to_lowercase().contains("raspberry pi");
        }

        false
    }

    async fn is_anbernic_device(&self) -> bool {
        // Check for Anbernic-specific files or processes
        if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
            let model = content.to_lowercase();
            return model.contains("rg351") || 
                   model.contains("rg552") || 
                   model.contains("rg35xx") ||
                   model.contains("anbernic");
        }

        // Check for Anbernic-specific directories
        std::path::Path::new("/opt/anbernic").exists() ||
        std::path::Path::new("/boot/anbernic").exists()
    }

    fn is_orange_pi(&self) -> bool {
        if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
            return content.to_lowercase().contains("orange pi");
        }
        false
    }

    fn is_banana_pi(&self) -> bool {
        if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
            return content.to_lowercase().contains("banana pi");
        }
        false
    }

    fn is_rock_pi(&self) -> bool {
        if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
            return content.to_lowercase().contains("rock pi");
        }
        false
    }

    fn is_odroid(&self) -> bool {
        if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
            return content.to_lowercase().contains("odroid");
        }
        false
    }

    fn get_device_model(&self, device_type: &DeviceType) -> Result<String> {
        match device_type {
            DeviceType::RaspberryPi => {
                if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
                    return Ok(content.trim_end_matches('\0').to_string());
                }
                Ok("Raspberry Pi (Unknown Model)".to_string())
            }
            DeviceType::Anbernic => {
                // Try to detect specific Anbernic model
                if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
                    let model = content.to_lowercase();
                    if model.contains("rg351") {
                        return Ok("Anbernic RG351".to_string());
                    } else if model.contains("rg552") {
                        return Ok("Anbernic RG552".to_string());
                    } else if model.contains("rg35xx") {
                        return Ok("Anbernic RG35XX".to_string());
                    }
                }
                Ok("Anbernic Gaming Handheld".to_string())
            }
            _ => {
                if let Ok(content) = fs::read_to_string("/proc/device-tree/model") {
                    Ok(content.trim_end_matches('\0').to_string())
                } else {
                    Ok(format!("{:?} Device", device_type))
                }
            }
        }
    }

    fn get_cpu_info(&self) -> Result<CpuInfo> {
        let mut model = "Unknown".to_string();
        let mut cores = 1u32;
        let mut frequency_mhz = None;

        if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
            for line in content.lines() {
                if line.starts_with("model name") || line.starts_with("Model") {
                    if let Some(value) = line.split(':').nth(1) {
                        model = value.trim().to_string();
                    }
                } else if line.starts_with("processor") {
                    cores += 1;
                } else if line.starts_with("cpu MHz") {
                    if let Some(value) = line.split(':').nth(1) {
                        if let Ok(freq) = value.trim().parse::<f32>() {
                            frequency_mhz = Some(freq as u32);
                        }
                    }
                }
            }
            
            // Correct cores count (processor lines start from 0)
            if cores > 0 {
                cores -= 1;
            }
        }

        Ok(CpuInfo {
            model,
            cores,
            frequency_mhz,
        })
    }

    fn has_gpio_support(&self, device_type: &DeviceType) -> bool {
        match device_type {
            DeviceType::RaspberryPi | DeviceType::OrangePi | 
            DeviceType::BananaPi | DeviceType::RockPi => true,
            DeviceType::Anbernic => false, // Gaming handhelds typically don't expose GPIO
            DeviceType::Odroid => true,
            DeviceType::Generic => {
                // Check for GPIO devices
                std::path::Path::new("/dev/gpiochip0").exists() ||
                std::path::Path::new("/sys/class/gpio").exists()
            }
        }
    }

    async fn detect_camera(&self) -> bool {
        // Check for camera devices
        std::path::Path::new("/dev/video0").exists() ||
        std::path::Path::new("/proc/device-tree/soc/i2c@7e804000/imx219@10").exists() ||
        std::path::Path::new("/proc/device-tree/soc/csi1").exists()
    }

    async fn detect_display_type(&self, device_type: &DeviceType) -> DisplayType {
        match device_type {
            DeviceType::RaspberryPi => {
                // Check for DSI display
                if std::path::Path::new("/proc/device-tree/soc/dsi@7e209000").exists() {
                    DisplayType::DSI
                } else {
                    DisplayType::HDMI
                }
            }
            DeviceType::Anbernic => {
                // Gaming handhelds typically have built-in LCD screens
                DisplayType::LCD
            }
            _ => {
                // Check for common display interfaces
                if std::path::Path::new("/sys/class/drm/card0-HDMI-A-1").exists() {
                    DisplayType::HDMI
                } else if std::path::Path::new("/sys/class/drm/card0-DSI-1").exists() {
                    DisplayType::DSI
                } else {
                    DisplayType::Unknown
                }
            }
        }
    }

    async fn detect_gaming_features(&self, device_type: &DeviceType) -> GamingFeatures {
        match device_type {
            DeviceType::Anbernic => {
                // Anbernic devices are gaming handhelds
                GamingFeatures {
                    has_dpad: true,
                    has_analog_sticks: true,
                    has_shoulder_buttons: true,
                    has_built_in_screen: true,
                    has_battery: true,
                    screen_size_inches: Some(3.5), // Typical for RG351
                    native_resolution: Some((480, 320)), // Common resolution
                }
            }
            DeviceType::RaspberryPi => {
                // Raspberry Pi can have gaming accessories
                GamingFeatures {
                    has_dpad: self.detect_gamepad_connected().await,
                    has_analog_sticks: self.detect_analog_controller().await,
                    has_shoulder_buttons: false,
                    has_built_in_screen: false,
                    has_battery: false,
                    screen_size_inches: None,
                    native_resolution: None,
                }
            }
            _ => {
                // Generic ARM device - minimal gaming features
                GamingFeatures {
                    has_dpad: false,
                    has_analog_sticks: false,
                    has_shoulder_buttons: false,
                    has_built_in_screen: false,
                    has_battery: false,
                    screen_size_inches: None,
                    native_resolution: None,
                }
            }
        }
    }

    async fn detect_gamepad_connected(&self) -> bool {
        // Check for input devices
        if let Ok(entries) = fs::read_dir("/dev/input") {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("js") || name.starts_with("event") {
                        return true;
                    }
                }
            }
        }
        false
    }

    async fn detect_analog_controller(&self) -> bool {
        // This would require more sophisticated input device analysis
        // For now, assume analog sticks are present if any controller is detected
        self.detect_gamepad_connected().await
    }
}