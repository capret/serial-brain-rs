use crate::commands::{
    start_video_recording as cmd_start_video_recording,
    stop_video_recording as cmd_stop_video_recording,
};
use crate::state::AppState;
use base64::{engine::general_purpose::STANDARD, Engine};
use image::codecs::png::PngEncoder;
use image::ImageEncoder;
use image::{ColorType, GenericImageView, ImageBuffer, Rgb};
use rand::{thread_rng, Rng};
use reqwest::blocking::Client;
use std::io::{BufRead, BufReader, Read};
use std::sync::{atomic::Ordering, Arc};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

// Constants for fake stream image generation
const W: u32 = 320;
const H: u32 = 240;
const FRAME_INTERVAL_MS: u64 = 33; // ~30 FPS for consistent timing

/// Starts a video streaming process, either fake (generated) or from a URL.
///
/// # Arguments
/// * `app_handle` - Tauri app handle for emitting events back to the frontend
/// * `path` - URL path for the stream source (only used if fake=false)
/// * `fake` - Whether to generate a fake stream or use the provided URL
pub fn start_streaming(app_handle: AppHandle, path: String, fake: bool) -> Result<(), String> {
    // stop any existing stream and wait for it to finish
    let state = app_handle.state::<Arc<AppState>>();
    if state.stream.camera_stream_running.load(Ordering::SeqCst) {
        state
            .stream
            .camera_stream_running
            .store(false, Ordering::SeqCst);
        if let Some(handle) = state.stream.camera_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }

    // Update the fake_camera_enabled flag to match the current request
    state
        .stream
        .fake_camera_enabled
        .store(fake, Ordering::SeqCst);
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
        stream_state
            .camera_stream_running
            .store(false, Ordering::SeqCst);
        if let Some(handle) = stream_state.camera_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    Ok(())
}

/// Starts a fake stream with generated image data.
fn start_fake_stream(app_handle: AppHandle) -> Result<(), String> {
    // Get state instance
    let app_state = app_handle.state::<Arc<AppState>>();

    // Set up stream state
    app_state
        .stream
        .camera_stream_running
        .store(true, Ordering::SeqCst);
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
            encoder
                .write_image(&raw, W, H, ColorType::Rgb8.into())
                .unwrap();
            let b64 = STANDARD.encode(&buf);
            let _ = app_clone.emit("frame", Arc::new(b64.clone()));

            // Check recording state
            if state_clone
                .recording
                .video_recording_active
                .load(Ordering::SeqCst)
            {
                match tauri_plugin_record_stream::push_frame(app_clone.clone(), raw.clone(), W, H) {
                    Ok(analysis) => {
                        let _ = app_clone.emit("frame_analysis", Arc::new(!analysis.is_covered));
                    }
                    Err(e) => println!("Error pushing frame to video recorder: {}", e),
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
fn start_real_stream(app_handle: AppHandle, url: String) -> Result<(), String> {
    // Get state instance
    let app_state = app_handle.state::<Arc<AppState>>();

    // Set up stream state
    app_state
        .stream
        .camera_stream_running
        .store(true, Ordering::SeqCst);
    let running = app_state.stream.camera_stream_running.clone();
    let app_clone = app_handle.clone();
    let state_clone = Arc::clone(&app_state);

    // Create shared frame buffer using Arc<Mutex<>>
    let frame_buffer: Arc<std::sync::Mutex<Option<Vec<u8>>>> =
        Arc::new(std::sync::Mutex::new(None));
    let frame_rgb_buffer: Arc<std::sync::Mutex<Option<(Vec<u8>, u32, u32)>>> =
        Arc::new(std::sync::Mutex::new(None));

    // Clone for the consumer thread
    let frame_buffer_consumer = Arc::clone(&frame_buffer);
    let frame_rgb_buffer_consumer = Arc::clone(&frame_rgb_buffer);
    let running_consumer = running.clone();
    let app_clone_consumer = app_clone.clone();
    let state_clone_consumer = Arc::clone(&state_clone);

    // Spawn consumer thread that continuously pushes frames at 33ms intervals
    let consumer_handle = thread::spawn(move || {
        let mut frame_count = 0;
        while running_consumer.load(Ordering::SeqCst) {
            frame_count += 1;
            let mut has_frame = false;

            // Emit frame to frontend if available
            if let Ok(buffer_guard) = frame_buffer_consumer.lock() {
                if let Some(ref frame_data) = *buffer_guard {
                    has_frame = true;
                    let b64 = STANDARD.encode(frame_data);
                    let _ = app_clone_consumer.emit("frame", Arc::new(b64));
                }
            }

            // Push frame to video recorder if recording is active
            if state_clone_consumer
                .recording
                .video_recording_active
                .load(Ordering::SeqCst)
            {
                if let Ok(rgb_guard) = frame_rgb_buffer_consumer.lock() {
                    if let Some((ref rgb_data, width, height)) = *rgb_guard {
                        if frame_count % 30 == 0 {
                            println!(
                                "[CONSUMER] Pushing frame {} ({}x{}, has_frame: {})",
                                frame_count, width, height, has_frame
                            );
                        }
                        match tauri_plugin_record_stream::push_frame(
                            app_clone_consumer.clone(),
                            rgb_data.clone(),
                            width,
                            height,
                        ) {
                            Ok(analysis) => {
                                let _ = app_clone_consumer
                                    .emit("frame_analysis", Arc::new(!analysis.is_covered));
                            }
                            Err(e) => {
                                println!("Error pushing frame to video recorder: {}", e);
                            }
                        }
                    }
                }
            }

            // Sleep for 33ms to maintain ~30fps
            thread::sleep(Duration::from_millis(FRAME_INTERVAL_MS));
        }
        println!("[CONSUMER] Thread exiting");
    });

    // Clone for the producer thread
    let frame_buffer_producer = Arc::clone(&frame_buffer);
    let frame_rgb_buffer_producer = Arc::clone(&frame_rgb_buffer);
    let running_producer = running.clone();

    // Spawn producer thread that reads frames from the network stream
    let producer_handle = thread::spawn(move || {
        let reconnect_delay_ms: u64 = 100; // Wait before reconnection attempts
        let mut producer_frame_count = 0;

        // Main streaming loop - only exits on manual stop
        while running_producer.load(Ordering::SeqCst) {
            // Attempt to establish connection
            let client = match Client::builder().timeout(Duration::from_secs(5)).build() {
                Ok(c) => c,
                Err(e) => {
                    let _ = app_clone.emit(
                        "stream_error",
                        Arc::new(format!("Client build error: {}, retrying...", e)),
                    );
                    thread::sleep(Duration::from_millis(reconnect_delay_ms));
                    continue;
                }
            };

            // Attempt to connect to stream
            let resp = match client.get(&url).send() {
                Ok(r) => match r.error_for_status() {
                    Ok(r2) => r2,
                    Err(e) => {
                        let _ = app_clone.emit(
                            "stream_error",
                            Arc::new(format!("HTTP error: {}, reconnecting...", e)),
                        );
                        thread::sleep(Duration::from_millis(reconnect_delay_ms));
                        continue;
                    }
                },
                Err(e) => {
                    let _ = app_clone.emit(
                        "stream_error",
                        Arc::new(format!("Connection error: {}, reconnecting...", e)),
                    );
                    thread::sleep(Duration::from_millis(reconnect_delay_ms));
                    continue;
                }
            };

            println!("[PRODUCER] Stream connected successfully");
            let _ = app_clone.emit(
                "stream_connected",
                Arc::new("Stream connected successfully"),
            );

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
            while running_producer.load(Ordering::SeqCst) && connection_active {
                line.clear();

                // Handle read line errors by breaking to reconnect
                if reader.read_line(&mut line).is_err() {
                    let _ = app_clone
                        .emit("stream_error", Arc::new("Connection lost, reconnecting..."));
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
                        let _ = app_clone.emit(
                            "stream_error",
                            Arc::new("Header parse error, reconnecting..."),
                        );
                        connection_active = false;
                        break;
                    }

                    // Read image data
                    if content_length > 0 {
                        let mut buffer = vec![0u8; content_length];
                        match reader.read_exact(&mut buffer) {
                            Ok(_) => {
                                producer_frame_count += 1;

                                // Successfully read new frame - update the shared frame buffer
                                if let Ok(mut frame_guard) = frame_buffer_producer.lock() {
                                    *frame_guard = Some(buffer.clone());
                                }

                                // Also update RGB buffer for recording
                                if let Ok(img) = image::load_from_memory(&buffer) {
                                    let rgb = img.to_rgb8();
                                    let dims = img.dimensions();
                                    let raw_data = rgb.into_raw();

                                    if let Ok(mut rgb_guard) = frame_rgb_buffer_producer.lock() {
                                        *rgb_guard = Some((raw_data, dims.0, dims.1));
                                    }

                                    if producer_frame_count % 30 == 0 {
                                        println!(
                                            "[PRODUCER] Read and updated frame {} ({}x{})",
                                            producer_frame_count, dims.0, dims.1
                                        );
                                    }
                                }
                            }
                            Err(_) => {
                                println!("[PRODUCER] Frame read error, reconnecting...");
                                let _ = app_clone.emit(
                                    "stream_error",
                                    Arc::new(
                                        "Frame read error, using last frame and reconnecting...",
                                    ),
                                );
                                connection_active = false;
                                break;
                            }
                        }
                    }
                } else if line.trim().is_empty() {
                    continue;
                }
            }

            // If connection error, wait before reconnecting
            if running_producer.load(Ordering::SeqCst) && !connection_active {
                println!(
                    "[PRODUCER] Connection lost, waiting {}ms before reconnecting...",
                    reconnect_delay_ms
                );
                thread::sleep(Duration::from_millis(reconnect_delay_ms));
            }
        }

        println!("[PRODUCER] Thread exiting");
        let _ = app_clone.emit("stream_stopped", Arc::new("Stream stopped by user"));
    });

    // Store both handles in the state
    let app_state = app_handle.state::<Arc<AppState>>();
    *app_state.stream.camera_stream_handle.lock().unwrap() = Some(producer_handle);

    // Note: We need to store the consumer handle somewhere too, but for now we'll let it detach
    // The consumer will exit when running flag is set to false
    std::mem::forget(consumer_handle); // Detach the consumer thread - it will exit when running becomes false

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
    runtime.block_on(async { cmd_start_video_recording(app_handle, filename, directory).await })
}

/// Stops an active video recording.
/// This function is a wrapper around the command-based implementation to allow
/// direct calls from other modules like recording.rs during segment rotation.
pub fn stop_video_recording(app_handle: AppHandle) -> Result<bool, String> {
    // Need to use a runtime for the async function call
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;

    // Run the async function to stop video recording
    runtime.block_on(async { cmd_stop_video_recording(app_handle).await })
}
