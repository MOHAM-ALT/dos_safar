use anyhow::Result;
use dos_safar::utils::config::Config;
use dos_safar::remote::web_server::WebServer;
use dos_safar::utils::logger::init_logger;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logger()?;
    
    info!("🌐 DOS Safar Web Server Test Tool");
    info!("==================================");
    
    // Create a test configuration
    let mut config = Config::default();
    config.web.port = 8080;
    config.web.host = "0.0.0.0".to_string();
    
    info!("Test configuration:");
    info!("  Host: {}", config.web.host);
    info!("  Port: {}", config.web.port);
    info!("  Static files: {}", config.web.static_files_path);
    
    // Start web server
    info!("Starting web server for testing...");
    info!("Open your browser and go to: http://localhost:{}", config.web.port);
    info!("Press Ctrl+C to stop the server");
    
    let web_server = WebServer::new(&config)?;
    web_server.start().await?;
    
    Ok(())
}