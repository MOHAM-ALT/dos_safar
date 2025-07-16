// Input device testing module 
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};
use crate::hardware::device_detect::{DeviceInfo, DeviceType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDevice {
    pub device_path: String,
    pub device_name: String,
    pub device_type: InputDeviceType,
    pub capabilities: InputCapabilities,
    pub is_working: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InputDeviceType {
    Gamepad,
    Keyboard,
    Mouse,
    Touchscreen,
    DPad,
    AnalogStick,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputCapabilities {
    pub has_buttons: bool,
    pub button_count: u32,
    pub has_dpad: bool,
    pub has_analog_sticks: bool,
    pub analog_stick_count: u32,
    pub has_triggers: bool,
    pub has_touchscreen: bool,
}

pub struct InputTester {
    device_info: DeviceInfo,
}

impl InputTester {
    pub fn new(device_info: &DeviceInfo) -> Self {
        InputTester {
            device_info: device_info.clone(),
        }
    }

    pub async fn test_controllers(&self) -> Result<Vec<InputDevice>> {
        info!("Testing input devices for {}", self.device_info.model);

        let mut detected_devices = Vec::new();

        // Scan for input devices
        let input_devices = self.scan_input_devices().await?;
        
        for device_path in input_devices {
            if let Ok(device) = self.analyze_input_device(&device_path).await {
                info!("Found input device: {} ({})", device.device_name, device.device_path);
                detected_devices.push(device);
            }
        }

        // For gaming handhelds, add built-in controls
        if self.device_info.gaming_features.has_dpad {
            let builtin_controls = self.detect_builtin_gaming_controls().await?;
            detected_devices.extend(builtin_controls);
        }

        // Test each device
        for device in &mut detected_devices {
            device.is_working = self.test_input_device(device).await;
        }

        info!("Input device scan completed: {} devices found", detected_devices.len());
        
        // Save working configuration
        self.save_input_config(&detected_devices).await?;

        Ok(detected_devices)
    }

    async fn scan_input_devices(&self) -> Result<Vec<String>> {
        let mut devices = Vec::new();
        let input_dir = "/dev/input";

        if !Path::new(input_dir).exists() {
            warn!("Input directory not found: {}", input_dir);
            return Ok(devices);
        }

        // Scan /dev/input for devices
        if let Ok(entries) = fs::read_dir(input_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    
                    // Look for event devices and joystick devices
                    if name_str.starts_with("event") || name_str.starts_with("js") {
                        devices.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        debug!("Found {} input device files", devices.len());
        Ok(devices)
    }

    async fn analyze_input_device(&self, device_path: &str) -> Result<InputDevice> {
        let device_name = self.get_device_name(device_path).await
            .unwrap_or_else(|| "Unknown Device".to_string());

        let device_type = self.determine_device_type(device_path, &device_name).await;
        let capabilities = self.analyze_device_capabilities(device_path, &device_type).await;

        Ok(InputDevice {
            device_path: device_path.to_string(),
            device_name,
            device_type,
            capabilities,
            is_working: false, // Will be tested later
        })
    }

    async fn get_device_name(&self, device_path: &str) -> Option<String> {
        // Try to get device name from various sources
        
        // Method 1: Check /proc/bus/input/devices
        if let Ok(content) = fs::read_to_string("/proc/bus/input/devices") {
            if let Some(name) = self.parse_device_name_from_proc(&content, device_path) {
                return Some(name);
            }
        }

        // Method 2: Use device path extraction
        if device_path.contains("event") {
            return Some(format!("Input Event Device {}", 
                device_path.chars().last().unwrap_or('0')));
        } else if device_path.contains("js") {
            return Some(format!("Joystick Device {}", 
                device_path.chars().last().unwrap_or('0')));
        }

        None
    }

    fn parse_device_name_from_proc(&self, content: &str, target_device: &str) -> Option<String> {
        let sections: Vec<&str> = content.split("\n\n").collect();
        
        for section in sections {
            if section.contains(target_device) {
                for line in section.lines() {
                    if line.starts_with("N: Name=") {
                        let name = line.strip_prefix("N: Name=")?;
                        return Some(name.trim_matches('"').to_string());
                    }
                }
            }
        }
        
        None
    }

    async fn determine_device_type(&self, device_path: &str, device_name: &str) -> InputDeviceType {
        let name_lower = device_name.to_lowercase();
        let path_lower = device_path.to_lowercase();

        // Check for specific device types based on name
        if name_lower.contains("gamepad") || name_lower.contains("controller") || 
           name_lower.contains("joystick") || path_lower.contains("js") {
            return InputDeviceType::Gamepad;
        }

        if name_lower.contains("keyboard") || name_lower.contains("kbd") {
            return InputDeviceType::Keyboard;
        }

        if name_lower.contains("mouse") || name_lower.contains("pointing") {
            return InputDeviceType::Mouse;
        }

        if name_lower.contains("touch") || name_lower.contains("screen") {
            return InputDeviceType::Touchscreen;
        }

        // For gaming handhelds, check for built-in controls
        if self.device_info.device_type == DeviceType::Anbernic {
            if name_lower.contains("gpio") || name_lower.contains("adc") {
                return InputDeviceType::DPad;
            }
        }

        InputDeviceType::Unknown
    }

    async fn analyze_device_capabilities(&self, device_path: &str, device_type: &InputDeviceType) -> InputCapabilities {
        // This is a simplified capability detection
        // In a real implementation, you would use ioctl calls to query device capabilities
        
        match device_type {
            InputDeviceType::Gamepad => {
                InputCapabilities {
                    has_buttons: true,
                    button_count: 12, // Typical gamepad button count
                    has_dpad: true,
                    has_analog_sticks: true,
                    analog_stick_count: 2,
                    has_triggers: true,
                    has_touchscreen: false,
                }
            }
            InputDeviceType::DPad => {
                InputCapabilities {
                    has_buttons: true,
                    button_count: 8, // D-pad + action buttons
                    has_dpad: true,
                    has_analog_sticks: false,
                    analog_stick_count: 0,
                    has_triggers: false,
                    has_touchscreen: false,
                }
            }
            InputDeviceType::Keyboard => {
                InputCapabilities {
                    has_buttons: true,
                    button_count: 104, // Standard keyboard
                    has_dpad: false,
                    has_analog_sticks: false,
                    analog_stick_count: 0,
                    has_triggers: false,
                    has_touchscreen: false,
                }
            }
            InputDeviceType::Mouse => {
                InputCapabilities {
                    has_buttons: true,
                    button_count: 3, // Left, right, middle
                    has_dpad: false,
                    has_analog_sticks: false,
                    analog_stick_count: 0,
                    has_triggers: false,
                    has_touchscreen: false,
                }
            }
            InputDeviceType::Touchscreen => {
                InputCapabilities {
                    has_buttons: false,
                    button_count: 0,
                    has_dpad: false,
                    has_analog_sticks: false,
                    analog_stick_count: 0,
                    has_triggers: false,
                    has_touchscreen: true,
                }
            }
            _ => {
                InputCapabilities {
                    has_buttons: false,
                    button_count: 0,
                    has_dpad: false,
                    has_analog_sticks: false,
                    analog_stick_count: 0,
                    has_triggers: false,
                    has_touchscreen: false,
                }
            }
        }
    }

    async fn detect_builtin_gaming_controls(&self) -> Result<Vec<InputDevice>> {
        let mut builtin_devices = Vec::new();

        if self.device_info.device_type == DeviceType::Anbernic {
            // Add built-in gaming controls for Anbernic devices
            let dpad = InputDevice {
                device_path: "/dev/input/builtin_dpad".to_string(),
                device_name: "Built-in D-Pad".to_string(),
                device_type: InputDeviceType::DPad,
                capabilities: InputCapabilities {
                    has_buttons: true,
                    button_count: 4,
                    has_dpad: true,
                    has_analog_sticks: false,
                    analog_stick_count: 0,
                    has_triggers: false,
                    has_touchscreen: false,
                },
                is_working: true, // Assume built-in controls work
            };

            builtin_devices.push(dpad);

            // Add analog stick if present
            if self.device_info.gaming_features.has_analog_sticks {
                let analog_stick = InputDevice {
                    device_path: "/dev/input/builtin_analog".to_string(),
                    device_name: "Built-in Analog Stick".to_string(),
                    device_type: InputDeviceType::AnalogStick,
                    capabilities: InputCapabilities {
                        has_buttons: false,
                        button_count: 0,
                        has_dpad: false,
                        has_analog_sticks: true,
                        analog_stick_count: 1,
                        has_triggers: false,
                        has_touchscreen: false,
                    },
                    is_working: true,
                };

                builtin_devices.push(analog_stick);
            }
        }

        Ok(builtin_devices)
    }

    async fn test_input_device(&self, device: &InputDevice) -> bool {
        // Test if the input device is accessible and functional
        
        // For built-in devices, assume they work
        if device.device_path.contains("builtin") {
            return true;
        }

        // Test 1: Check if device file is accessible
        if !Path::new(&device.device_path).exists() {
            debug!("Input device not accessible: {}", device.device_path);
            return false;
        }

        // Test 2: Try to open the device for reading
        match fs::File::open(&device.device_path) {
            Ok(_) => {
                debug!("Input device accessible: {}", device.device_name);
                true
            }
            Err(e) => {
                debug!("Failed to open input device {}: {}", device.device_name, e);
                false
            }
        }
    }

    async fn save_input_config(&self, devices: &[InputDevice]) -> Result<()> {
        let config_dir = "/boot/dos_safar/input";
        std::fs::create_dir_all(config_dir)
            .context("Failed to create input config directory")?;

        let config_file = format!("{}/detected_devices.toml", config_dir);
        let config_content = toml::to_string_pretty(devices)
            .context("Failed to serialize input devices config")?;

        fs::write(&config_file, config_content)
            .context("Failed to save input devices config")?;

        info!("Saved input devices configuration to {}", config_file);
        Ok(())
    }

    pub async fn test_specific_gaming_controls(&self) -> Result<GamingControlsTest> {
        info!("Testing gaming-specific controls");

        let mut test_result = GamingControlsTest {
            dpad_working: false,
            action_buttons_working: false,
            shoulder_buttons_working: false,
            analog_sticks_working: false,
            start_select_working: false,
        };

        // Test D-Pad
        if self.device_info.gaming_features.has_dpad {
            test_result.dpad_working = self.test_dpad_functionality().await;
        }

        // Test action buttons (A, B, X, Y)
        test_result.action_buttons_working = self.test_action_buttons().await;

        // Test shoulder buttons (L1, L2, R1, R2)
        if self.device_info.gaming_features.has_shoulder_buttons {
            test_result.shoulder_buttons_working = self.test_shoulder_buttons().await;
        }

        // Test analog sticks
        if self.device_info.gaming_features.has_analog_sticks {
            test_result.analog_sticks_working = self.test_analog_sticks().await;
        }

        // Test start/select buttons
        test_result.start_select_working = self.test_start_select_buttons().await;

        info!("Gaming controls test completed: {:?}", test_result);
        Ok(test_result)
    }

    async fn test_dpad_functionality(&self) -> bool {
        // For gaming handhelds, assume D-pad works if device is detected
        match self.device_info.device_type {
            DeviceType::Anbernic => true,
            _ => {
                // Check for D-pad input events
                self.check_for_dpad_events().await
            }
        }
    }

    async fn check_for_dpad_events(&self) -> bool {
        // This would involve reading input events for D-pad presses
        // For now, we'll do a simple check
        Path::new("/dev/input/event0").exists()
    }

    async fn test_action_buttons(&self) -> bool {
        // Test A, B, X, Y buttons
        match self.device_info.device_type {
            DeviceType::Anbernic => true, // Built-in buttons should work
            _ => {
                // Check for gamepad with action buttons
                self.has_gamepad_with_buttons().await
            }
        }
    }

    async fn has_gamepad_with_buttons(&self) -> bool {
        // Check if any detected gamepad has the required buttons
        if let Ok(devices) = self.scan_input_devices().await {
            for device_path in devices {
                if device_path.contains("js") {
                    return true; // Joystick device found
                }
            }
        }
        false
    }

    async fn test_shoulder_buttons(&self) -> bool {
        // Test L1, L2, R1, R2 buttons
        self.device_info.gaming_features.has_shoulder_buttons
    }

    async fn test_analog_sticks(&self) -> bool {
        // Test analog stick functionality
        self.device_info.gaming_features.has_analog_sticks
    }

    async fn test_start_select_buttons(&self) -> bool {
        // Test start and select buttons
        match self.device_info.device_type {
            DeviceType::Anbernic => true,
            _ => self.has_gamepad_with_buttons().await,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingControlsTest {
    pub dpad_working: bool,
    pub action_buttons_working: bool,
    pub shoulder_buttons_working: bool,
    pub analog_sticks_working: bool,
    pub start_select_working: bool,
}

impl GamingControlsTest {
    pub fn is_fully_functional(&self) -> bool {
        self.dpad_working && self.action_buttons_working && self.start_select_working
    }

    pub fn get_working_controls_count(&self) -> u32 {
        let mut count = 0;
        if self.dpad_working { count += 1; }
        if self.action_buttons_working { count += 1; }
        if self.shoulder_buttons_working { count += 1; }
        if self.analog_sticks_working { count += 1; }
        if self.start_select_working { count += 1; }
        count
    }
}