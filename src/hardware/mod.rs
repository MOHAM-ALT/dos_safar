pub mod device_detect;
pub mod display;
pub mod input;
pub mod network;
pub mod config_persist;

// Re-export commonly used types
pub use device_detect::{DeviceDetector, DeviceInfo, DeviceType};
pub use display::{DisplayTester, DisplayConfig};
pub use input::{InputTester, InputDevice, GamingControlsTest};
pub use network::{NetworkManager, NetworkConnection};