use crate::mdns;
use crate::reader::{
    reader_loop, FakeBinaryReader, SerialBinaryReader, SocketBinaryReader,
};
use crate::state::AppState;
use crate::types::{FakeDataConfig};
use serialport;
use std::{
    sync::{Arc, atomic::Ordering, mpsc}, thread,
};
use tauri::{AppHandle, Emitter, Manager, State};
use chrono;

#[tauri::command]
pub fn connect_serial(
    app_handle: AppHandle,
    port: String,
    baud_rate: u32,
    stop_bits: u8,
    parity: String,
    data_bits: u8,
    state: State<Arc<AppState>>,
) -> Result<(), String> {
    if state.stream.signal_stream_running.load(Ordering::SeqCst) {
        state.stream.signal_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.stream.signal_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    let (tx, rx) = mpsc::channel::<String>();
    *state.communication.outbound_tx.lock().unwrap() = Some(tx);

    let reader = SerialBinaryReader::new(port.clone(), baud_rate, stop_bits, parity, data_bits, rx);
    state.stream.signal_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.stream.signal_stream_running.clone();
    let app_clone = app_handle.clone();
    let state_clone = state.inner().clone();
    let handle = thread::spawn(move || {
        reader_loop(reader, running_flag, state_clone, app_clone);
    });
    *state.stream.signal_stream_handle.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
pub fn connect_socket(
    app_handle: AppHandle,
    host: String,
    port: u16,
) -> Result<(), String> {
    // Send a clear status message to reset UI state
    let _ = app_handle.emit("socket_status", format!("[SOCKET] Attempting to connect to {}:{}", host, port));
    
    let state = app_handle.state::<Arc<AppState>>();
    if state.stream.signal_stream_running.load(Ordering::SeqCst) {
        state.stream.signal_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.stream.signal_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    // Start mDNS service to advertise the socket connection
    // This will make the app discoverable on the local network
    if let Err(e) = mdns::start_mdns_service(app_handle.clone(), 8080) {
        println!("[SOCKET] Warning: Failed to start mDNS service: {}", e);
        // Continue even if mDNS fails, as it's not critical for the connection
    } else {
        println!("[SOCKET] mDNS service started successfully");
    }

    // Create the reader but don't set it up yet - setup will be done in reader_loop
    // Pass the app_handle to the reader so it can emit socket status events
    let reader = SocketBinaryReader::new(host.clone(), port)
        .with_app_handle(app_handle.clone());
    state.stream.signal_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.stream.signal_stream_running.clone();
    let app_clone = app_handle.clone();
    let state_clone = state.inner().clone();
    let handle = thread::spawn(move || {
        reader_loop(reader, running_flag, state_clone, app_clone);
    });
    *state.stream.signal_stream_handle.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
pub fn send_serial(message: String, state: State<Arc<AppState>>) -> Result<(), String> {
    if let Some(tx) = state.communication.outbound_tx.lock().unwrap().as_ref() {
        tx.send(message)
            .map_err(|e| format!("Failed to send message: {}", e))
    } else {
        Err("Serial port is not connected.".into())
    }
}

#[tauri::command]
pub fn start_fake_data(
    app_handle: AppHandle,
    config: FakeDataConfig,
    state: State<Arc<AppState>>,
) -> Result<bool, String> {
    if state.stream.signal_stream_running.load(Ordering::SeqCst) {
        state.stream.signal_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.stream.signal_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    // Set the fake signal data enabled flag to true
    state.stream.fake_signal_enabled.store(true, Ordering::SeqCst);
    
    let reader = FakeBinaryReader::new(config);
    state.stream.signal_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.stream.signal_stream_running.clone();
    let state_inner = state.inner().clone();
    let app_clone = app_handle.clone();
    let handle = thread::spawn(move || {
        reader_loop(reader, running_flag, state_inner, app_clone);
    });
    *state.stream.signal_stream_handle.lock().unwrap() = Some(handle);

    Ok(true)
}

#[tauri::command]
pub fn stop_data_acquisition(app_handle: AppHandle) -> Result<(), String> {
    let app_state = app_handle.state::<Arc<AppState>>();
    
    // Stop all running threads
    if app_state.stream.signal_stream_running.load(Ordering::SeqCst) {
        app_state.stream.signal_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = app_state.stream.signal_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    *app_state.communication.outbound_tx.lock().unwrap() = None;
    if app_state.mdns.is_active() {
        let _ = mdns::stop_mdns_service(&app_handle);
    }
    Ok(())
}

#[tauri::command]
pub fn get_available_ports() -> Result<Vec<String>, String> {
    match serialport::available_ports() {
        Ok(ports) => Ok(ports.into_iter().map(|p| p.port_name).collect()),
        Err(e) => Err(format!("Failed to list serial ports: {}", e)),
    }
}

#[tauri::command]
pub fn start_streaming(
    app_handle: AppHandle,
    path: String,
    fake: bool
) -> Result<(), String> {
    // Delegate to the implementation in the streaming module
    crate::streaming::start_streaming(app_handle, path, fake)
}

#[tauri::command]
pub fn stop_streaming(app_handle: AppHandle) -> Result<(), String> {
    // Delegate to the implementation in the streaming module
    crate::streaming::stop_streaming(app_handle)
}

// Recording directory is now handled by the frontend using the fs plugin

#[tauri::command]
pub fn start_recording(
    format: String,
    directory: String,
    max_duration_minutes: u32,
    auto_start: bool,
    app_handle: AppHandle,
) -> Result<String, String> {
    // Delegate to the implementation in the recording module
    crate::recording::start_recording(format, directory, max_duration_minutes, auto_start, app_handle)
}

#[tauri::command]
pub fn stop_recording(app_handle: AppHandle) -> Result<(), String> {
    // Delegate to the implementation in the recording module
    crate::recording::stop_recording(app_handle)
}


#[tauri::command]
pub fn get_recording_filename(app_handle: tauri::AppHandle) -> String {
    // Get the current recording filename from application state
    let state = app_handle.state::<Arc<AppState>>();
    let filename = state.recording.recording_filename.lock().unwrap();
    
    match *filename {
        Some(ref name) => name.clone(),
        None => String::new()
    }
}

#[tauri::command]
pub async fn push_video_frame(
    app_handle: tauri::AppHandle,
    frame_data: Vec<u8>,
    width: u32,
    height: u32,
) -> Result<tauri_plugin_record_stream::FrameAnalysisResponse, String> {
    // Don't print the raw data - it would flood the console
    println!("[Main App] Pushing video frame...({} x {})", width, height);
    
    tauri_plugin_record_stream::push_frame(app_handle, frame_data, width, height)
        .map_err(|e| format!("Failed to push video frame: {}", e))
}

#[tauri::command]
pub async fn start_stream_recording(
    app_handle: tauri::AppHandle,
    file_path: String,
) -> Result<bool, String> {
    println!("[Main App] Starting video stream recording to: {}", file_path);
    
    // Parse the file path to get directory and filename
    let path = std::path::Path::new(&file_path);
    let directory = path.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());
    let filename = path.file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("recording_{}.mp4", chrono::Utc::now().timestamp()));
    
    // Call the recording module function to start recording
    let result = crate::recording::start_video_recording(
        app_handle,
        filename,
        directory
    )?;
    
    println!("[Main App] Video recording start result: {}", result);
    
    Ok(result)
}

#[tauri::command]
pub async fn stop_stream_recording(
    app_handle: tauri::AppHandle,
) -> Result<bool, String> {
    println!("[Main App] Stopping video stream recording");
    
    // Call the recording module function to stop recording
    let result = crate::recording::stop_video_recording(app_handle)?;
    
    println!("[Main App] Video recording stopped successfully");
    
    Ok(result)
}

#[tauri::command]
pub fn toggle_fake_data(state: State<Arc<AppState>>) -> Result<bool, String> {
    // Get current state using centralized function
    let current_state = match get_app_state(state.clone(), "stream".to_string(), "fake_camera_enabled".to_string()) {
        Ok(value) => value.as_bool().unwrap_or(false),
        Err(_) => false,
    };
    
    // Toggle the state
    let new_state = !current_state;
    
    // Update the state with the new value
    state.stream.fake_camera_enabled.store(new_state, Ordering::SeqCst);
    
    println!("[Backend] Toggled fake camera to: {}", new_state);
    
    Ok(new_state)
}

#[tauri::command]
pub fn toggle_fake_signal(state: State<Arc<AppState>>) -> Result<bool, String> {
    // Get current state using centralized function
    let current = match get_app_state(state.clone(), "stream".to_string(), "fake_signal_enabled".to_string()) {
        Ok(value) => value.as_bool().unwrap_or(false),
        Err(_) => false,
    };
    
    // Toggle the state
    let new_value = !current;
    state.stream.fake_signal_enabled.store(new_value, Ordering::SeqCst);
    
    println!("[Main App] Fake signal data enabled: {}", new_value);
    
    Ok(new_value)
}

#[tauri::command]
pub fn discover_streaming_devices(app_handle: AppHandle) -> Result<(), String> {
    let result = crate::mdns::discover_mdns_devices(app_handle, "_iot._tcp.local.".into());
    if result.is_err() {
        eprintln!("IoT discovery error: {}", result.as_ref().err().unwrap());
        // Return the first error if both failed
        if result.is_err() {
            return result.map_err(|e| e.to_string());
        }
    }
    
    // If either succeeded, return Ok
    Ok(())
}

#[tauri::command]
pub fn set_default_stream_url(app_handle: AppHandle, url: String) -> Result<(), String> {
    // Set the default stream URL in the app state
    let state = app_handle.state::<Arc<AppState>>();
    *state.stream.default_stream_url.lock().unwrap() = url.clone();
    
    println!("Default stream URL set to: {}", url);
    Ok(())
}


#[tauri::command]
pub fn get_app_state(
    state: State<Arc<AppState>>,
    category: String,
    key: String,
) -> Result<serde_json::Value, String> {
    match category.as_str() {
        "communication" => match key.as_str() {
            // Communication state doesn't have many directly accessible values
            _ => Err(format!("Invalid key '{}' for communication category", key)),
        },
        "buffer" => match key.as_str() {
            "data" => Ok(serde_json::to_value(&state.get_data()).unwrap_or(serde_json::Value::Null)),
            _ => Err(format!("Invalid key '{}' for buffer category", key)),
        },
        "signal_quality" => match key.as_str() {
            "quality" => {
                let quality = state.signal_quality.check_signal_quality();
                Ok(serde_json::to_value(quality).unwrap_or(serde_json::Value::Null))
            },
            _ => Err(format!("Invalid key '{}' for signal_quality category", key)),
        },
        "stream" => match key.as_str() {
            "signal_running" => Ok(serde_json::json!(state.stream.signal_stream_running.load(Ordering::SeqCst))),
            "camera_running" => Ok(serde_json::json!(state.stream.camera_stream_running.load(Ordering::SeqCst))),
            "fake_signal_enabled" => Ok(serde_json::json!(state.stream.fake_signal_enabled.load(Ordering::SeqCst))),
            "fake_camera_enabled" => Ok(serde_json::json!(state.stream.fake_camera_enabled.load(Ordering::SeqCst))),
            "default_stream_url" => Ok(serde_json::json!(state.stream.default_stream_url.lock().unwrap().clone())),
            "all" => {
                let signal_running = state.stream.signal_stream_running.load(Ordering::SeqCst);
                let camera_running = state.stream.camera_stream_running.load(Ordering::SeqCst);
                let fake_signal = state.stream.fake_signal_enabled.load(Ordering::SeqCst);
                let fake_camera = state.stream.fake_camera_enabled.load(Ordering::SeqCst);
                let url = state.stream.default_stream_url.lock().unwrap().clone();
                
                Ok(serde_json::json!({
                    "signalRunning": signal_running,
                    "cameraRunning": camera_running,
                    "fakeSignalEnabled": fake_signal,
                    "fakeCameraEnabled": fake_camera,
                    "defaultStreamUrl": url,
                }))
            },
            _ => Err(format!("Invalid key '{}' for stream category", key)),
        },
        "recording" => match key.as_str() {
            "active" => Ok(serde_json::json!(state.recording.recording_active.load(Ordering::SeqCst))),
            "video_active" => Ok(serde_json::json!(state.recording.video_recording_active.load(Ordering::SeqCst))),
            "filename" => {
                let filename = state.recording.recording_filename.lock().unwrap().clone().unwrap_or_default();
                Ok(serde_json::json!(filename))
            },
            "status" => {
                let recording_active = state.recording.recording_active.load(Ordering::SeqCst);
                let video_active = state.recording.video_recording_active.load(Ordering::SeqCst);
                let filename = state.recording.recording_filename.lock().unwrap().clone().unwrap_or_default();
                
                Ok(serde_json::json!({
                    "active": recording_active,
                    "videoActive": video_active,
                    "filename": filename,
                    "isRecording": recording_active || video_active,
                }))
            },
            _ => Err(format!("Invalid key '{}' for recording category", key)),
        },
        "mdns" => match key.as_str() {
            "active" => Ok(serde_json::json!(*state.mdns.active.lock().unwrap())),
            "host" => Ok(serde_json::json!(state.mdns.host.lock().unwrap().clone())),
            "port" => Ok(serde_json::json!(*state.mdns.port.lock().unwrap())),
            "discovered_devices" => {
                let devices = state.mdns.discovered_devices.lock().unwrap().clone();
                Ok(serde_json::to_value(devices).unwrap_or(serde_json::Value::Null))
            },
            _ => Err(format!("Invalid key '{}' for mdns category", key)),
        },
        // Handle combined views for frontend convenience
        "streaming_view" => match key.as_str() {
            "state" => {
                let camera_streaming = state.stream.camera_stream_running.load(Ordering::SeqCst);
                let is_fake_enabled = state.stream.fake_camera_enabled.load(Ordering::SeqCst);
                let recording_active = state.recording.recording_active.load(Ordering::SeqCst);
                let video_recording_active = state.recording.video_recording_active.load(Ordering::SeqCst);
                let is_recording = recording_active || video_recording_active;
                let filename = if recording_active {
                    state.recording.recording_filename.lock().unwrap().clone().unwrap_or_default()
                } else {
                    String::new()
                };
                
                Ok(serde_json::json!({
                    "isStreaming": camera_streaming,
                    "isFakeEnabled": is_fake_enabled,
                    "isRecording": is_recording,
                    "videoRecordingActive": video_recording_active,
                    "regularRecordingActive": recording_active,
                    "recordingFilename": filename
                }))
            },
            _ => Err(format!("Invalid key '{}' for streaming_view category", key)),
        },
        "signal_config" => match key.as_str() {
            "state" => {
                let is_running = state.stream.signal_stream_running.load(Ordering::SeqCst);
                let fake_signal_enabled = state.stream.fake_signal_enabled.load(Ordering::SeqCst);
                
                // Determine connection status
                let status = if state.communication.outbound_tx.lock().unwrap().is_some() {
                    "connected"
                } else {
                    "disconnected"
                };
                
                Ok(serde_json::json!({
                    "isRunning": is_running,
                    "isFakeSignalEnabled": fake_signal_enabled,
                    "connectionStatus": if is_running { "connected" } else { "disconnected" },
                    "dataSource": status
                }))
            },
            _ => Err(format!("Invalid key '{}' for signal_config category", key)),
        },
        _ => Err(format!("Invalid category: {}", category)),
    }
}
