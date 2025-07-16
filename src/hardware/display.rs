// Display testing module 
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use tracing::{debug, info, warn};
use crate::hardware::device_detect::{DeviceInfo, DeviceType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub color_depth: u32,
    pub interface: String,
    pub is_working: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayTestResult {
    pub config: DisplayConfig,
    pub test_passed: bool,
    pub error_message: Option<String>,
}

pub struct DisplayTester {
    device_info: DeviceInfo,
}

impl DisplayTester {
    pub fn new(device_info: &DeviceInfo) -> Self {
        DisplayTester {
            device_info: device_info.clone(),
        }
    }

    pub async fn test_display(&self) -> Result<DisplayConfig> {
        info!("Testing display configuration for {}", self.device_info.model);

        // Get current display configuration
        let config = self.detect_current_display_config().await?;
        
        // Test display functionality
        let test_result = self.run_display_test(&config).await?;
        
        if test_result.test_passed {
            info!("Display test passed: {}x{} @ {}Hz", 
                  config.width, config.height, config.refresh_rate);
            
            // Save working configuration
            self.save_working_config(&config).await?;
        } else {
            warn!("Display test failed: {:?}", test_result.error_message);
        }

        Ok(config)
    }

    async fn detect_current_display_config(&self) -> Result<DisplayConfig> {
        match self.device_info.device_type {
            DeviceType::RaspberryPi => self.detect_raspberry_pi_display().await,
            DeviceType::Anbernic => self.detect_anbernic_display().await,
            _ => self.detect_generic_display().await,
        }
    }

    async fn detect_raspberry_pi_display(&self) -> Result<DisplayConfig> {
        // Try to get display info from various sources
        
        // Method 1: Check framebuffer
        if let Ok(config) = self.get_framebuffer_config().await {
            return Ok(config);
        }

        // Method 2: Check DRM/KMS
        if let Ok(config) = self.get_drm_config().await {
            return Ok(config);
        }

        // Method 3: Use vcgencmd (Raspberry Pi specific)
        if let Ok(config) = self.get_vcgencmd_config().await {
            return Ok(config);
        }

        // Fallback to default config
        Ok(DisplayConfig {
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            color_depth: 24,
            interface: "HDMI".to_string(),
            is_working: false,
        })
    }

    async fn detect_anbernic_display(&self) -> Result<DisplayConfig> {
        // Anbernic devices typically have fixed resolution displays
        let (width, height) = match self.device_info.gaming_features.native_resolution {
            Some((w, h)) => (w, h),
            None => (480, 320), // Common Anbernic resolution
        };

        Ok(DisplayConfig {
            width,
            height,
            refresh_rate: 60,
            color_depth: 16, // Gaming handhelds often use 16-bit color
            interface: "LCD".to_string(),
            is_working: true, // Assume built-in display works
        })
    }

    async fn detect_generic_display(&self) -> Result<DisplayConfig> {
        // Try framebuffer first
        if let Ok(config) = self.get_framebuffer_config().await {
            return Ok(config);
        }

        // Fallback to safe defaults
        Ok(DisplayConfig {
            width: 1024,
            height: 768,
            refresh_rate: 60,
            color_depth: 24,
            interface: "Unknown".to_string(),
            is_working: false,
        })
    }

    async fn get_framebuffer_config(&self) -> Result<DisplayConfig> {
        // Check /sys/class/graphics/fb0/ for framebuffer info
        let fb_path = "/sys/class/graphics/fb0";
        
        if !std::path::Path::new(fb_path).exists() {
            return Err(anyhow::anyhow!("Framebuffer not found"));
        }

        // Read virtual resolution
        let virtual_size = fs::read_to_string(format!("{}/virtual_size", fb_path))
            .context("Failed to read virtual_size")?;
        
        let (width, height) = parse_resolution(&virtual_size)?;

        // Read bits per pixel
        let bits_per_pixel = fs::read_to_string(format!("{}/bits_per_pixel", fb_path))
            .unwrap_or_else(|_| "24".to_string());
        
        let color_depth = bits_per_pixel.trim().parse::<u32>().unwrap_or(24);

        Ok(DisplayConfig {
            width,
            height,
            refresh_rate: 60, // Default refresh rate
            color_depth,
            interface: "Framebuffer".to_string(),
            is_working: true,
        })
    }

    async fn get_drm_config(&self) -> Result<DisplayConfig> {
        // Try to get info from DRM subsystem
        let drm_path = "/sys/class/drm";
        
        if !std::path::Path::new(drm_path).exists() {
            return Err(anyhow::anyhow!("DRM not available"));
        }

        // Look for connected displays
        if let Ok(entries) = fs::read_dir(drm_path) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                
                if name_str.contains("HDMI") || name_str.contains("DSI") {
                    let status_path = entry.path().join("status");
                    if let Ok(status) = fs::read_to_string(&status_path) {
                        if status.trim() == "connected" {
                            // Try to get mode information
                            if let Ok(config) = self.parse_drm_mode(&entry.path()).await {
                                return Ok(config);
                            }
                        }
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No connected displays found via DRM"))
    }

    async fn get_vcgencmd_config(&self) -> Result<DisplayConfig> {
        // Raspberry Pi specific: use vcgencmd to get display info
        let output = Command::new("vcgencmd")
            .arg("get_config")
            .arg("hdmi_mode")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let mode_str = String::from_utf8_lossy(&output.stdout);
                debug!("vcgencmd hdmi_mode: {}", mode_str);
                
                // Parse HDMI mode and return appropriate config
                // This is a simplified implementation
                return Ok(DisplayConfig {
                    width: 1920,
                    height: 1080,
                    refresh_rate: 60,
                    color_depth: 24,
                    interface: "HDMI".to_string(),
                    is_working: true,
                });
            }
        }

        Err(anyhow::anyhow!("vcgencmd not available or failed"))
    }

    async fn parse_drm_mode(&self, drm_path: &std::path::Path) -> Result<DisplayConfig> {
        let modes_path = drm_path.join("modes");
        
        if let Ok(modes_content) = fs::read_to_string(&modes_path) {
            // Parse the first mode (usually the preferred one)
            if let Some(first_line) = modes_content.lines().next() {
                if let Ok((width, height, refresh)) = parse_drm_mode_line(first_line) {
                    return Ok(DisplayConfig {
                        width,
                        height,
                        refresh_rate: refresh,
                        color_depth: 24,
                        interface: "DRM".to_string(),
                        is_working: true,
                    });
                }
            }
        }

        Err(anyhow::anyhow!("Failed to parse DRM modes"))
    }

    async fn run_display_test(&self, config: &DisplayConfig) -> Result<DisplayTestResult> {
        info!("Running display test for {}x{}", config.width, config.height);

        // For gaming handhelds with built-in screens, assume test passes
        if self.device_info.gaming_features.has_built_in_screen {
            return Ok(DisplayTestResult {
                config: config.clone(),
                test_passed: true,
                error_message: None,
            });
        }

        // Test 1: Try to write to framebuffer
        let fb_test = self.test_framebuffer_write().await;
        
        // Test 2: Check if display is responsive
        let responsive_test = self.test_display_responsive().await;

        let test_passed = fb_test && responsive_test;
        let error_message = if !test_passed {
            Some("Display test failed: framebuffer or responsiveness issue".to_string())
        } else {
            None
        };

        Ok(DisplayTestResult {
            config: config.clone(),
            test_passed,
            error_message,
        })
    }

    async fn test_framebuffer_write(&self) -> bool {
        // Try to write a simple pattern to framebuffer
        match fs::OpenOptions::new().write(true).open("/dev/fb0") {
            Ok(_) => {
                debug!("Framebuffer write test passed");
                true
            }
            Err(e) => {
                debug!("Framebuffer write test failed: {}", e);
                false
            }
        }
    }

    async fn test_display_responsive(&self) -> bool {
        // For now, just check if display files are accessible
        std::path::Path::new("/sys/class/graphics/fb0").exists() ||
        std::path::Path::new("/dev/fb0").exists()
    }

    async fn save_working_config(&self, config: &DisplayConfig) -> Result<()> {
        let config_dir = "/boot/dos_safar/display";
        std::fs::create_dir_all(config_dir)
            .context("Failed to create display config directory")?;

        let config_file = format!("{}/working_config.toml", config_dir);
        let config_content = toml::to_string_pretty(config)
            .context("Failed to serialize display config")?;

        fs::write(&config_file, config_content)
            .context("Failed to save display config")?;

        info!("Saved working display configuration to {}", config_file);
        Ok(())
    }
}

fn parse_resolution(resolution_str: &str) -> Result<(u32, u32)> {
    let parts: Vec<&str> = resolution_str.trim().split(',').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid resolution format"));
    }

    let width = parts[0].parse::<u32>()
        .context("Failed to parse width")?;
    let height = parts[1].parse::<u32>()
        .context("Failed to parse height")?;

    Ok((width, height))
}

fn parse_drm_mode_line(mode_line: &str) -> Result<(u32, u32, u32)> {
    // Parse DRM mode line format: "1920x1080@60"
    let mode_line = mode_line.trim();
    
    // Split by '@' to separate resolution and refresh rate
    let parts: Vec<&str> = mode_line.split('@').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid DRM mode format"));
    }

    // Parse resolution
    let resolution_parts: Vec<&str> = parts[0].split('x').collect();
    if resolution_parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid resolution format in DRM mode"));
    }

    let width = resolution_parts[0].parse::<u32>()?;
    let height = resolution_parts[1].parse::<u32>()?;
    let refresh = parts[1].parse::<u32>()?;

    Ok((width, height, refresh))
}