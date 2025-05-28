// src/mdns.rs
//
// mDNS service publisher and discovery for the application
// Uses libmdns 0.7.5 for publishing and mdns-sd for discovery
// 
// Works on Windows, macOS and Linux.

use std::{
    net::{IpAddr, Ipv4Addr, UdpSocket},
    sync::Arc,
    thread,
    time::Duration,
};

use tauri::{AppHandle, Manager, Emitter};
use libmdns::Responder;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::{Serialize, Deserialize};

use crate::state::AppState;

/// Convenience wrapper that tries to pick the first non-loopback IPv4
/// address of this host. Falls back to 127.0.0.1.
fn pick_ipv4() -> Ipv4Addr {
    // Create a UDP socket "to" 8.8.8.8 – no packets are actually sent,
    // but the OS will choose the outbound interface and give us its IP.
    let sock = UdpSocket::bind(("0.0.0.0", 0)).unwrap();
    if sock.connect(("8.8.8.8", 80)).is_ok() {
        if let Ok(local) = sock.local_addr() {
            if let IpAddr::V4(v4) = local.ip() {
                if !v4.is_loopback() {
                    return v4;
                }
            }
        }
    }
    Ipv4Addr::LOCALHOST
}

// MdnsState has been moved to state.rs

/// Start the mDNS service for the given port.
/// This makes the application discoverable on the local network.
/// 
/// Returns an error if the service cannot be started.
pub fn start_mdns_service(app: AppHandle, port: u16) -> anyhow::Result<()> {
    // Get the state from the app
    let state = app.state::<Arc<AppState>>();
    
    // Check if already running
    {
        let active = state.mdns.active.lock().unwrap();
        if *active {
            println!("[mDNS] Service already active, skipping");
            return Ok(());
        }
    }
    
    // Get the host information
    let host_ip = pick_ipv4();
    // let hostname = hostname::get()?;
    // let host_name = hostname.to_string_lossy().to_string();
    
    // Create TXT records as string slices
    // libmdns expects &[&str] for TXT records in the format "key=value"
    let txt_records = ["info=petbrain app"];
    
    // Create responder
    let responder = Responder::new().unwrap();
    
    // Register the service
    let service = responder.register(
        "_iot._tcp".to_string(),
        "petbrain".to_string(),
        port,
        &txt_records
    );
    
    // Update state
    {
        let mut responder_guard = state.mdns.responder.lock().unwrap();
        *responder_guard = Some(responder);
        
        let mut service_guard = state.mdns.service.lock().unwrap();
        *service_guard = Some(service);
        
        let mut host_guard = state.mdns.host.lock().unwrap();
        *host_guard = host_ip.to_string();
        
        let mut port_guard = state.mdns.port.lock().unwrap();
        *port_guard = port;
        
        let mut active_guard = state.mdns.active.lock().unwrap();
        *active_guard = true;
    }
    let report_msg = format!("[mDNS] Published petbrain._iot._tcp.local. at {}:{}", host_ip, port);
    app.emit("socket_status", report_msg.clone()).unwrap();
    println!("{}", report_msg);
    
    // Create a separate Arc for the background thread
    let state_for_thread = Arc::clone(&state);
    
    // Create a background thread for keeping the service alive
    // This avoids the AppHandle lifetime issue
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(25));
            
            // Check if we should exit
            let should_exit;
            {
                let active_guard = state_for_thread.mdns.active.lock().unwrap();
                should_exit = !*active_guard;
            }
            
            if should_exit {
                let report_msg = String::from("[mDNS] Background thread exiting");
                app.emit("socket_status", report_msg.clone()).unwrap();
                println!("{}", report_msg);
                break;
            }
            
            // Just log that we're still active
            let report_msg = String::from("[mDNS] Service still active");
            app.emit("socket_status", report_msg.clone()).unwrap();
            println!("{}", report_msg);
        }
    });
    
    Ok(())
}

// Structure to represent a discovered mDNS device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdnsDevice {
    pub name: String,     // Device name
    pub ip: String,       // IP address
    pub port: u16,        // Port number
    pub service_type: String,  // Service type
    pub txt_records: Vec<String>,  // TXT records
}

/// Discover devices on the local network using mDNS
/// Uses mdns-sd for discovery
pub fn discover_mdns_devices(app: AppHandle, service_type: String) -> anyhow::Result<()> {
    // Get the state from the app
    let state = app.state::<Arc<AppState>>();
    
    // Log that we're starting discovery
    println!("[mDNS] Starting discovery for {}", service_type);
    
    // Clear previous discoveries
    {
        let mut devices = state.mdns.discovered_devices.lock().unwrap();
        devices.clear();
        
        // Add some test devices for development purposes
        // #[cfg(debug_assertions)]
        // {
        //     println!("[mDNS] Debug mode: Adding test camera devices");
        //     // Add a few test devices to ensure we always see something in the UI
        //     devices.push(MdnsDevice {
        //         name: "Test Camera 1".to_string(),
        //         ip: "192.168.1.100".to_string(),
        //         port: 8080,
        //         service_type: "_rtsp._tcp.local.".to_string(),
        //         txt_records: vec!["camera=true".to_string()],
        //     });
            
        //     devices.push(MdnsDevice {
        //         name: "IP Webcam".to_string(),
        //         ip: "192.168.1.101".to_string(),
        //         port: 8081,
        //         service_type: "_rtsp._tcp.local.".to_string(),
        //         txt_records: vec!["camera=true".to_string()],
        //     });
        // }
    }
    
    let app_clone = app.clone();
    
    // Spawn a thread for discovery to avoid blocking the main thread
    thread::spawn(move || {
        // Try to create the daemon
        match ServiceDaemon::new() {
            Ok(mdns) => {
                // Log that we're starting discovery
                let msg = format!("[mDNS] Starting discovery for {}", service_type);
                app_clone.emit("socket_status", msg.clone()).unwrap_or_default();
                println!("{}", msg);
                
                // Browse for services
                match mdns.browse(&service_type) {
                    Ok(receiver) => {
                        // Set a timeout for discovery
                        let discovery_timeout = Duration::from_secs(3);
                        let start_time = std::time::Instant::now();
                        
                        // Collect devices as they're discovered
                        while start_time.elapsed() < discovery_timeout {
                            match receiver.recv_timeout(Duration::from_millis(100)) {
                                Ok(event) => {
                                    match event {
                                        ServiceEvent::ServiceResolved(info) => {
                                            // Get device details
                                            let name = info.get_hostname().to_string();
                                            
                                            // Get the first IPv4 address
                                            if let Some(addr) = info.get_addresses().iter().find(|a| a.is_ipv4()) {
                                                let ip = addr.to_string();
                                                let port = info.get_port();
                                                let service_type = info.get_type().to_string();
                                                
                                                // Get TXT records
                                                let txt_records: Vec<String> = info.get_properties()
                                                    .iter()
                                                    .map(|prop| {
                                                        let key = prop.key();
                                                        // Handle Option<&[u8]> by using unwrap_or_default
                                                        let value = match prop.val() {
                                                            Some(bytes) => String::from_utf8_lossy(bytes),
                                                            None => std::borrow::Cow::Borrowed("")
                                                        };
                                                        format!("{key}={value}")
                                                    })
                                                    .collect();
                                                
                                                // Create device info
                                                let device = MdnsDevice {
                                                    name: name.clone(),
                                                    ip,
                                                    port,
                                                    service_type,
                                                    txt_records,
                                                };
                                                
                                                // Add to state only if name contains "cam" (case insensitive)
                                                if name.to_lowercase().contains("cam") {
                                                    let state = app_clone.state::<Arc<AppState>>();
                                                    let mut devices = state.mdns.discovered_devices.lock().unwrap();
                                                    devices.push(device.clone());
                                                    
                                                    // Emit event with device info
                                                    let msg = format!("[mDNS] Discovered camera: {} at {}:{}", name, device.ip, device.port);
                                                    app_clone.emit("mdns_device_discovered", device).unwrap_or_default();
                                                    app_clone.emit("socket_status", msg.clone()).unwrap_or_default();
                                                    println!("{}", msg);
                                                }
                                                
                                                // Event emitting moved to inside the if statement above
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                Err(_) => {}
                                // Just continue if timeout, this is expected
                            }
                        }
                        
                        // Final emit of all discovered devices
                        let state = app_clone.state::<Arc<AppState>>();
                        let devices = state.mdns.discovered_devices.lock().unwrap().clone();
                        app_clone.emit("mdns_devices_list", devices).unwrap_or_default();
                        
                        // Log completion
                        let msg = format!("[mDNS] Discovery completed for {}", service_type);
                        app_clone.emit("socket_status", msg.clone()).unwrap_or_default();
                        println!("{}", msg);
                        
                        // Shutdown the daemon
                        if let Err(e) = mdns.shutdown() {
                            println!("[mDNS] Error shutting down daemon: {}", e);
                        }
                    },
                    Err(e) => {
                        let msg = format!("[mDNS] Error browsing for services: {}", e);
                        app_clone.emit("socket_status", msg.clone()).unwrap_or_default();
                        println!("{}", msg);
                    }
                }
            },
            Err(e) => {
                let msg = format!("[mDNS] Error creating mDNS daemon: {}", e);
                app_clone.emit("socket_status", msg.clone()).unwrap_or_default();
                println!("{}", msg);
            }
        }
    });
    
    Ok(())
}

/// Get the list of discovered mDNS devices
#[allow(dead_code)]
pub fn get_discovered_devices(app: &AppHandle) -> Vec<MdnsDevice> {
    let state = app.state::<Arc<AppState>>();
    let devices = state.mdns.discovered_devices.lock().unwrap();
    devices.clone()
}

/// Stop the mDNS service.
/// This is safe to call even if the service is not running.
pub fn stop_mdns_service(app: &AppHandle) -> anyhow::Result<()> {
    let state = app.state::<Arc<AppState>>();
    
    // Check if service is active
    {
        let active_guard = state.mdns.active.lock().unwrap();
        if !*active_guard {
            println!("[mDNS] No active service to stop");
            app.emit("socket_status", String::from("[mDNS] No active service to stop")).unwrap();
            return Ok(());
        }
    }
    
    // Unregister the service
    {
        let mut responder_guard = state.mdns.responder.lock().unwrap();
        let mut service_guard = state.mdns.service.lock().unwrap();
        
        // Just drop the service and responder
        // The service is automatically unregistered when dropped
        if service_guard.is_some() { service_guard.take(); }
        if responder_guard.is_some() { responder_guard.take(); }
    }
    
    // Mark as inactive
    {
        let mut active_guard = state.mdns.active.lock().unwrap();
        *active_guard = false;
    }
    
    println!("[mDNS] Service stopped");
    app.emit("socket_status", String::from("[mDNS] Service stopped")).unwrap();
    Ok(())
}
