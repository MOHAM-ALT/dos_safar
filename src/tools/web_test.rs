use anyhow::Result;
use dos_safar::utils::config::Config;
use dos_safar::utils::logger::init_logger;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    info("🌐 DOS Safar Web Server Test Tool");
    info http://localhost:8080");
ECHO is off.
    // Simple web server for testing
    let listener = std::net::TcpListener::bind("0.0.0.0:8080")?;
    info("Server listening on port 8080");
ECHO is off.
    for stream in listener.incoming() {
        let _stream = stream?;
        // Handle connections here
    }
ECHO is off.
    Ok(())
}
