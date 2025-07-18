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
use hardware::lcd_display::LcdDisplayDetector; // إضافة جديدةuse bootloader::menu::BootMenu;
use remote::web_server::WebServer;
use utils::config::Config;
use utils::logger::init_logger;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logger()?;
    
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

    info!("🎮 Starting DOS Safar Boot Manager");
    info!("📁 Configuration: {}", config_path);

    // Load configuration
    let config = Config::load(config_path)?;
    
    // If web-only mode, start web server and exit
    if web_only {
        info!("🌐 Starting in web-only mode for development");
        start_web_server(&config).await?;
        return Ok(());
    }

    // Phase 1: Device Detection
    info!("🔍 === Phase 1: Device Detection ===");
    let device_detector = DeviceDetector::new();
    let device_info = device_detector.detect_device().await?;
    info!("✅ Detected device: {} ({})", device_info.model, device_info.architecture);

    // Phase 2: Show boot options with keyboard interrupt detection
    info!("⏰ === Phase 2: Boot Timeout ({}s) ===", config.boot.menu_timeout_seconds);
    println!("\n🎮 DOS Safar Boot Manager");
    println!("Device: {}", device_info.model);
    println!("═══════════════════════════════════════");
    println!("Press ANY KEY to access boot menu...");
    println!("Or wait {} seconds for automatic web interface", config.boot.menu_timeout_seconds);
    println!("═══════════════════════════════════════");

    // Wait for keyboard input or timeout
    let user_interrupted = wait_for_keyboard_or_timeout(&config).await;

    if user_interrupted {
        info!("⌨️  User input detected - showing boot menu");
        
        // Phase 2a: Hardware Testing (if requested)
        if !skip_tests {
            info!("🔧 === Hardware Testing ===");
            run_hardware_tests(&device_info).await?;
        }

        // Phase 2b: Show boot menu
        info!("📋 === Boot Menu ===");
        let boot_menu = BootMenu::new(&config, &device_info)?;
        boot_menu.show_menu().await?;
        
    } else {
        info!("⏱️  Timeout reached - starting automatic web interface");
        
        // Phase 3: Smart Network Auto-Connect
        info!("🌐 === Phase 3: Smart Network Connection ===");
        let network_result = auto_connect_and_start_web(&config).await;
        
        match network_result {
            Ok(connection) => {
                info!("✅ Web interface started successfully");
                
                // Keep the system running
                info!("🔄 System ready - web interface active");
                loop {
                    sleep(Duration::from_secs(60)).await;
                }
            }
            Err(e) => {
                warn!("❌ Failed to start web interface: {}", e);
                info!("📋 Falling back to boot menu...");
                
                let boot_menu = BootMenu::new(&config, &device_info)?;
                boot_menu.show_menu().await?;
            }
        }
    }

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
// إضافة قبل "All tests completed"
// Test LCD displays
info!("Testing LCD displays...");
let lcd_detector = LcdDisplayDetector::new(device_info);
let lcd_result = lcd_detector.detect_lcd_displays().await;
match lcd_result {
    Ok(displays) => {
        info!("Found {} LCD displays", displays.len());
        
        // Test each LCD display
        for display in displays {
            info!("Testing LCD: {:?} - {}\"", display.driver, display.size_inch);
            if let Ok(test_passed) = lcd_detector.test_lcd_display(&display).await {
                if test_passed {
                    info!("LCD display test passed");
                    
                    // Configure the LCD display
                    if let Err(e) = lcd_detector.configure_lcd_display(&display).await {
                        warn!("LCD configuration failed: {}", e);
                    } else {
                        info!("LCD display configured successfully");
                    }
                } else {
                    warn!("LCD display test failed");
                }
            }
        }
    },
    Err(e) => warn!("LCD detection failed: {}", e),
}
    // All tests completed
    info!("Hardware tests completed");
    Ok(())
}

// Smart keyboard detection with timeout
async fn wait_for_keyboard_or_timeout(config: &Config) -> bool {
    use std::io::{self, Read};
    use std::sync::mpsc;
    use std::thread;
    
    let (tx, rx) = mpsc::channel();
    
    // Spawn thread to listen for keyboard input
    thread::spawn(move || {
        let mut stdin = io::stdin();
        let mut buffer = [0; 1];
        
        // Non-blocking read attempt
        if stdin.read(&mut buffer).is_ok() {
            let _ = tx.send(true);
        }
    });
    
    // Wait for either keyboard input or timeout
    match tokio::time::timeout(
        Duration::from_secs(config.boot.menu_timeout_seconds), 
        tokio::task::spawn_blocking(move || rx.recv())
    ).await {
        Ok(Ok(Ok(_))) => {
            info!("⌨️  Keyboard input detected!");
            true
        }
        _ => {
            info!("⏱️  No keyboard input - proceeding with auto-connect");
            false
        }
    }
}

// Smart auto-connect and web interface startup
async fn auto_connect_and_start_web(config: &Config) -> Result<()> {
    use crate::hardware::enhanced_network::SmartNetworkManager;
    
    let network_manager = SmartNetworkManager::new(config);
    
    // Try to connect to network
    println!("🔍 Searching for networks...");
    match network_manager.auto_connect().await {
        Ok(connection) => {
            // Display connection info on screen
            network_manager.display_connection_info(&connection);
            
            // Start web server
            info!("🚀 Starting web interface...");
            tokio::spawn(async move {
                if let Err(e) = start_web_server(config).await {
                    error!("❌ Web server error: {}", e);
                }
            });
            
            // Wait a moment for web server to start
            sleep(Duration::from_secs(2)).await;
            
            println!("✅ Web interface is ready!");
            println!("📱 Open your browser/phone and go to: http://{}", connection.ip_address);
            println!("🔧 Use the web interface to:");
            println!("   • View current screen");
            println!("   • Fix display/keyboard issues");
            println!("   • Manage operating systems");
            println!("   • Change settings");
            
            Ok(())
        }
        Err(e) => {
            error!("❌ Network connection failed: {}", e);
            println!("\n⚠️  No network connection available");
            println!("Options:");
            println!("1. Check network settings in config/default.toml");
            println!("2. Connect Ethernet cable");
            println!("3. Restart to try again");
            
            Err(e)
        }
    }
}