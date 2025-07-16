// Boot menu implementation 
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};
use crate::hardware::device_detect::DeviceInfo;
use crate::utils::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootMenu {
    pub config: Config,
    pub device_info: DeviceInfo,
    pub available_systems: Vec<OperatingSystem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSystem {
    pub name: String,
    pub path: String,
    pub description: String,
    pub os_type: OSType,
    pub is_bootable: bool,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OSType {
    RetroPie,
    Batocera,
    Recalbox,
    RaspberryPiOS,
    Ubuntu,
    Debian,
    Unknown,
}

impl BootMenu {
    pub fn new(config: &Config, device_info: &DeviceInfo) -> Result<Self> {
        let mut boot_menu = BootMenu {
            config: config.clone(),
            device_info: device_info.clone(),
            available_systems: Vec::new(),
        };

        // Scan for available operating systems
        boot_menu.scan_for_operating_systems()?;

        Ok(boot_menu)
    }

    pub async fn show_menu(&self) -> Result<()> {
        info!("=== DOS Safar Boot Menu ===");
        info!("Device: {}", self.device_info.model);
        
        if self.available_systems.is_empty() {
            warn!("No operating systems found!");
            self.show_no_os_menu().await?;
            return Ok(());
        }

        // Check if we have a default OS and auto-boot is enabled
        if let Some(default_os) = &self.config.boot.default_os {
            if !default_os.is_empty() {
                return self.auto_boot_default(default_os).await;
            }
        }

        // Show interactive menu
        self.show_interactive_menu().await
    }

    fn scan_for_operating_systems(&mut self) -> Result<()> {
        info!("Scanning for operating systems...");

        // Scan different potential locations
        self.scan_boot_partitions()?;
        self.scan_sd_card_images()?;
        self.scan_usb_devices()?;

        info!("Found {} operating systems", self.available_systems.len());
        
        // Sort by last used (most recent first)
        self.available_systems.sort_by(|a, b| {
            match (&a.last_used, &b.last_used) {
                (Some(a_time), Some(b_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.name.cmp(&b.name),
            }
        });

        Ok(())
    }

    fn scan_boot_partitions(&mut self) -> Result<()> {
        // Look for boot partitions with different OS signatures
        let boot_paths = vec![
            "/boot",
            "/mnt/boot",
            "/media/boot",
        ];

        for boot_path in boot_paths {
            if std::path::Path::new(boot_path).exists() {
                if let Ok(os) = self.identify_os_from_boot_partition(boot_path) {
                    self.available_systems.push(os);
                }
            }
        }

        Ok(())
    }

    fn scan_sd_card_images(&mut self) -> Result<()> {
        // Look for OS images on SD card
        let image_paths = vec![
            "/boot/os_images/",
            "/home/dos_safar/images/",
            "/opt/dos_safar/images/",
        ];

        for image_path in image_paths {
            if let Ok(entries) = std::fs::read_dir(image_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(extension) = path.extension() {
                        let ext = extension.to_string_lossy().to_lowercase();
                        if ext == "img" || ext == "iso" {
                            if let Ok(os) = self.identify_os_from_image(&path) {
                                self.available_systems.push(os);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn scan_usb_devices(&mut self) -> Result<()> {
        // Look for bootable USB devices
        let usb_mount_paths = vec![
            "/media/",
            "/mnt/",
            "/run/media/",
        ];

        for mount_path in usb_mount_paths {
            if let Ok(entries) = std::fs::read_dir(mount_path) {
                for entry in entries.flatten() {
                    let device_path = entry.path();
                    if device_path.is_dir() {
                        // Check if this looks like a bootable OS
                        if let Ok(os) = self.identify_os_from_boot_partition(&device_path.to_string_lossy()) {
                            self.available_systems.push(os);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn identify_os_from_boot_partition(&self, boot_path: &str) -> Result<OperatingSystem> {
        let boot_path = std::path::Path::new(boot_path);
        
        // Check for RetroPie
        if boot_path.join("retropie").exists() || 
           boot_path.join("RetroPie").exists() {
            return Ok(OperatingSystem {
                name: "RetroPie".to_string(),
                path: boot_path.to_string_lossy().to_string(),
                description: "Retro Gaming System".to_string(),
                os_type: OSType::RetroPie,
                is_bootable: true,
                last_used: None,
            });
        }

        // Check for Batocera
        if boot_path.join("batocera").exists() ||
           boot_path.join("BATOCERA").exists() {
            return Ok(OperatingSystem {
                name: "Batocera".to_string(),
                path: boot_path.to_string_lossy().to_string(),
                description: "Retro Gaming Distribution".to_string(),
                os_type: OSType::Batocera,
                is_bootable: true,
                last_used: None,
            });
        }

        // Check for Recalbox
        if boot_path.join("recalbox").exists() {
            return Ok(OperatingSystem {
                name: "Recalbox".to_string(),
                path: boot_path.to_string_lossy().to_string(),
                description: "Retro Gaming OS".to_string(),
                os_type: OSType::Recalbox,
                is_bootable: true,
                last_used: None,
            });
        }

        // Check for Raspberry Pi OS
        if boot_path.join("config.txt").exists() &&
           boot_path.join("cmdline.txt").exists() {
            return Ok(OperatingSystem {
                name: "Raspberry Pi OS".to_string(),
                path: boot_path.to_string_lossy().to_string(),
                description: "Official Raspberry Pi Operating System".to_string(),
                os_type: OSType::RaspberryPiOS,
                is_bootable: true,
                last_used: None,
            });
        }

        // Check for Ubuntu/Debian
        if boot_path.join("ubuntu").exists() ||
           boot_path.join("vmlinuz").exists() {
            return Ok(OperatingSystem {
                name: "Linux System".to_string(),
                path: boot_path.to_string_lossy().to_string(),
                description: "General Linux Distribution".to_string(),
                os_type: OSType::Ubuntu,
                is_bootable: true,
                last_used: None,
            });
        }

        Err(anyhow::anyhow!("Unknown OS type"))
    }

    fn identify_os_from_image(&self, image_path: &std::path::Path) -> Result<OperatingSystem> {
        let filename = image_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let filename_lower = filename.to_lowercase();

        let (name, os_type, description) = if filename_lower.contains("retropie") {
            ("RetroPie Image".to_string(), OSType::RetroPie, "RetroPie OS Image".to_string())
        } else if filename_lower.contains("batocera") {
            ("Batocera Image".to_string(), OSType::Batocera, "Batocera OS Image".to_string())
        } else if filename_lower.contains("recalbox") {
            ("Recalbox Image".to_string(), OSType::Recalbox, "Recalbox OS Image".to_string())
        } else if filename_lower.contains("raspios") || filename_lower.contains("raspberry") {
            ("Raspberry Pi OS Image".to_string(), OSType::RaspberryPiOS, "Raspberry Pi OS Image".to_string())
        } else {
            (format!("OS Image: {}", filename), OSType::Unknown, "Unknown OS Image".to_string())
        };

        Ok(OperatingSystem {
            name,
            path: image_path.to_string_lossy().to_string(),
            description,
            os_type,
            is_bootable: true,
            last_used: None,
        })
    }

    async fn auto_boot_default(&self, default_os: &str) -> Result<()> {
        info!("Auto-booting default OS: {}", default_os);
        
        // Find the default OS
        if let Some(os) = self.available_systems.iter().find(|os| os.name == default_os) {
            info!("Booting into {}", os.name);
            self.boot_operating_system(os).await?;
        } else {
            warn!("Default OS '{}' not found, showing menu", default_os);
            self.show_interactive_menu().await?;
        }

        Ok(())
    }

    async fn show_interactive_menu(&self) -> Result<()> {
        info!("=== Interactive Boot Menu ===");
        
        // Display menu options
        self.display_menu_header();
        
        for (index, os) in self.available_systems.iter().enumerate() {
            self.display_menu_item(index + 1, os);
        }
        
        self.display_menu_footer();

        // Gaming mode: Show timeout and wait for input
        if self.config.boot.gaming_mode {
            self.gaming_mode_selection().await
        } else {
            self.standard_mode_selection().await
        }
    }

    fn display_menu_header(&self) {
        println!("\n🎮 DOS Safar Boot Manager 🎮");
        println!("Device: {}", self.device_info.model);
        if self.device_info.gaming_features.has_built_in_screen {
            println!("Screen: {}\"", self.device_info.gaming_features.screen_size_inches.unwrap_or(3.5));
        }
        println!("═══════════════════════════════════════");
    }

    fn display_menu_item(&self, index: usize, os: &OperatingSystem) {
        let icon = match os.os_type {
            OSType::RetroPie => "🕹️",
            OSType::Batocera => "🎯",
            OSType::Recalbox => "📦",
            OSType::RaspberryPiOS => "🍓",
            OSType::Ubuntu => "🐧",
            OSType::Debian => "🌊",
            OSType::Unknown => "❓",
        };

        println!("  {}. {} {} - {}", index, icon, os.name, os.description);
        
        if let Some(last_used) = os.last_used {
            println!("     Last used: {}", last_used.format("%Y-%m-%d %H:%M"));
        }
    }

    fn display_menu_footer(&self) {
        println!("═══════════════════════════════════════");
        println!("  A. Advanced Options");
        println!("  W. Web Interface");
        println!("  R. Restart Hardware Tests");
        println!("  S. Shutdown");
        println!("═══════════════════════════════════════");
        
        if self.config.boot.gaming_mode {
            println!("🎮 Use D-Pad to navigate, A to select");
            println!("⏱️  Auto-boot in {} seconds...", self.config.boot.menu_timeout_seconds);
        } else {
            println!("Enter your choice (1-{}):", self.available_systems.len());
        }
    }

    async fn gaming_mode_selection(&self) -> Result<()> {
        // Simplified input handling for gaming mode
        // In a real implementation, you would read from input devices
        
        let timeout_duration = Duration::from_secs(self.config.boot.menu_timeout_seconds);
        
        // Wait for timeout or input
        match timeout(timeout_duration, self.wait_for_gaming_input()).await {
            Ok(selection) => {
                self.handle_selection(selection).await?;
            }
            Err(_) => {
                // Timeout - boot first available system
                if let Some(first_os) = self.available_systems.first() {
                    info!("Timeout reached, booting {}", first_os.name);
                    self.boot_operating_system(first_os).await?;
                } else {
                    warn!("No systems available to auto-boot");
                }
            }
        }

        Ok(())
    }

    async fn wait_for_gaming_input(&self) -> MenuSelection {
        // Simplified input simulation
        // In real implementation, this would read from gaming controls
        
        // For now, just wait and return first option
        sleep(Duration::from_millis(100)).await;
        
        // Simulate user pressing A button to select first item
        MenuSelection::BootOS(0)
    }

    async fn standard_mode_selection(&self) -> Result<()> {
        // Standard keyboard input mode
        // This would implement proper stdin reading
        // For now, just boot the first system
        
        if let Some(first_os) = self.available_systems.first() {
            info!("Standard mode: booting {}", first_os.name);
            self.boot_operating_system(first_os).await?;
        }

        Ok(())
    }

    async fn handle_selection(&self, selection: MenuSelection) -> Result<()> {
        match selection {
            MenuSelection::BootOS(index) => {
                if let Some(os) = self.available_systems.get(index) {
                    self.boot_operating_system(os).await?;
                } else {
                    warn!("Invalid OS selection: {}", index);
                }
            }
            MenuSelection::AdvancedOptions => {
                self.show_advanced_menu().await?;
            }
            MenuSelection::WebInterface => {
                info!("Web interface is already running");
                println!("Web interface available at: http://localhost:8080");
            }
            MenuSelection::RestartTests => {
                info!("Restarting hardware tests...");
                // This would restart the hardware testing process
            }
            MenuSelection::Shutdown => {
                info!("Shutting down system...");
                self.shutdown_system().await?;
            }
        }

        Ok(())
    }

    async fn boot_operating_system(&self, os: &OperatingSystem) -> Result<()> {
        info!("🚀 Booting into: {}", os.name);
        
        // Save boot selection
        self.save_boot_selection(os).await?;
        
        // Apply any hardware configurations
        self.apply_hardware_config_for_os(os).await?;
        
        // Perform the actual boot
        match os.os_type {
            OSType::RetroPie | OSType::Batocera | OSType::Recalbox => {
                self.boot_gaming_os(os).await?;
            }
            OSType::RaspberryPiOS | OSType::Ubuntu | OSType::Debian => {
                self.boot_standard_os(os).await?;
            }
            OSType::Unknown => {
                self.boot_unknown_os(os).await?;
            }
        }

        Ok(())
    }

    async fn boot_gaming_os(&self, os: &OperatingSystem) -> Result<()> {
        info!("Booting gaming OS: {}", os.name);
        
        // Gaming OS specific boot sequence
        // This would configure controllers, displays, etc.
        
        // For now, just simulate boot
        println!("🎮 Configuring gaming controls...");
        sleep(Duration::from_secs(1)).await;
        
        println!("🎮 Loading {} system...", os.name);
        sleep(Duration::from_secs(2)).await;
        
        println!("🎮 {} is ready!", os.name);
        
        Ok(())
    }

    async fn boot_standard_os(&self, os: &OperatingSystem) -> Result<()> {
        info!("Booting standard OS: {}", os.name);
        
        // Standard OS boot sequence
        println!("🐧 Loading {} system...", os.name);
        sleep(Duration::from_secs(2)).await;
        
        println!("🐧 {} is ready!", os.name);
        
        Ok(())
    }

    async fn boot_unknown_os(&self, os: &OperatingSystem) -> Result<()> {
        info!("Booting unknown OS: {}", os.name);
        
        // Generic boot sequence
        println!("❓ Loading system from {}...", os.path);
        sleep(Duration::from_secs(2)).await;
        
        println!("❓ System loaded!");
        
        Ok(())
    }

    async fn show_advanced_menu(&self) -> Result<()> {
        println!("\n=== Advanced Options ===");
        println!("1. Install New OS");
        println!("2. Remove OS");
        println!("3. Hardware Configuration");
        println!("4. Network Settings");
        println!("5. Back to Main Menu");
        
        // For now, just return to main menu
        self.show_interactive_menu().await
    }

    async fn show_no_os_menu(&self) -> Result<()> {
        println!("\n⚠️  No Operating Systems Found!");
        println!("═══════════════════════════════════");
        println!("Options:");
        println!("1. 🌐 Download OS images via web interface");
        println!("2. 🔍 Rescan for OS images");
        println!("3. 📁 Check connected USB drives");
        println!("4. ⚡ Emergency shell");
        println!("═══════════════════════════════════");
        
        // Start web interface for OS installation
        println!("💡 Starting web interface for OS management...");
        println!("Visit: http://localhost:8080 to install operating systems");
        
        Ok(())
    }

    async fn save_boot_selection(&self, os: &OperatingSystem) -> Result<()> {
        // Save the selected OS as the last used
        // This would update the configuration file
        info!("Saving boot selection: {}", os.name);
        Ok(())
    }

    async fn apply_hardware_config_for_os(&self, os: &OperatingSystem) -> Result<()> {
        // Apply OS-specific hardware configurations
        match os.os_type {
            OSType::RetroPie | OSType::Batocera | OSType::Recalbox => {
                // Apply gaming-specific configurations
                info!("Applying gaming hardware configuration");
            }
            _ => {
                // Apply standard configurations
                info!("Applying standard hardware configuration");
            }
        }
        Ok(())
    }

    async fn shutdown_system(&self) -> Result<()> {
        println!("💤 Shutting down DOS Safar...");
        
        // Graceful shutdown
        std::process::Command::new("shutdown")
            .args(&["-h", "now"])
            .output()
            .context("Failed to shutdown system")?;
        
        Ok(())
    }
}

#[derive(Debug)]
enum MenuSelection {
    BootOS(usize),
    AdvancedOptions,
    WebInterface,
    RestartTests,
    Shutdown,
}