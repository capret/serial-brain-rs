use crate::state::AppState;
use crate::commands::{start_video_recording as cmd_start_video_recording, stop_video_recording as cmd_stop_video_recording};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{ImageBuffer, Rgb, GenericImageView, ColorType};
use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use rand::{Rng, thread_rng};
use reqwest::blocking::Client;
use std::io::{BufRead, BufReader, Read};
use std::sync::{atomic::Ordering, Arc};
use std::time::Duration;
use std::thread;
use tauri::{AppHandle, Emitter, Manager};

// Constants for fake stream image generation
const W: u32 = 320;
const H: u32 = 240;

/// Starts a video streaming process, either fake (generated) or from a URL.
/// 
/// # Arguments
/// * `app_handle` - Tauri app handle for emitting events back to the frontend
/// * `path` - URL path for the stream source (only used if fake=false)
/// * `fake` - Whether to generate a fake stream or use the provided URL
pub fn start_streaming(
    app_handle: AppHandle,
    path: String,
    fake: bool
) -> Result<(), String> {
    // stop any existing stream and wait for it to finish
    let state = app_handle.state::<Arc<AppState>>();
    if state.stream.camera_stream_running.load(Ordering::SeqCst) {
        state.stream.camera_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.stream.camera_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    
    // Update the fake_camera_enabled flag to match the current request
    state.stream.fake_camera_enabled.store(fake, Ordering::SeqCst);
    println!("[Streaming] Setting fake camera enabled to: {}", fake);
    
    if fake {
        start_fake_stream(app_handle)
    } else {
        start_real_stream(app_handle, path)
    }
}

/// Stops an active streaming process.
pub fn stop_streaming(app_handle: AppHandle) -> Result<(), String> {
    let app_state = app_handle.state::<Arc<AppState>>();
    let stream_state = &app_state.stream;
    
    if stream_state.camera_stream_running.load(Ordering::SeqCst) {
        stream_state.camera_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = stream_state.camera_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    Ok(())
}

/// Starts a fake stream with generated image data.
fn start_fake_stream(
    app_handle: AppHandle,
) -> Result<(), String> {
    // Get state instance
    let app_state = app_handle.state::<Arc<AppState>>();
    
    // Set up stream state
    app_state.stream.camera_stream_running.store(true, Ordering::SeqCst);
    let running = app_state.stream.camera_stream_running.clone();
    let app_clone = app_handle.clone();
    let state_clone = Arc::clone(&app_state);
    
    let handle = thread::spawn(move || {
        let mut rng = thread_rng();
        let mut frame_count = 0;
        while running.load(Ordering::SeqCst) {
            let is_light_frame = (frame_count / 30) % 2 == 0;
            frame_count += 1;
            
            let img: ImageBuffer<Rgb<u8>, Vec<u8>> = if is_light_frame {
                ImageBuffer::from_fn(W, H, |_x, _y| {
                    let r = rng.gen_range(100..=255);
                    let g = rng.gen_range(100..=255);
                    let b = rng.gen_range(100..=255);
                    Rgb([r, g, b])
                })
            } else {
                ImageBuffer::from_fn(W, H, |_x, _y| {
                    let r = rng.gen_range(0..=70);
                    let g = rng.gen_range(0..=70);
                    let b = rng.gen_range(0..=70);
                    Rgb([r, g, b])
                })
            };
            
            let mut buf = Vec::new();
            let raw = img.clone().into_raw();
            let encoder = PngEncoder::new(&mut buf);
            encoder.write_image(&raw, W, H, ColorType::Rgb8.into())
                .unwrap();
            let b64 = STANDARD.encode(&buf);
            let _ = app_clone.emit("frame", Arc::new(b64.clone()));
            
            // Check recording state
            if state_clone.recording.video_recording_active.load(Ordering::SeqCst) {
                match tauri_plugin_record_stream::push_frame(app_clone.clone(), raw.clone(), W, H) {
                    Ok(analysis) => {
                        let _ = app_clone.emit("frame_analysis", Arc::new(!analysis.is_covered));
                    },
                    Err(e) => println!("Error pushing frame to video recorder: {}", e)
                }
            }
            std::thread::sleep(Duration::from_millis(33));
        }
    });
    
    // Store the handle in the state - create a new clone for this operation
    let app_state = app_handle.state::<Arc<AppState>>();
    *app_state.stream.camera_stream_handle.lock().unwrap() = Some(handle);
    Ok(())
}

/// Starts a real stream from a camera or network source.
/// This implementation provides robust reconnection and frame buffering.
/// Only manual stop can end the streaming; network errors trigger reconnection.
fn start_real_stream(
    app_handle: AppHandle,
    url: String,
) -> Result<(), String> {
    // Get state instance
    let app_state = app_handle.state::<Arc<AppState>>();
    
    // Set up stream state
    app_state.stream.camera_stream_running.store(true, Ordering::SeqCst);
    let running = app_state.stream.camera_stream_running.clone();
    let app_clone = app_handle.clone();
    let state_clone = Arc::clone(&app_state);
    
    let handle = thread::spawn(move || {
        let mut last_frame_buffer: Option<Vec<u8>> = None;
        let reconnect_delay = Duration::from_millis(100); // Wait before reconnection attempts
        
        // Main streaming loop - only exits on manual stop
        while running.load(Ordering::SeqCst) {
            // Attempt to establish connection
            let client = match Client::builder()
                .timeout(Duration::from_secs(5)) // Increased timeout for better stability
                .build() 
            {
                Ok(c) => c,
                Err(e) => {
                    let _ = app_clone.emit("stream_error", Arc::new(format!("Client build error: {}, retrying...", e)));
                    thread::sleep(reconnect_delay);
                    continue; // Retry connection
                }
            };
            
            // Attempt to connect to stream
            let resp = match client.get(&url).send() {
                Ok(r) => match r.error_for_status() {
                    Ok(r2) => r2,
                    Err(e) => {
                        let _ = app_clone.emit("stream_error", Arc::new(format!("HTTP error: {}, reconnecting...", e)));
                        thread::sleep(reconnect_delay);
                        continue; // Retry connection
                    }
                },
                Err(e) => {
                    let _ = app_clone.emit("stream_error", Arc::new(format!("Connection error: {}, reconnecting...", e)));
                    thread::sleep(reconnect_delay);
                    continue; // Retry connection
                }
            };
            
            let _ = app_clone.emit("stream_connected", Arc::new("Stream connected successfully"));
            
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
            
            // Frame reading loop - breaks on connection errors to trigger reconnection
            let mut connection_active = true;
            while running.load(Ordering::SeqCst) && connection_active {
                line.clear();
                
                // Handle read line errors by breaking to reconnect
                if reader.read_line(&mut line).is_err() {
                    let _ = app_clone.emit("stream_error", Arc::new("Connection lost, reconnecting..."));
                    connection_active = false;
                    break;
                }
                
                if line.trim() == boundary_marker {
                    // Parse headers
                    let mut content_length = 0;
                    let mut header_parse_error = false;
                    
                    loop {
                        let mut header_line = String::new();
                        if reader.read_line(&mut header_line).is_err() {
                            header_parse_error = true;
                            break;
                        }
                        
                        let h = header_line.trim();
                        if h.is_empty() {
                            break;
                        }
                        
                        let mut parts = h.splitn(2, ':');
                        if let Some(key) = parts.next() {
                            if let Some(val) = parts.next() {
                                if key.to_lowercase() == "content-length" {
                                    content_length = val.trim().parse::<usize>().unwrap_or(0);
                                }
                            }
                        }
                    }
                    
                    if header_parse_error {
                        let _ = app_clone.emit("stream_error", Arc::new("Header parse error, reconnecting..."));
                        connection_active = false;
                        break;
                    }
                    
                    // Read image data
                    if content_length > 0 {
                        let mut buffer = vec![0u8; content_length];
                        match reader.read_exact(&mut buffer) {
                            Ok(_) => {
                                // Successfully read new frame - update last frame buffer
                                last_frame_buffer = Some(buffer.clone());
                                
                                // Encode the image to base64 and emit to frontend
                                let b64 = STANDARD.encode(&buffer);
                                let _ = app_clone.emit("frame", Arc::new(b64.clone()));
                                
                                // Also push frame to video recorder if recording is active
                                if state_clone.recording.video_recording_active.load(Ordering::SeqCst) {
                                    // Convert to raw RGB bytes for recording instead of using base64 PNG
                                    if let Ok(img) = image::load_from_memory(&buffer) {
                                        let rgb = img.to_rgb8();
                                        let dims = img.dimensions();
                                        match tauri_plugin_record_stream::push_frame(app_clone.clone(), rgb.into_raw(), dims.0, dims.1) {
                                            Ok(analysis) => {
                                                let _ = app_clone.emit("frame_analysis", Arc::new(!analysis.is_covered));
                                            },
                                            Err(e) => println!("Error pushing frame to video recorder: {}", e)
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                // Failed to read new frame - send last frame if available
                                if let Some(ref last_buffer) = last_frame_buffer {
                                    let b64 = STANDARD.encode(last_buffer);
                                    let _ = app_clone.emit("frame", Arc::new(b64.clone()));
                                    let _ = app_clone.emit("stream_error", Arc::new("Frame read error, sending last frame and reconnecting..."));
                                } else {
                                    let _ = app_clone.emit("stream_error", Arc::new("Frame read error, no last frame available, reconnecting..."));
                                }
                                connection_active = false;
                                break;
                            }
                        }
                    }
                } else if line.trim().is_empty() {
                    // Empty line might indicate connection issues
                    continue;
                }
            }
            
            // If we're here due to connection error (not manual stop), wait before reconnecting
            if running.load(Ordering::SeqCst) && !connection_active {
                thread::sleep(reconnect_delay);
            }
        }
        
        let _ = app_clone.emit("stream_stopped", Arc::new("Stream stopped by user"));
    });
    
    let app_state = app_handle.state::<Arc<AppState>>();
    *app_state.stream.camera_stream_handle.lock().unwrap() = Some(handle);
    Ok(())
}

/// Starts a video recording with the specified filename and directory.
/// This function is a wrapper around the command-based implementation to allow
/// direct calls from other modules like recording.rs during segment rotation.
pub fn start_video_recording(
    app_handle: AppHandle,
    filename: String,
    directory: String,
) -> Result<bool, String> {
    // Need to use a runtime for the async function call
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;
    
    // Run the async function to start video recording
    runtime.block_on(async {
        cmd_start_video_recording(app_handle, filename, directory).await
    })
}

/// Stops an active video recording.
/// This function is a wrapper around the command-based implementation to allow
/// direct calls from other modules like recording.rs during segment rotation.
pub fn stop_video_recording(
    app_handle: AppHandle,
) -> Result<bool, String> {
    // Need to use a runtime for the async function call
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;
    
    // Run the async function to stop video recording
    runtime.block_on(async {
        cmd_stop_video_recording(app_handle).await
    })
}
