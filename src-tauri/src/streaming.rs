use crate::state::SerialState;
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
use tauri::{AppHandle, State, Emitter};

// Constants for fake stream image generation
const W: u32 = 640;
const H: u32 = 480;

/// Starts a video streaming process, either fake (generated) or from a URL.
/// 
/// # Arguments
/// * `app_handle` - Tauri app handle for emitting events back to the frontend
/// * `path` - URL path for the stream source (only used if fake=false)
/// * `fake` - Whether to generate a fake stream or use the provided URL
/// * `state` - Application state containing shared data
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
        start_fake_stream(app_handle, state)
    } else {
        start_real_stream(app_handle, path, state)
    }
}

/// Stops an active streaming process.
pub fn stop_streaming(state: State<Arc<SerialState>>) -> Result<(), String> {
    if state.camera_stream_running.load(Ordering::SeqCst) {
        state.camera_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.camera_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    Ok(())
}

/// Starts a fake stream with generated image data.
fn start_fake_stream(
    app_handle: AppHandle,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    state.camera_stream_running.store(true, Ordering::SeqCst);
    let running = state.camera_stream_running.clone();
    let app_clone = app_handle.clone();
    
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
                
            let total_pixels = (W * H) as u64;
            let sum: u64 = raw.chunks(3).map(|p| (p[0] as u64 + p[1] as u64 + p[2] as u64) / 3).sum();
            let avg = if total_pixels > 0 { sum / total_pixels } else { 0 };

            let is_bright_enough = avg >= 80;
            let _ = app_clone.emit("frame_analysis", Arc::new(is_bright_enough));

            let b64 = STANDARD.encode(&buf);
            let _ = app_clone.emit("frame", Arc::new(b64));
            std::thread::sleep(Duration::from_millis(33));
        }
    });
    
    *state.camera_stream_handle.lock().unwrap() = Some(handle);
    Ok(())
}

/// Starts a real stream from a camera or network source.
fn start_real_stream(
    app_handle: AppHandle,
    url: String,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    state.camera_stream_running.store(true, Ordering::SeqCst);
    let running = state.camera_stream_running.clone();
    let app_clone = app_handle.clone();
    
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
        
        while running.load(Ordering::SeqCst) {
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
                    
                    let mut parts = h.splitn(2, ':');
                    if let Some(key) = parts.next() {
                        if let Some(val) = parts.next() {
                            if key.to_lowercase() == "content-length" {
                                content_length = val.trim().parse::<usize>().unwrap_or(0);
                            }
                        }
                    }
                }
                
                // read image data
                if content_length > 0 {
                    let mut buffer = vec![0u8; content_length];
                    match reader.read_exact(&mut buffer) {
                        Ok(_) => {
                            // Try to analyze the image for brightness
                            let result = image::load_from_memory(&buffer);
                            if let Ok(img) = result {
                                // Calculate average brightness
                                let dims = img.dimensions();
                                let total_pixels = dims.0 * dims.1;
                                if total_pixels > 0 {
                                    let mut sum: u64 = 0;
                                    for pixel in img.to_rgb8().pixels() {
                                        sum += (pixel[0] as u64 + pixel[1] as u64 + pixel[2] as u64) / 3;
                                    }
                                    let avg = sum / total_pixels as u64;
                                    let is_bright_enough = avg >= 80;
                                    let _ = app_clone.emit("frame_analysis", Arc::new(is_bright_enough));
                                }
                            }
                            
                            // Encode the image to base64 and emit to frontend
                            let b64 = STANDARD.encode(&buffer);
                            let _ = app_clone.emit("frame", Arc::new(b64));
                        }
                        Err(e) => {
                            let _ = app_clone.emit("stream_error", Arc::new(format!("Error reading frame: {}", e)));
                            break;
                        }
                    }
                }
            }
        }
    });
    
    *state.camera_stream_handle.lock().unwrap() = Some(handle);
    Ok(())
}
