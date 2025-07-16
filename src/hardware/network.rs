use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, warn};
use crate::utils::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub interface: String,
    pub connection_type: ConnectionType,
    pub ip_address: String,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub is_connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    Ethernet,
    WiFi,
    Hotspot,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiNetwork {
    pub ssid: String,
    pub signal_strength: i32,
    pub security: String,
    pub frequency: Option<u32>,
}

pub struct NetworkManager {
    config: Config,
}

impl NetworkManager {
    pub fn new(config: &Config) -> Self {
        NetworkManager {
            config: config.clone(),
        }
    }

    pub async fn connect(&self) -> Result<NetworkConnection> {
        info!("Starting network connection process");

        // Try Ethernet first if preferred
        if self.config.network.ethernet_preferred {
            if let Ok(connection) = self.try_ethernet_connection().await {
                info!("Connected via Ethernet: {}", connection.ip_address);
                return Ok(connection);
            }
        }

        // Try WiFi connection
        if let Ok(connection) = self.try_wifi_connection().await {
            info!("Connected via WiFi: {}", connection.ip_address);
            return Ok(connection);
        }

        // Try Ethernet as fallback if not preferred
        if !self.config.network.ethernet_preferred {
            if let Ok(connection) = self.try_ethernet_connection().await {
                info!("Connected via Ethernet (fallback): {}", connection.ip_address);
                return Ok(connection);
            }
        }

        Err(anyhow::anyhow!("Failed to establish any network connection"))
    }

    async fn try_ethernet_connection(&self) -> Result<NetworkConnection> {
        info!("Attempting Ethernet connection");

        // Check if Ethernet interface exists
        let eth_interfaces = self.get_ethernet_interfaces().await?;
        if eth_interfaces.is_empty() {
            return Err(anyhow::anyhow!("No Ethernet interfaces found"));
        }

        for interface in eth_interfaces {
            debug!("Checking Ethernet interface: {}", interface);
            
            // Check if interface is up and has link
            if self.is_interface_up(&interface).await? {
                // Try to get IP address
                if let Ok(ip) = self.get_interface_ip(&interface).await {
                    let connection = NetworkConnection {
                        interface: interface.clone(),
                        connection_type: ConnectionType::Ethernet,
                        ip_address: ip,
                        gateway: self.get_default_gateway().await.ok(),
                        dns_servers: self.get_dns_servers().await.unwrap_or_default(),
                        is_connected: true,
                    };
                    
                    // Test connectivity
                    if self.test_internet_connectivity().await {
                        return Ok(connection);
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Ethernet connection failed"))
    }

    async fn try_wifi_connection(&self) -> Result<NetworkConnection> {
        info!("Attempting WiFi connection");

        // Check if WiFi interface exists
        let wifi_interfaces = self.get_wifi_interfaces().await?;
        if wifi_interfaces.is_empty() {
            return Err(anyhow::anyhow!("No WiFi interfaces found"));
        }

        for interface in wifi_interfaces {
            debug!("Checking WiFi interface: {}", interface);
            
            // Try to connect to configured network
            if let Some(ssid) = &self.config.network.wifi_ssid {
                if !ssid.is_empty() {
                    if let Ok(connection) = self.connect_to_wifi(&interface, ssid).await {
                        return Ok(connection);
                    }
                }
            }

            // Try to connect to any available open network
            if let Ok(connection) = self.connect_to_open_wifi(&interface).await {
                return Ok(connection);
            }
        }

        Err(anyhow::anyhow!("WiFi connection failed"))
    }

    async fn get_ethernet_interfaces(&self) -> Result<Vec<String>> {
        let mut interfaces = Vec::new();
        
        // Check /sys/class/net for network interfaces
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let interface_name = entry.file_name().to_string_lossy().to_string();
                
                // Check if it's an Ethernet interface
                if interface_name.starts_with("eth") || 
                   interface_name.starts_with("enp") || 
                   interface_name.starts_with("eno") {
                    interfaces.push(interface_name);
                }
            }
        }

        Ok(interfaces)
    }

    async fn get_wifi_interfaces(&self) -> Result<Vec<String>> {
        let mut interfaces = Vec::new();
        
        // Check /sys/class/net for wireless interfaces
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let interface_name = entry.file_name().to_string_lossy().to_string();
                
                // Check if it's a wireless interface
                if interface_name.starts_with("wlan") || 
                   interface_name.starts_with("wlp") || 
                   interface_name.starts_with("wlx") {
                    // Verify it's actually a wireless interface
                    let wireless_path = format!("/sys/class/net/{}/wireless", interface_name);
                    if std::path::Path::new(&wireless_path).exists() {
                        interfaces.push(interface_name);
                    }
                }
            }
        }

        Ok(interfaces)
    }

    async fn is_interface_up(&self, interface: &str) -> Result<bool> {
        let operstate_path = format!("/sys/class/net/{}/operstate", interface);
        
        if let Ok(state) = fs::read_to_string(&operstate_path) {
            Ok(state.trim() == "up")
        } else {
            Ok(false)
        }
    }

    async fn get_interface_ip(&self, interface: &str) -> Result<String> {
        // Use ip command to get interface IP
        let output = Command::new("ip")
            .args(&["addr", "show", interface])
            .output()
            .context("Failed to run ip command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("ip command failed"));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse output to find inet address
        for line in output_str.lines() {
            if line.trim().starts_with("inet ") {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let ip_with_cidr = parts[1];
                    if let Some(ip) = ip_with_cidr.split('/').next() {
                        return Ok(ip.to_string());
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No IP address found for interface"))
    }

    async fn get_default_gateway(&self) -> Result<String> {
        let output = Command::new("ip")
            .args(&["route", "show", "default"])
            .output()
            .context("Failed to get default route")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get default route"));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        
        // Parse output to find gateway IP
        for line in output_str.lines() {
            if line.contains("default via") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pos) = parts.iter().position(|&x| x == "via") {
                    if pos + 1 < parts.len() {
                        return Ok(parts[pos + 1].to_string());
                    }
                }
            }
        }

        Err(anyhow::anyhow!("No default gateway found"))
    }

    async fn get_dns_servers(&self) -> Result<Vec<String>> {
        let mut dns_servers = Vec::new();

        // Read /etc/resolv.conf
        if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
            for line in content.lines() {
                if line.starts_with("nameserver") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        dns_servers.push(parts[1].to_string());
                    }
                }
            }
        }

        Ok(dns_servers)
    }

    async fn connect_to_wifi(&self, interface: &str, ssid: &str) -> Result<NetworkConnection> {
        info!("Connecting to WiFi network: {}", ssid);

        // Bring interface up
        self.bring_interface_up(interface).await?;

        // Scan for the network
        let networks = self.scan_wifi_networks(interface).await?;
        if !networks.iter().any(|n| n.ssid == ssid) {
            return Err(anyhow::anyhow!("Network {} not found", ssid));
        }

        // Connect using wpa_supplicant or NetworkManager
        if let Ok(connection) = self.connect_with_wpa_supplicant(interface, ssid).await {
            return Ok(connection);
        }

        Err(anyhow::anyhow!("Failed to connect to WiFi network"))
    }

    async fn connect_to_open_wifi(&self, interface: &str) -> Result<NetworkConnection> {
        info!("Scanning for open WiFi networks");

        let networks = self.scan_wifi_networks(interface).await?;
        
        // Find open networks (no security)
        let open_networks: Vec<&WiFiNetwork> = networks.iter()
            .filter(|n| n.security.is_empty() || n.security == "Open")
            .collect();

        if open_networks.is_empty() {
            return Err(anyhow::anyhow!("No open WiFi networks found"));
        }

        // Try to connect to the strongest open network
        let best_network = open_networks.iter()
            .max_by_key(|n| n.signal_strength)
            .unwrap();

        info!("Connecting to open network: {}", best_network.ssid);
        
        // Connect to open network
        self.connect_to_open_network(interface, &best_network.ssid).await
    }

    async fn scan_wifi_networks(&self, interface: &str) -> Result<Vec<WiFiNetwork>> {
        // Use iwlist to scan for networks
        let output = Command::new("iwlist")
            .args(&[interface, "scan"])
            .output()
            .context("Failed to scan WiFi networks")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("WiFi scan failed"));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(self.parse_iwlist_output(&output_str))
    }

    fn parse_iwlist_output(&self, output: &str) -> Vec<WiFiNetwork> {
        let mut networks = Vec::new();
        let mut current_network: Option<WiFiNetwork> = None;

        for line in output.lines() {
            let line = line.trim();
            
            if line.starts_with("Cell ") {
                // Save previous network if exists
                if let Some(network) = current_network.take() {
                    networks.push(network);
                }
                
                // Start new network
                current_network = Some(WiFiNetwork {
                    ssid: String::new(),
                    signal_strength: 0,
                    security: String::new(),
                    frequency: None,
                });
            } else if let Some(ref mut network) = current_network {
                if line.starts_with("ESSID:") {
                    let ssid = line.strip_prefix("ESSID:").unwrap_or("")
                        .trim_matches('"');
                    network.ssid = ssid.to_string();
                } else if line.starts_with("Quality=") {
                    // Parse signal quality
                    if let Some(quality_part) = line.split_whitespace().next() {
                        if let Some(quality_str) = quality_part.strip_prefix("Quality=") {
                            if let Some(numerator) = quality_str.split('/').next() {
                                if let Ok(quality) = numerator.parse::<i32>() {
                                    network.signal_strength = quality;
                                }
                            }
                        }
                    }
                } else if line.contains("Encryption key:off") {
                    network.security = "Open".to_string();
                } else if line.contains("WPA") || line.contains("WEP") {
                    network.security = "Secured".to_string();
                }
            }
        }

        // Add the last network
        if let Some(network) = current_network {
            networks.push(network);
        }

        networks
    }

    async fn bring_interface_up(&self, interface: &str) -> Result<()> {
        let output = Command::new("ip")
            .args(&["link", "set", interface, "up"])
            .output()
            .context("Failed to bring interface up")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to bring interface up"));
        }

        Ok(())
    }

    async fn connect_with_wpa_supplicant(&self, interface: &str, ssid: &str) -> Result<NetworkConnection> {
        // This is a simplified implementation
        // In a real implementation, you would generate wpa_supplicant.conf
        // and manage the connection properly
        
        // For now, try to connect using iwconfig for open networks
        if let Some(password) = &self.config.network.wifi_password {
            if !password.is_empty() {
                // TODO: Implement WPA/WPA2 connection
                warn!("WPA/WPA2 connection not implemented yet");
            }
        }

        self.connect_to_open_network(interface, ssid).await
    }

    async fn connect_to_open_network(&self, interface: &str, ssid: &str) -> Result<NetworkConnection> {
        // Connect to open network using iwconfig
        let output = Command::new("iwconfig")
            .args(&[interface, "essid", ssid])
            .output()
            .context("Failed to connect to network")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to set ESSID"));
        }

        // Wait a moment for connection
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Try to get IP via DHCP
        let dhcp_output = Command::new("dhclient")
            .arg(interface)
            .output()
            .context("Failed to run DHCP client")?;

        // Wait for DHCP
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Get IP address
        let ip = self.get_interface_ip(interface).await?;

        Ok(NetworkConnection {
            interface: interface.to_string(),
            connection_type: ConnectionType::WiFi,
            ip_address: ip,
            gateway: self.get_default_gateway().await.ok(),
            dns_servers: self.get_dns_servers().await.unwrap_or_default(),
            is_connected: true,
        })
    }

    async fn test_internet_connectivity(&self) -> bool {
        // Try to ping a reliable server
        let ping_test = timeout(
            Duration::from_secs(3),
            Command::new("ping")
                .args(&["-c", "1", "-W", "2", "8.8.8.8"])
                .output()
        ).await;

        match ping_test {
            Ok(Ok(output)) => {
                let success = output.status.success();
                debug!("Internet connectivity test: {}", if success { "passed" } else { "failed" });
                success
            }
            _ => {
                debug!("Internet connectivity test: timeout/error");
                false
            }
        }
    }

    pub async fn get_local_ip(&self) -> Option<String> {
        // Get the first non-loopback IP address
        if let Ok(interfaces) = self.get_all_interfaces().await {
            for interface in interfaces {
                if interface != "lo" {
                    if let Ok(ip) = self.get_interface_ip(&interface).await {
                        if !ip.starts_with("127.") {
                            return Some(ip);
                        }
                    }
                }
            }
        }
        None
    }

    async fn get_all_interfaces(&self) -> Result<Vec<String>> {
        let mut interfaces = Vec::new();
        
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                let interface_name = entry.file_name().to_string_lossy().to_string();
                interfaces.push(interface_name);
            }
        }

        Ok(interfaces)
    }
}