// src/mdns.rs
//
// Minimal but complete mDNS publisher for "petbrain._iot._tcp.local."
// Uses libmdns 0.7.5 and Tauri 2.
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
