#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

use serde::Deserialize;
use tauri::{self, AppHandle, State}; // Import the Emitter trait to bring the `emit` method into scope.

// Define a type alias for the eight channel data.
type ChannelData = [f32; 8];

mod reader;
use reader::{SerialBinaryReader, SocketBinaryReader, FakeBinaryReader, reader_loop};

#[derive(Debug, Deserialize, Clone)]
struct FakeDataConfig {
    min_value: i32,
    max_value: i32,
    frequency: f64,
    channel_count: usize,
    waveform: String,
}

/// Shared state for our serial connection.
struct SerialState {
    outbound_tx: Mutex<Option<Sender<String>>>,
    active_buffer: Mutex<VecDeque<ChannelData>>,
    read_buffer: Mutex<VecDeque<ChannelData>>,
    buffer_lock: Mutex<()>,
    fake_stream_running: Arc<AtomicBool>,
    fake_stream_handle: Mutex<Option<JoinHandle<()>>>,
}

impl SerialState {
    fn new() -> Self {
        Self {
            outbound_tx: Mutex::new(None),
            active_buffer: Mutex::new(VecDeque::with_capacity(2000)),
            read_buffer: Mutex::new(VecDeque::with_capacity(2000)),
            buffer_lock: Mutex::new(()),
            fake_stream_running: Arc::new(AtomicBool::new(false)),
            fake_stream_handle: Mutex::new(None),
        }
    }

    fn add_data(&self, data: ChannelData) {
        // Acquire the lock to ensure consistency during writes
        let _guard = self.buffer_lock.lock().unwrap();
        
        // Add data to both buffers to ensure no data loss
        // Active buffer
        let mut active_buf = self.active_buffer.lock().unwrap();
        if active_buf.len() >= 2000 {
            active_buf.pop_front();
        }
        active_buf.push_back(data);
        
        // Read buffer
        let mut read_buf = self.read_buffer.lock().unwrap();
        if read_buf.len() >= 2000 {
            read_buf.pop_front();
        }
        read_buf.push_back(data);
    }

    fn get_data(&self) -> Vec<ChannelData> {
        // Acquire the lock to prevent data modifications during reading
        let _guard = self.buffer_lock.lock().unwrap();
        
        // Get read buffer and copy its contents
        let mut read_buf = self.read_buffer.lock().unwrap();
        let result: Vec<ChannelData> = read_buf.iter().cloned().collect();
        
        // Clear the read buffer after readings
        read_buf.clear();
        
        result
    }
}

#[tauri::command]
fn connect_serial(
    app_handle: AppHandle,
    port: String,
    baud_rate: u32,
    stop_bits: u8,
    parity: String,
    data_bits: u8,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    // Map stop bits

    // Stop existing reader if running
    if state.fake_stream_running.load(Ordering::SeqCst) {
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    // Setup outbound channel
    let (tx, rx) = mpsc::channel::<String>();
    {
        let mut outbound = state.outbound_tx.lock().unwrap();
        *outbound = Some(tx);
    }

    // Spawn serial reader loop
    let reader = SerialBinaryReader::new(port.clone(), baud_rate, stop_bits, parity, data_bits, rx);
    state.fake_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.fake_stream_running.clone();
    let state_inner = state.inner().clone();
    let app_clone = app_handle.clone();
    let handle = thread::spawn(move || {
        reader_loop(reader, running_flag, state_inner, app_clone);
    });
    *state.fake_stream_handle.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
fn connect_socket(
    app_handle: AppHandle,
    host: String,
    port: u16,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    // Stop existing reader if running
    if state.fake_stream_running.load(Ordering::SeqCst) {
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    // Spawn socket reader loop
    let reader = SocketBinaryReader::new(host.clone(), port);
    state.fake_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.fake_stream_running.clone();
    let state_inner = state.inner().clone();
    let app_clone = app_handle.clone();
    let handle = thread::spawn(move || {
        reader_loop(reader, running_flag, state_inner, app_clone);
    });
    *state.fake_stream_handle.lock().unwrap() = Some(handle);

    Ok(())
}

#[tauri::command]
fn send_serial(message: String, state: State<Arc<SerialState>>) -> Result<(), String> {
    if let Some(tx) = state.outbound_tx.lock().unwrap().as_ref() {
        tx.send(message)
            .map_err(|e| format!("Failed to send message: {}", e))
    } else {
        Err("Serial port is not connected.".into())
    }
}

#[tauri::command]
fn get_recent_data(state: State<Arc<SerialState>>) -> Result<Vec<ChannelData>, String> {
    Ok(state.get_data())
}

#[tauri::command]
fn start_fake_data(
    app_handle: AppHandle,
    config: FakeDataConfig,
    state: State<Arc<SerialState>>,
) -> Result<bool, String> {
    // Stop existing reader if running
    if state.fake_stream_running.load(Ordering::SeqCst) {
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    // Spawn fake data reader loop
    let reader = FakeBinaryReader::new(config.clone());
    state.fake_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.fake_stream_running.clone();
    let state_inner = state.inner().clone();
    let app_clone = app_handle.clone();
    let handle = thread::spawn(move || {
        reader_loop(reader, running_flag, state_inner, app_clone);
    });
    *state.fake_stream_handle.lock().unwrap() = Some(handle);

    Ok(true)
}

#[tauri::command]
fn stop_data_acquisition(state: State<Arc<SerialState>>) -> Result<(), String> {
    // Stop fake data stream if running
    if state.fake_stream_running.load(Ordering::SeqCst) {
        println!("Stopping fake data stream...");
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            handle
                .join()
                .map_err(|_| "Failed to join fake stream thread".to_string())?;
        }
        println!("Fake data stream stopped.");
    }
    
    // Close serial connection if exists (by setting the sender to None)
    let mut outbound = state.outbound_tx.lock().unwrap();
    *outbound = None;
    
    // Note: Proper cleanup of other acquisition types (e.g., TCP) would be added here
    
    Ok(())
}

#[tauri::command]
fn get_available_ports() -> Result<Vec<String>, String> {
    match serialport::available_ports() {
        Ok(ports) => {
            let port_names = ports.into_iter().map(|p| p.port_name).collect();
            Ok(port_names)
        }
        Err(e) => Err(format!("Failed to list serial ports: {}", e)),
    }
}

fn main() {
    let serial_state = Arc::new(SerialState::new());

    tauri::Builder::default()
        .manage(serial_state)
        .invoke_handler(tauri::generate_handler![
            connect_serial,
            connect_socket,
            send_serial,
            get_recent_data,
            get_available_ports,
            start_fake_data,
            stop_data_acquisition
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
