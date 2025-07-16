use anyhow::Result;
use clap::{Arg, Command};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, error};

mod hardware;
mod bootloader;
mod remote;
mod utils;

use hardware::device_detect::DeviceDetector;
use hardware::display::DisplayTester;
use hardware::input::InputTester;
use hardware::network::NetworkManager;
use bootloader::menu::BootMenu;
use remote::web_server::WebServer;
use utils::config::Config;
use utils::logger::init_logger;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logger()?;
    
    // Parse command line arguments
    let matches = Command::new("DOS Safar")
        .version("0.1.0")
        .about("Universal ARM Boot Manager for gaming handhelds and Raspberry Pi")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .help("Configuration file path")
            .default_value("config/default.toml"))
        .arg(Arg::new("skip-tests")
            .long("skip-tests")
            .help("Skip hardware tests and go directly to boot menu")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("web-only")
            .long("web-only")
            .help("Start only web interface (for development)")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let skip_tests = matches.get_flag("skip-tests");
    let web_only = matches.get_flag("web-only");

    info!("Starting DOS Safar Boot Manager");
    info!("Configuration: {}", config_path);

    // Load configuration
    let config = Config::load(config_path)?;
    
    // If web-only mode, start web server and exit
    if web_only {
        info!("Starting in web-only mode for development");
        start_web_server(&config).await?;
        return Ok(());
    }

    // Phase 1: Device Detection
    info!("=== Phase 1: Device Detection ===");
    let device_detector = DeviceDetector::new();
    let device_info = device_detector.detect_device().await?;
    info!("Detected device: {} ({})", device_info.model, device_info.architecture);

    // Phase 2: Hardware Testing (unless skipped)
    if !skip_tests {
        info!("=== Phase 2: Hardware Testing ===");
        run_hardware_tests(&device_info).await?;
    } else {
        info!("Skipping hardware tests as requested");
    }

    // Phase 3: Network Connection with 3-second timeout
    info!("=== Phase 3: Network Connection ===");
    let network_manager = NetworkManager::new(&config);
    let network_connected = connect_with_timeout(&network_manager).await;

    // Phase 4: Start Web Server (if network available)
    if network_connected {
        info!("=== Phase 4: Starting Web Interface ===");
        tokio::spawn(async move {
            if let Err(e) = start_web_server(&config).await {
                error!("Web server error: {}", e);
            }
        });
    }

    // Phase 5: Boot Menu
    info!("=== Phase 5: Boot Menu ===");
    let boot_menu = BootMenu::new(&config, &device_info)?;
    boot_menu.show_menu().await?;

    Ok(())
}

async fn run_hardware_tests(device_info: &hardware::device_detect::DeviceInfo) -> Result<()> {
    info!("Running hardware tests for {}", device_info.model);

    // Test display
    info!("Testing display configuration...");
    let display_tester = DisplayTester::new(device_info);
    let display_result = display_tester.test_display().await;
    match display_result {
        Ok(config) => info!("Display test passed: {}x{}", config.width, config.height),
        Err(e) => warn!("Display test failed: {}", e),
    }

    // Test input devices
    info!("Testing input devices...");
    let input_tester = InputTester::new(device_info);
    let input_result = input_tester.test_controllers().await;
    match input_result {
        Ok(controllers) => info!("Found {} input devices", controllers.len()),
        Err(e) => warn!("Input test failed: {}", e),
    }

    // All tests completed
    info!("Hardware tests completed");
    Ok(())
}

async fn connect_with_timeout(network_manager: &NetworkManager) -> bool {
    info!("Attempting network connection (3 second timeout)...");
    
    // Try to connect with 3-second timeout
    match tokio::time::timeout(Duration::from_secs(3), network_manager.connect()).await {
        Ok(Ok(connection)) => {
            info!("Network connected: {}", connection.ip_address);
            info!("Web interface will be available at: http://{}", connection.ip_address);
            true
        }
        Ok(Err(e)) => {
            warn!("Network connection failed: {}", e);
            false
        }
        Err(_) => {
            warn!("Network connection timeout (3 seconds)");
            false
        }
    }
}

async fn start_web_server(config: &Config) -> Result<()> {
    let web_server = WebServer::new(config)?;
    web_server.start().await
}