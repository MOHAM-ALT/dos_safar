pub mod device_detect;
pub mod display;
pub mod input;
pub mod network;
pub mod config_persist;
pub mod lcd_display; // إضافة جديدة

// Re-export commonly used types
pub use device_detect::{DeviceDetector, DeviceInfo, DeviceType};
pub use display::{DisplayTester, DisplayConfig};
pub use input::{InputTester, InputDevice, GamingControlsTest};
pub use network::{NetworkManager, NetworkConnection};
pub use lcd_display::{LcdDisplayDetector, LcdDisplayConfig, LcdDriver}; // إضافة جديدة