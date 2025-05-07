use crate::reader::{
    reader_loop, FakeBinaryReader, SerialBinaryReader, SocketBinaryReader,
};
use crate::state::SerialState;
use crate::types::{ChannelData, FakeDataConfig};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageBuffer, Rgb, ColorType};

use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use rand::{thread_rng, Rng};
use reqwest::blocking::Client;
use serialport;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use std::{
    sync::{atomic::Ordering, mpsc, Arc},
    thread,
};
use tauri::{AppHandle, Emitter, State};

const W: u32 = 640;
const H: u32 = 480;

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
pub fn get_recording_status(state: State<Arc<SerialState>>) -> Result<bool, String> {
    // Delegate to the implementation in the recording module
    crate::recording::get_recording_status(state)
}

#[tauri::command]
pub fn get_recording_filename(state: State<Arc<SerialState>>) -> Result<String, String> {
    // Delegate to the implementation in the recording module
    crate::recording::get_recording_filename(state)
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
