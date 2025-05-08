use crate::reader::{
    reader_loop, FakeBinaryReader, SerialBinaryReader, SocketBinaryReader,
};
use crate::state::SerialState;
use crate::types::{ChannelData, FakeDataConfig};
use serialport;
use std::{
    sync::{atomic::Ordering, mpsc, Arc},
    thread,
};
use tauri::{AppHandle, Emitter, State};


#[tauri::command]
pub fn connect_serial(
    app_handle: AppHandle,
    port: String,
    baud_rate: u32,
    stop_bits: u8,
    parity: String,
    data_bits: u8,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    if state.signal_stream_running.load(Ordering::SeqCst) {
        state.signal_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.signal_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    let (tx, rx) = mpsc::channel::<String>();
    *state.outbound_tx.lock().unwrap() = Some(tx);

    let reader = SerialBinaryReader::new(port.clone(), baud_rate, stop_bits, parity, data_bits, rx);
    state.signal_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.signal_stream_running.clone();
    let state_inner = state.inner().clone();
    let app_clone = app_handle.clone();
    let handle = thread::spawn(move || {
        reader_loop(reader, running_flag, state_inner, app_clone);
    });
    *state.signal_stream_handle.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
pub fn connect_socket(
    app_handle: AppHandle,
    host: String,
    port: u16,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    // Send a clear status message to reset UI state
    let _ = app_handle.emit("socket_status", format!("[SOCKET] Attempting to connect to {}:{}", host, port));
    
    if state.signal_stream_running.load(Ordering::SeqCst) {
        state.signal_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.signal_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    // Create the reader but don't set it up yet - setup will be done in reader_loop
    // Pass the app_handle to the reader so it can emit socket status events
    let reader = SocketBinaryReader::new(host.clone(), port)
        .with_app_handle(app_handle.clone());
    state.signal_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.signal_stream_running.clone();
    let state_inner = state.inner().clone();
    let app_clone = app_handle.clone();
    let handle = thread::spawn(move || {
        reader_loop(reader, running_flag, state_inner, app_clone);
    });
    *state.signal_stream_handle.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
pub fn send_serial(message: String, state: State<Arc<SerialState>>) -> Result<(), String> {
    if let Some(tx) = state.outbound_tx.lock().unwrap().as_ref() {
        tx.send(message)
            .map_err(|e| format!("Failed to send message: {}", e))
    } else {
        Err("Serial port is not connected.".into())
    }
}

#[tauri::command]
pub fn get_recent_data(state: State<Arc<SerialState>>) -> Result<Vec<ChannelData>, String> {
    Ok(state.get_data())
}

#[tauri::command]
pub fn start_fake_data(
    app_handle: AppHandle,
    config: FakeDataConfig,
    state: State<Arc<SerialState>>,
) -> Result<bool, String> {
    if state.signal_stream_running.load(Ordering::SeqCst) {
        state.signal_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.signal_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    // Set the fake data enabled flag to true
    state.fake_data_enabled.store(true, Ordering::SeqCst);

    let reader = FakeBinaryReader::new(config.clone());
    state.signal_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.signal_stream_running.clone();
    let state_inner = state.inner().clone();
    let app_clone = app_handle.clone();
    let handle = thread::spawn(move || {
        reader_loop(reader, running_flag, state_inner, app_clone);
    });
    *state.signal_stream_handle.lock().unwrap() = Some(handle);

    Ok(true)
}

#[tauri::command]
pub fn stop_data_acquisition(state: State<Arc<SerialState>>) -> Result<(), String> {
    if state.signal_stream_running.load(Ordering::SeqCst) {
        state.signal_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.signal_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    *state.outbound_tx.lock().unwrap() = None;
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
    fake: bool,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    // Delegate to the implementation in the streaming module
    crate::streaming::start_streaming(app_handle, path, fake, state)
}

#[tauri::command]
pub fn stop_streaming(state: State<Arc<SerialState>>) -> Result<(), String> {
    // Delegate to the implementation in the streaming module
    crate::streaming::stop_streaming(state)
}

// Recording directory is now handled by the frontend using the fs plugin

#[tauri::command]
pub fn start_recording(
    format: String,
    directory: String,
    max_duration_minutes: u32,
    auto_start: bool,
    state: State<Arc<SerialState>>,
) -> Result<String, String> {
    // Delegate to the implementation in the recording module
    crate::recording::start_recording(format, directory, max_duration_minutes, auto_start, state)
}

#[tauri::command]
pub fn stop_recording(state: State<Arc<SerialState>>) -> Result<(), String> {
    // Delegate to the implementation in the recording module
    crate::recording::stop_recording(state)
}

#[tauri::command]
pub fn get_recording_status(
    state: State<Arc<SerialState>>,
) -> Result<serde_json::Value, String> {
    // Return the current recording status including filename and recording state
    let recording_active = state.recording_active.load(Ordering::SeqCst);
    let filename = if recording_active {
        state.recording_filename.lock().unwrap().clone().unwrap_or_default()
    } else {
        String::new()
    };

    let json = serde_json::json!({
        "isRecording": recording_active,
        "filename": filename,
    });

    Ok(json)
}

#[tauri::command]
pub fn get_connection_status(
    state: State<Arc<SerialState>>,
) -> Result<serde_json::Value, String> {
    // Get outbound_tx to check if we have a serial connection
    let has_serial = state.outbound_tx.lock().unwrap().is_some();
    
    // Check for camera streaming
    let camera_streaming = state.camera_stream_running.load(Ordering::SeqCst);
    
    // Check if signal streaming is active
    let signal_streaming = state.signal_stream_running.load(Ordering::SeqCst);
    
    // Determine connection status based on state
    let status = if has_serial {
        "serial"
    } else if camera_streaming {
        "stream"
    } else if signal_streaming {
        "fake"
    } else {
        "disconnected"
    };
    
    // Check if data is being received (any kind of streaming is active)
    let is_running = camera_streaming || signal_streaming;
    
    let json = serde_json::json!({
        "status": status,
        "isRunning": is_running,
    });

    Ok(json)
}

#[tauri::command]
pub fn get_signal_quality(state: State<Arc<SerialState>>) -> Result<Vec<bool>, String> {
    // Use the on-demand check_signal_quality method instead of just getting current values
    Ok(state.check_signal_quality())
}

#[tauri::command]
pub async fn record_video_stream(
    app_handle: tauri::AppHandle,
    file_path: String,
) -> Result<bool, String> {
    println!("[Main App] Calling record stream plugin with path: {}", file_path);
    
    let result = tauri_plugin_record_stream::start_record(
        app_handle,
        tauri_plugin_record_stream::StartRecordRequest { file_path }
    )
    .await
    .map_err(|e| format!("Failed to record stream: {}", e))?;
    
    Ok(result.success)
}

#[tauri::command]
pub async fn stop_video_recording(
    app_handle: tauri::AppHandle,
) -> Result<bool, String> {
    println!("[Main App] Stopping video recording");
    
    tauri_plugin_record_stream::stop_record(app_handle)
        .map_err(|e| format!("Failed to stop video recording: {}", e))
}

#[tauri::command]
pub async fn push_video_frame(
    app_handle: tauri::AppHandle,
    frame_data: String,
) -> Result<bool, String> {
    // Don't print the base64 data - it would flood the console
    println!("[Main App] Pushing video frame...");
    
    tauri_plugin_record_stream::push_frame(app_handle, frame_data)
        .map_err(|e| format!("Failed to push video frame: {}", e))
}

#[tauri::command]
pub async fn start_stream_recording(
    app_handle: tauri::AppHandle,
    file_path: String,
    state: tauri::State<'_, Arc<SerialState>>,
) -> Result<bool, String> {
    println!("[Main App] Starting video stream recording to: {}", file_path);
    
    // Start the recording in the plugin
    let result = tauri_plugin_record_stream::start_record(
        app_handle.clone(),
        tauri_plugin_record_stream::StartRecordRequest { file_path }
    )
    .await
    .map_err(|e| format!("Failed to start video recording: {}", e))?;
    
    // If successful, set the recording flag in SerialState
    if result.success {
        state.video_recording_active.store(true, std::sync::atomic::Ordering::SeqCst);
        println!("[Main App] Video recording started successfully");
    }
    
    Ok(result.success)
}

#[tauri::command]
pub async fn stop_stream_recording(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Arc<SerialState>>,
) -> Result<bool, String> {
    println!("[Main App] Stopping video stream recording");
    
    // First set the recording flag to false to stop streaming frames
    state.video_recording_active.store(false, std::sync::atomic::Ordering::SeqCst);
    
    // Then stop the recording in the plugin
    let result = tauri_plugin_record_stream::stop_record(app_handle)
        .map_err(|e| format!("Failed to stop video recording: {}", e))?;
    
    println!("[Main App] Video recording stopped successfully");
    
    Ok(result)
}

#[tauri::command]
pub fn is_video_recording_active(
    state: State<Arc<SerialState>>,
) -> Result<bool, String> {
    // Return the current video recording active state
    let video_recording_active = state.video_recording_active.load(std::sync::atomic::Ordering::SeqCst);
    Ok(video_recording_active)
}

#[tauri::command]
pub fn toggle_fake_data(
    state: State<Arc<SerialState>>,
) -> Result<bool, String> {
    // Toggle the fake data enabled flag
    let current = state.fake_data_enabled.load(Ordering::SeqCst);
    let new_value = !current;
    state.fake_data_enabled.store(new_value, Ordering::SeqCst);
    
    println!("[Main App] Fake data enabled: {}", new_value);
    
    Ok(new_value)
}

#[tauri::command]
pub fn get_signal_config_state(
    state: State<Arc<SerialState>>,
) -> Result<serde_json::Value, String> {
    // Get signal connection status information
    let has_serial = state.outbound_tx.lock().unwrap().is_some();
    let signal_streaming = state.signal_stream_running.load(Ordering::SeqCst);
    
    // Determine data source based on state
    let status = if has_serial {
        "serial"
    } else if signal_streaming {
        "fake"
    } else {
        "disconnected"
    };
    
    // Check if signal data is being received
    let is_running = signal_streaming || has_serial;
    
    // Get fake data status directly from the state
    let is_fake_enabled = state.fake_data_enabled.load(Ordering::SeqCst);
    
    // Determine connection status string for the frontend
    let connection_status = if is_running {
        "connected"
    } else {
        "disconnected"
    };
    
    // Combine all state information into a single JSON response
    let json = serde_json::json!({
        "isRunning": is_running,
        "isFakeEnabled": is_fake_enabled,
        "connectionStatus": connection_status,
        "dataSource": status
    });

    Ok(json)
}

#[tauri::command]
pub fn get_streaming_view_state(
    state: State<Arc<SerialState>>,
) -> Result<serde_json::Value, String> {
    // Get camera streaming status specifically
    let camera_streaming = state.camera_stream_running.load(Ordering::SeqCst);
    
    // Get fake data status directly from the state
    let is_fake_enabled = state.fake_data_enabled.load(Ordering::SeqCst);
    
    // Get recording status information
    let recording_active = state.recording_active.load(Ordering::SeqCst);
    let video_recording_active = state.video_recording_active.load(Ordering::SeqCst);
    let is_recording = recording_active || video_recording_active;
    
    // Get recording filename if available
    let filename = if recording_active {
        state.recording_filename.lock().unwrap().clone().unwrap_or_default()
    } else {
        String::new()
    };
    
    // Combine all state information into a single JSON response
    let json = serde_json::json!({
        "isStreaming": camera_streaming,
        "isFakeEnabled": is_fake_enabled,
        "isRecording": is_recording,
        "videoRecordingActive": video_recording_active,
        "regularRecordingActive": recording_active,
        "recordingFilename": filename
    });

    Ok(json)
}
