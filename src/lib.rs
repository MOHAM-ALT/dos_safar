pub mod hardware;
pub mod bootloader;
pub mod remote;
pub mod utils;

// Re-export commonly used types
pub use hardware::device_detect::{DeviceInfo, DeviceType};
pub use utils::config::Config;
pub use utils::logger::init_logger;

// Common error types
pub type Result<T> = anyhow::Result<T>;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = "DOS Safar";
pub const DESCRIPTION: &str = "Universal ARM Boot Manager for gaming handhelds and Raspberry Pi";