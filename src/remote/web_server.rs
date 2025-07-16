use anyhow::{Context, Result};
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
};
use tracing::{info, warn};
use crate::utils::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub device_model: String,
    pub uptime: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub temperature: Option<f32>,
    pub network_status: NetworkStatus,
    pub available_systems: Vec<OSInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub connected: bool,
    pub interface: String,
    pub ip_address: String,
    pub signal_strength: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OSInfo {
    pub name: String,
    pub description: String,
    pub size_mb: u64,
    pub bootable: bool,
    pub last_used: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BootRequest {
    pub os_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ControllerInput {
    pub button: String,
    pub pressed: bool,
}

pub struct WebServer {
    config: Config,
}

impl WebServer {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(WebServer {
            config: config.clone(),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let app = self.create_app().await?;
        
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.web.port));
        info!("🌐 Starting DOS Safar web server on {}", addr);
        info!("📱 Mobile-friendly interface available");
        
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .context("Failed to start web server")?;

        Ok(())
    }

    async fn create_app(&self) -> Result<Router> {
        let app = Router::new()
            // API routes
            .route("/api/status", get(get_system_status))
            .route("/api/systems", get(get_available_systems))
            .route("/api/boot", post(boot_system))
            .route("/api/input", post(send_input))
            .route("/api/screenshot", get(get_screenshot))
            .route("/api/files/upload", post(upload_file))
            .route("/api/network/scan", get(scan_networks))
            .route("/api/network/connect", post(connect_network))
            
            // Web interface routes
            .route("/", get(serve_index))
            .route("/remote", get(serve_remote_control))
            .route("/systems", get(serve_systems_manager))
            .route("/settings", get(serve_settings))
            .route("/troubleshoot", get(serve_troubleshoot)) // ← صفحة الـ troubleshooting الجديدة
            
            // Static files
            .nest_service("/static", ServeDir::new(&self.config.web.static_files_path))
            
            // Middleware
            .layer(ServiceBuilder::new()
                .layer(CorsLayer::permissive())
            );

        Ok(app)
    }
}

// API Handlers

async fn get_system_status() -> Result<Json<SystemStatus>, StatusCode> {
    let status = SystemStatus {
        device_model: "Raspberry Pi 4B".to_string(), // This would come from device detection
        uptime: get_system_uptime().unwrap_or_else(|_| "Unknown".to_string()),
        cpu_usage: get_cpu_usage().unwrap_or(0.0),
        memory_usage: get_memory_usage().unwrap_or(0.0),
        temperature: get_cpu_temperature().ok(),
        network_status: NetworkStatus {
            connected: true,
            interface: "wlan0".to_string(),
            ip_address: "192.168.1.100".to_string(),
            signal_strength: Some(-45),
        },
        available_systems: vec![
            OSInfo {
                name: "RetroPie".to_string(),
                description: "Retro Gaming System".to_string(),
                size_mb: 4096,
                bootable: true,
                last_used: Some("2024-01-15T10:30:00Z".to_string()),
            },
            OSInfo {
                name: "Raspberry Pi OS".to_string(),
                description: "Official Raspberry Pi OS".to_string(),
                size_mb: 8192,
                bootable: true,
                last_used: None,
            },
        ],
    };

    Ok(Json(status))
}

async fn get_available_systems() -> Result<Json<Vec<OSInfo>>, StatusCode> {
    // This would scan for actual systems
    let systems = vec![
        OSInfo {
            name: "RetroPie".to_string(),
            description: "Retro Gaming System".to_string(),
            size_mb: 4096,
            bootable: true,
            last_used: Some("2024-01-15T10:30:00Z".to_string()),
        },
        OSInfo {
            name: "Batocera".to_string(),
            description: "Gaming Distribution".to_string(),
            size_mb: 2048,
            bootable: true,
            last_used: None,
        },
    ];

    Ok(Json(systems))
}

async fn boot_system(Json(request): Json<BootRequest>) -> Result<Json<HashMap<String, String>>, StatusCode> {
    info!("Boot request received for: {}", request.os_name);
    
    // This would trigger the actual boot process
    let mut response = HashMap::new();
    response.insert("status".to_string(), "success".to_string());
    response.insert("message".to_string(), format!("Booting {}", request.os_name));
    
    Ok(Json(response))
}

async fn send_input(Json(input): Json<ControllerInput>) -> Result<Json<HashMap<String, String>>, StatusCode> {
    info!("Input received: {} = {}", input.button, input.pressed);
    
    // This would send the input to the running system
    let mut response = HashMap::new();
    response.insert("status".to_string(), "received".to_string());
    
    Ok(Json(response))
}

async fn get_screenshot() -> Result<Json<HashMap<String, String>>, StatusCode> {
    // This would capture the current screen
    let mut response = HashMap::new();
    response.insert("image_data".to_string(), "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==".to_string());
    
    Ok(Json(response))
}

async fn upload_file() -> Result<Json<HashMap<String, String>>, StatusCode> {
    // This would handle OS image uploads
    let mut response = HashMap::new();
    response.insert("status".to_string(), "uploaded".to_string());
    response.insert("message".to_string(), "File uploaded successfully".to_string());
    
    Ok(Json(response))
}

async fn scan_networks() -> Result<Json<Vec<HashMap<String, String>>>, StatusCode> {
    // This would scan for WiFi networks
    let networks = vec![
        {
            let mut network = HashMap::new();
            network.insert("ssid".to_string(), "Gaming_Network".to_string());
            network.insert("signal".to_string(), "-45".to_string());
            network.insert("security".to_string(), "WPA2".to_string());
            network
        },
        {
            let mut network = HashMap::new();
            network.insert("ssid".to_string(), "Public_WiFi".to_string());
            network.insert("signal".to_string(), "-60".to_string());
            network.insert("security".to_string(), "Open".to_string());
            network
        },
    ];
    
    Ok(Json(networks))
}

async fn connect_network() -> Result<Json<HashMap<String, String>>, StatusCode> {
    // This would connect to a WiFi network
    let mut response = HashMap::new();
    response.insert("status".to_string(), "connected".to_string());
    response.insert("ip".to_string(), "192.168.1.101".to_string());
    
    Ok(Json(response))
}

// Web Interface Handlers

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../../assets/web/index.html"))
}

async fn serve_remote_control() -> Html<&'static str> {
    Html(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DOS Safar - Remote Control</title>
    <style>
        body { 
            margin: 0; 
            padding: 20px; 
            font-family: Arial, sans-serif; 
            background: #1a1a1a; 
            color: white;
        }
        .container { max-width: 800px; margin: 0 auto; }
        .screen { 
            width: 100%; 
            height: 300px; 
            background: #000; 
            border: 2px solid #333; 
            margin-bottom: 20px;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .controls { 
            display: grid; 
            grid-template-columns: 1fr 1fr; 
            gap: 20px; 
        }
        .dpad {
            display: grid;
            grid-template-columns: 1fr 1fr 1fr;
            grid-template-rows: 1fr 1fr 1fr;
            gap: 5px;
            max-width: 150px;
        }
        .btn {
            background: #333;
            border: none;
            color: white;
            padding: 15px;
            border-radius: 8px;
            font-size: 16px;
            cursor: pointer;
            touch-action: manipulation;
        }
        .btn:active { background: #555; }
        .btn:nth-child(2) { grid-column: 2; }
        .btn:nth-child(4) { grid-column: 1; grid-row: 2; }
        .btn:nth-child(5) { grid-column: 3; grid-row: 2; }
        .btn:nth-child(6) { grid-column: 2; grid-row: 3; }
        .action-buttons {
            display: grid;
            grid-template-columns: 1fr 1fr;
            grid-template-rows: 1fr 1fr;
            gap: 10px;
            max-width: 150px;
            margin-left: auto;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎮 DOS Safar Remote Control</h1>
        
        <div class="screen">
            <div>📺 Screen Sharing (Coming Soon)</div>
        </div>
        
        <div class="controls">
            <div>
                <h3>D-Pad</h3>
                <div class="dpad">
                    <div></div>
                    <button class="btn" data-button="up">↑</button>
                    <div></div>
                    <button class="btn" data-button="left">←</button>
                    <div></div>
                    <button class="btn" data-button="right">→</button>
                    <div></div>
                    <button class="btn" data-button="down">↓</button>
                    <div></div>
                </div>
            </div>
            
            <div>
                <h3>Action Buttons</h3>
                <div class="action-buttons">
                    <button class="btn" data-button="y">Y</button>
                    <button class="btn" data-button="x">X</button>
                    <button class="btn" data-button="b">B</button>
                    <button class="btn" data-button="a">A</button>
                </div>
            </div>
        </div>
        
        <div style="margin-top: 20px;">
            <button class="btn" data-button="start" style="margin-right: 10px;">START</button>
            <button class="btn" data-button="select">SELECT</button>
        </div>
    </div>
    
    <script>
        document.querySelectorAll('.btn').forEach(btn => {
            btn.addEventListener('touchstart',