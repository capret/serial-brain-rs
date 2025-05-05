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
    // stop any existing stream and wait for it to finish
    if state.camera_stream_running.load(Ordering::SeqCst) {
        state.camera_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.camera_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    if fake {
        state.camera_stream_running.store(true, Ordering::SeqCst);
        let running = state.camera_stream_running.clone();
        let app_clone = app_handle.clone();
        let handle = thread::spawn(move || {
            let mut rng = thread_rng();
            let mut frame_count = 0;
            while running.load(Ordering::SeqCst) {
                // Alternate between light and dark frames every 30 frames (about 1 second)
                let is_light_frame = (frame_count / 30) % 2 == 0;
                frame_count += 1;
                
                // Generate different brightness levels for testing
                let img: ImageBuffer<Rgb<u8>, Vec<u8>> = if is_light_frame {
                    // Light frame - values between 100 and 255
                    ImageBuffer::from_fn(W, H, |_x, _y| {
                        let r = rng.gen_range(100..=255);
                        let g = rng.gen_range(100..=255);
                        let b = rng.gen_range(100..=255);
                        Rgb([r, g, b])
                    })
                } else {
                    // Dark frame - values between 0 and 70
                    ImageBuffer::from_fn(W, H, |_x, _y| {
                        let r = rng.gen_range(0..=70);
                        let g = rng.gen_range(0..=70);
                        let b = rng.gen_range(0..=70);
                        Rgb([r, g, b])
                    })
                };
                let mut buf = Vec::new();
                let raw = img.clone().into_raw();
                PngEncoder::new(&mut buf)
                    .write_image(&raw, W, H, ColorType::Rgb8.into())
                    .unwrap();
                // Analyze the image before sending
                let total_pixels = (W * H) as u64;
                let sum: u64 = raw.chunks(3).map(|p| (p[0] as u64 + p[1] as u64 + p[2] as u64) / 3).sum();
                let avg = if total_pixels > 0 { sum / total_pixels } else { 0 };
                
                // Emit frame analysis result (true if avg >= 80, false otherwise)
                let is_bright_enough = avg >= 80;
                let _ = app_clone.emit("frame_analysis", Arc::new(is_bright_enough));
                
                // Encode and send the frame
                let b64 = STANDARD.encode(&buf);
                let _ = app_clone.emit("frame", Arc::new(b64));
                std::thread::sleep(Duration::from_millis(33));
            }
        });
        *state.camera_stream_handle.lock().unwrap() = Some(handle);
    } else {
        state.camera_stream_running.store(true, Ordering::SeqCst);
        let _running = state.camera_stream_running.clone();
        let app_clone = app_handle.clone();
        let url = path.clone();
        let handle = thread::spawn(move || {
            // build HTTP client with timeout
            let client = match Client::builder().timeout(Duration::from_secs(1)).build() {
                Ok(c) => c,
                Err(e) => {
                    let _ = app_clone.emit("stream_error", Arc::new(e.to_string()));
                    return;
                }
            };
            // send request and handle status errors
            let resp = match client.get(&url).send() {
                Ok(r) => match r.error_for_status() {
                    Ok(r2) => r2,
                    Err(e) => {
                        let _ = app_clone.emit("stream_error", Arc::new(e.to_string()));
                        return;
                    }
                },
                Err(e) => {
                    let _ = app_clone.emit("stream_error", Arc::new(e.to_string()));
                    return;
                }
            };
            let content_type = resp
                .headers()
                .get("Content-Type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let boundary = content_type
                .split(';')
                .find_map(|s| s.trim().strip_prefix("boundary="))
                .unwrap_or("frame");
            let boundary_marker = format!("--{}", boundary);
            let mut reader = BufReader::new(resp);
            let mut line = String::new();
            while _running.load(Ordering::SeqCst) {
                line.clear();
                if reader.read_line(&mut line).is_err() {
                    break;
                }
                if line.trim() == boundary_marker {
                    // parse headers
                    let mut content_length = 0;
                    loop {
                        let mut header_line = String::new();
                        if reader.read_line(&mut header_line).is_err() {
                            break;
                        }
                        let h = header_line.trim();
                        if h.is_empty() {
                            break;
                        }
                        if let Some(val) = h.split(':').nth(1) {
                            if header_line.to_lowercase().starts_with("content-length") {
                                content_length = val.trim().parse().unwrap_or(0);
                            }
                        }
                    }
                    if content_length > 0 {
                        let mut buf = vec![0u8; content_length];
                        if reader.read_exact(&mut buf).is_err() {
                            break;
                        }
                        let mut crlf = [0u8; 2];
                        let _ = reader.read_exact(&mut crlf);
                        // Analyze image before sending it
                        let image = match image::load_from_memory(&buf) {
                            Ok(img) => img,
                            Err(_) => {
                                // If image analysis fails, just send the frame without analysis
                                let b64 = STANDARD.encode(&buf);
                                app_clone.emit("frame", Arc::new(b64)).unwrap();
                                continue;
                            }
                        };

                        // Calculate average pixel value for brightness analysis
                        let rgb_image = image.to_rgb8();
                        let pixels = rgb_image.pixels();
                        let total_pixels = pixels.len() as u64;
                        
                        // Calculate average pixel value (across all rgb channels)
                        let sum: u64 = pixels.map(|p| (p[0] as u64 + p[1] as u64 + p[2] as u64) / 3).sum();
                        let avg = if total_pixels > 0 { sum / total_pixels } else { 0 };
                        
                        // Emit frame analysis result (true if avg >= 80, false otherwise)
                        let is_bright_enough = avg >= 80;
                        app_clone.emit("frame_analysis", Arc::new(is_bright_enough)).unwrap();
                        
                        // Encode and send the frame
                        let b64 = STANDARD.encode(&buf);
                        app_clone.emit("frame", Arc::new(b64)).unwrap();
                    }
                }
            }
        });
        *state.camera_stream_handle.lock().unwrap() = Some(handle);
    }
    Ok(())
}

#[tauri::command]
pub fn stop_streaming(state: State<Arc<SerialState>>) -> Result<(), String> {
    if state.camera_stream_running.load(Ordering::SeqCst) {
        state.camera_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.camera_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    Ok(())
}

// Recording directory is now handled by the frontend using the fs plugin

#[tauri::command]
pub fn start_recording(
    format: String,
    directory: String,
    max_duration_minutes: u32,
    _auto_start: bool,
    state: State<Arc<SerialState>>,
) -> Result<bool, String> {
    let mut path = PathBuf::from(&directory);
    
    // Create a timestamped filename
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    
    let timestamp = now.as_millis();
    let filename = match format.as_str() {
        "csv" => format!("serial_recording_{}.csv", timestamp),
        "json" => format!("serial_recording_{}.json", timestamp),
        "binary" => format!("serial_recording_{}.bin", timestamp),
        _ => return Err("Invalid format specified".to_string()),
    };
    
    path.push(filename);
    
    // Set up the file
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| format!("Failed to create recording file: {}", e))?;
    
    // Store the file in state for continued writing
    *state.recording_file.lock().unwrap() = Some((file, format.clone()));
    
    // Start the recording thread that will poll data and write to file
    if !state.recording_active.load(Ordering::SeqCst) {
        state.recording_active.store(true, Ordering::SeqCst);
        let state_clone = state.inner().clone();
        // let _format_clone = format.clone(); // Not needed currently
        let max_duration = Duration::from_secs(max_duration_minutes as u64 * 60);
        let start_time = SystemTime::now();
        
        // Write header for CSV format
        if format == "csv" {
            if let Some((ref mut file, _)) = *state.recording_file.lock().unwrap() {
                // Write CSV header based on channel count
                let mut header = String::from("timestamp");
                for i in 0..8 { // Support up to 8 channels as per memory
                    header.push_str(&format!(",channel_{}", i));
                }
                if let Err(e) = writeln!(file, "{}", header) {
                    return Err(format!("Failed to write CSV header: {}", e));
                }
            }
        } else if format == "json" {
            // Start JSON array
            if let Some((ref mut file, _)) = *state.recording_file.lock().unwrap() {
                if let Err(e) = file.write_all(b"[") {
                    return Err(format!("Failed to write JSON opening: {}", e));
                }
            }
        }
        
        let handle = thread::spawn(move || {
            let mut first_json_entry = true;
            
            while state_clone.recording_active.load(Ordering::SeqCst) {
                // Check if max duration is reached
                if let Ok(elapsed) = SystemTime::now().duration_since(start_time) {
                    if elapsed > max_duration {
                        state_clone.recording_active.store(false, Ordering::SeqCst);
                        break;
                    }
                }
                
                // Get data from the recording buffer with timestamps
                let timestamped_data = state_clone.get_recording_data();
                if timestamped_data.is_empty() {
                    // If no new data, sleep a bit and try again
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                
                // Record the data based on format
                let mut recording_file = state_clone.recording_file.lock().unwrap();
                if let Some((ref mut file, ref format)) = *recording_file {
                    match format.as_str() {
                        "csv" => {
                            // Process each data point with its timestamp
                            for (timestamp, channel_data) in &timestamped_data {
                                // Convert timestamp to milliseconds
                                let timestamp_ms = timestamp
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap_or_else(|_| Duration::from_secs(0))
                                    .as_millis();
                                    
                                // CSV: timestamp,val1,val2,...
                                let mut line = format!("{}", timestamp_ms);
                                // Each channel_data is an array of 8 f32 values
                                for &value in channel_data.iter() {
                                    line.push_str(&format!(",{}", value));
                                }
                                if let Err(e) = writeln!(file, "{}", line) {
                                    eprintln!("Error writing to CSV file: {}", e);
                                    break;
                                }
                            }
                            // Flush CSV entries to disk in real time
                            if let Err(e) = file.flush() {
                                eprintln!("Error flushing CSV file: {}", e);
                            }
                        },
                        "json" => {
                            // Process each data point with its timestamp
                            for (timestamp, channel_data) in &timestamped_data {
                                // Convert timestamp to milliseconds
                                let timestamp_ms = timestamp
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap_or_else(|_| Duration::from_secs(0))
                                    .as_millis();
                                                    
                                // Convert data to flattened Vec for JSON serialization
                                let mut values = Vec::new();
                                for &value in channel_data.iter() {
                                    values.push(value);
                                }
                                                    
                                let json_entry = format!("{}{{\"timestamp\": {},\"values\": {}}}",
                                    if first_json_entry { "" } else { "," },
                                    timestamp_ms,
                                    serde_json::to_string(&values).unwrap()
                                );
                                first_json_entry = false;
                                                    
                                if let Err(e) = file.write_all(json_entry.as_bytes()) {
                                    eprintln!("Error writing to JSON file: {}", e);
                                    break;
                                }
                            }
                        },
                        "binary" => {
                            // Process each data point with its timestamp
                            for (timestamp, channel_data) in &timestamped_data {
                                // Convert timestamp to milliseconds
                                let timestamp_ms = timestamp
                                    .duration_since(SystemTime::UNIX_EPOCH)
                                    .unwrap_or_else(|_| Duration::from_secs(0))
                                    .as_millis() as u64;
                                    
                                // Calculate number of values
                                let num_values = channel_data.len() as u32;
                                
                                // Write timestamp
                                if let Err(e) = file.write_all(&timestamp_ms.to_le_bytes()) {
                                    eprintln!("Error writing timestamp to binary file: {}", e);
                                    break;
                                }
                                
                                // Write number of values
                                if let Err(e) = file.write_all(&num_values.to_le_bytes()) {
                                    eprintln!("Error writing value count to binary file: {}", e);
                                    break;
                                }
                                
                                // Write each value
                                for &value in channel_data.iter() {
                                    let value_f64 = value as f64;
                                    if let Err(e) = file.write_all(&value_f64.to_le_bytes()) {
                                        eprintln!("Error writing value to binary file: {}", e);
                                        break;
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
                
                // Sleep a shorter time to check for new data more frequently
                thread::sleep(Duration::from_millis(5));
            }
            
            // Finalize the recording
            let mut recording_file = state_clone.recording_file.lock().unwrap();
            if let Some((ref mut file, ref format)) = *recording_file {
                if format == "json" {
                    // Close the JSON array
                    let _ = file.write_all(b"]");
                }
                
                // Flush the file
                let _ = file.flush();
            }
            
            // Clear the file handle
            *recording_file = None;
        });
        
        *state.recording_handle.lock().unwrap() = Some(handle);
    }
    
    Ok(true)
}

#[tauri::command]
pub fn stop_recording(state: State<Arc<SerialState>>) -> Result<(), String> {
    if state.recording_active.load(Ordering::SeqCst) {
        state.recording_active.store(false, Ordering::SeqCst);
        
        // Wait for the recording thread to finish
        if let Some(handle) = state.recording_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    
    Ok(())
}

#[tauri::command]
pub fn get_recording_status(state: State<Arc<SerialState>>) -> Result<bool, String> {
    Ok(state.recording_active.load(Ordering::SeqCst))
}

#[tauri::command]
pub fn get_signal_quality(state: State<Arc<SerialState>>) -> Result<Vec<bool>, String> {
    // Use the on-demand check_signal_quality method instead of just getting current values
    Ok(state.check_signal_quality())
}
