use std::{
    sync::{Arc, mpsc, atomic::Ordering},
    thread,
};
use tauri::{AppHandle, State, Emitter};
use serialport;
use image::{ImageBuffer, Rgb, codecs::png::PngEncoder, ColorType, ImageEncoder};
use rand::{thread_rng, Rng};
use crate::state::SerialState;
use crate::types::{ChannelData, FakeDataConfig};
use crate::reader::{SerialBinaryReader, SocketBinaryReader, FakeBinaryReader, reader_loop};
use reqwest::blocking::Client;
use std::io::{BufRead, BufReader, Read};
use std::time::Duration;
use base64::{engine::general_purpose::STANDARD, Engine};

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

#[tauri::command]
pub fn start_streaming(
    app_handle: AppHandle,
    path: String,
    fake: bool,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    // stop any existing stream and wait for it to finish
    if state.stream_running.load(Ordering::SeqCst) {
        state.stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    if fake {
        state.stream_running.store(true, Ordering::SeqCst);
        let running = state.stream_running.clone();
        let app_clone = app_handle.clone();
        let handle = thread::spawn(move || {
            let mut rng = thread_rng();
            while running.load(Ordering::SeqCst) {
                let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(W, H, |_x, _y| Rgb([rng.gen(), rng.gen(), rng.gen()]));
                let mut buf = Vec::new();
                let raw = img.clone().into_raw();
                PngEncoder::new(&mut buf).write_image(&raw, W, H, ColorType::Rgb8.into()).unwrap();
                let b64 = STANDARD.encode(&buf);
                let _ = app_clone.emit("frame", Arc::new(b64));
                std::thread::sleep(Duration::from_millis(33));
            }
        });
        *state.stream_handle.lock().unwrap() = Some(handle);
    } else {
        state.stream_running.store(true, Ordering::SeqCst);
        let _running = state.stream_running.clone();
        let app_clone = app_handle.clone();
        let url = path.clone();
        let handle = thread::spawn(move || {
            // build HTTP client with timeout
            let client = match Client::builder().timeout(Duration::from_secs(1)).build() {
                Ok(c) => c,
                Err(e) => { let _ = app_clone.emit("stream_error", Arc::new(e.to_string())); return; }
            };
            // send request and handle status errors
            let resp = match client.get(&url).send() {
                Ok(r) => match r.error_for_status() {
                    Ok(r2) => r2,
                    Err(e) => { let _ = app_clone.emit("stream_error", Arc::new(e.to_string())); return; }
                },
                Err(e) => { let _ = app_clone.emit("stream_error", Arc::new(e.to_string())); return; }
            };
            let content_type = resp.headers().get("Content-Type").and_then(|v| v.to_str().ok()).unwrap_or("");
            let boundary = content_type.split(';')
                .find_map(|s| s.trim().strip_prefix("boundary="))
                .unwrap_or("frame");
            let boundary_marker = format!("--{}", boundary);
            let mut reader = BufReader::new(resp);
            let mut line = String::new();
            while _running.load(Ordering::SeqCst) {
                line.clear();
                if reader.read_line(&mut line).is_err() { break; }
                if line.trim() == boundary_marker {
                    // parse headers
                    let mut content_length = 0;
                    loop {
                        let mut header_line = String::new();
                        if reader.read_line(&mut header_line).is_err() { break; }
                        let h = header_line.trim();
                        if h.is_empty() { break; }
                        if let Some(val) = h.split(':').nth(1) {
                            if header_line.to_lowercase().starts_with("content-length") {
                                content_length = val.trim().parse().unwrap_or(0);
                            }
                        }
                    }
                    if content_length > 0 {
                        let mut buf = vec![0u8; content_length];
                        if reader.read_exact(&mut buf).is_err() { break; }
                        let mut crlf = [0u8; 2];
                        let _ = reader.read_exact(&mut crlf);
                        let b64 = STANDARD.encode(&buf);
                        app_clone.emit("frame", Arc::new(b64)).unwrap();
                    }
                }
            }
        });
        *state.stream_handle.lock().unwrap() = Some(handle);
    }
    Ok(())
}

#[tauri::command]
pub fn stop_streaming(
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    if state.stream_running.load(Ordering::SeqCst) {
        state.stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    Ok(())
}
