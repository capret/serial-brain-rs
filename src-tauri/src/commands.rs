use std::{
    sync::{Arc, mpsc, atomic::Ordering},
    thread,
};
use tauri::{AppHandle, State};
use serialport;
use crate::state::SerialState;
use crate::types::{ChannelData, FakeDataConfig};
use crate::reader::{SerialBinaryReader, SocketBinaryReader, FakeBinaryReader, reader_loop};

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
    if state.fake_stream_running.load(Ordering::SeqCst) {
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    let (tx, rx) = mpsc::channel::<String>();
    *state.outbound_tx.lock().unwrap() = Some(tx);

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
pub fn connect_socket(
    app_handle: AppHandle,
    host: String,
    port: u16,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    if state.fake_stream_running.load(Ordering::SeqCst) {
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

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
pub fn send_serial(
    message: String,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    if let Some(tx) = state.outbound_tx.lock().unwrap().as_ref() {
        tx.send(message)
            .map_err(|e| format!("Failed to send message: {}", e))
    } else {
        Err("Serial port is not connected.".into())
    }
}

#[tauri::command]
pub fn get_recent_data(
    state: State<Arc<SerialState>>,
) -> Result<Vec<ChannelData>, String> {
    Ok(state.get_data())
}

#[tauri::command]
pub fn start_fake_data(
    app_handle: AppHandle,
    config: FakeDataConfig,
    state: State<Arc<SerialState>>,
) -> Result<bool, String> {
    if state.fake_stream_running.load(Ordering::SeqCst) {
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

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
pub fn stop_data_acquisition(
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    if state.fake_stream_running.load(Ordering::SeqCst) {
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
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
