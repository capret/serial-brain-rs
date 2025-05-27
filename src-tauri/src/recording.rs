use crate::state::AppState;
use serde_json::json;
use std::fs::{OpenOptions, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{atomic::Ordering, Arc};
use std::thread::{self, JoinHandle};
use std::time::{SystemTime, Duration};
use tauri::{AppHandle, Emitter, Manager};

/// Starts recording data to a file in the specified format.
/// 
/// # Arguments
/// * `format` - The format to use for recording ("csv", "json", or "binary")
/// * `directory` - The directory path where recordings should be saved
/// * `max_duration_minutes` - Maximum duration in minutes for each recording segment
/// * `_auto_start` - Whether to automatically start recording (currently unused)
/// * `state` - Application state containing shared data
pub fn start_recording(
    format: String,
    directory: String,
    max_duration_minutes: u32,
    _auto_start: bool,
    app_handle: AppHandle,
) -> Result<String, String> {
    let state = app_handle.state::<Arc<AppState>>();
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
    
    // Clone the filename before pushing to path to avoid ownership issues
    path.push(filename.clone());
    
    // Set up the file
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| format!("Failed to create recording file: {}", e))?;
    
    // Store the file in state for continued writing
    *state.recording.recording_file.lock().unwrap() = Some((file, format.clone()));
    
    // Store the filename for retrieval even when switching views
    *state.recording.recording_filename.lock().unwrap() = Some(filename.clone());
    
    // Start the recording thread that will poll data and write to file
    if !state.recording.recording_active.load(Ordering::SeqCst) {
        state.recording.recording_active.store(true, Ordering::SeqCst);
        let state_clone = state.inner().clone();
        
        // Clone format and directory for the emit event after max duration
        let format_clone = format.clone();
        let directory_clone = directory.clone();
        
        let max_duration = Duration::from_secs(max_duration_minutes as u64 * 60);
        let start_time = SystemTime::now();
        
        // Write header for CSV format
        if format == "csv" {
            if let Some((ref mut file, _)) = *state.recording.recording_file.lock().unwrap() {
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
            if let Some((ref mut file, _)) = *state.recording.recording_file.lock().unwrap() {
                if let Err(e) = file.write_all(b"[") {
                    return Err(format!("Failed to write JSON opening: {}", e));
                }
            }
        }
        
        let handle = spawn_recording_thread(state_clone, format_clone, directory_clone, max_duration, start_time);
        
        *state.recording.recording_handle.lock().unwrap() = Some(handle);
    }
    
    // Return the actual filename that was created
    Ok(filename)
}

/// Stops an active recording process.
pub fn stop_recording(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<Arc<AppState>>();
    
    // Only do something if recording is active
    if state.recording.recording_active.load(Ordering::SeqCst) {
        // Close the JSON array for JSON format recordings
        if let Some((ref mut file, ref format)) = *state.recording.recording_file.lock().unwrap() {
            if format == "json" {
                if let Err(e) = file.write_all(b"]") {
                    eprintln!("Error closing JSON file: {}", e);
                }
            }
        }
        
        // Set the flag to false before cleaning up
        state.recording.recording_active.store(false, Ordering::SeqCst);
        
        // Clean up the file handles and threads
        let _ = state.recording.recording_handle.lock().unwrap().take();
        *state.recording.recording_file.lock().unwrap() = None;
        *state.recording.recording_filename.lock().unwrap() = None;
    }
    
    Ok(())
}

/// Spawns a thread to handle recording data to files.
fn spawn_recording_thread(
    state_clone: Arc<AppState>,
    format_clone: String,
    directory_clone: String,
    max_duration: Duration,
    start_time: SystemTime,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut first_json_entry = true;
        let mut segment_start_time = start_time;
        
        while state_clone.recording.recording_active.load(Ordering::SeqCst) {
            // Check if the current segment exceeded the configured duration
            if let Ok(elapsed) = SystemTime::now().duration_since(segment_start_time) {
                if elapsed > max_duration {
                    handle_segment_rotation(
                        &state_clone,
                        &format_clone,
                        &directory_clone,
                        &mut first_json_entry,
                        &mut segment_start_time,
                        &max_duration,
                    );
                    continue; // Skip to next loop to immediately write to the new segment
                }
            }

            // Get batch of recording data
            let data_batch = state_clone.recording.get_recording_data();
            if data_batch.is_empty() {
                // If no new data, sleep a bit and try again
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            
            // Record the data based on format
            let mut recording_file = state_clone.recording.recording_file.lock().unwrap();
            if let Some((ref mut file, ref format)) = *recording_file {
                match format.as_str() {
                    "csv" => {
                        write_csv_data(file, &data_batch);
                    },
                    "json" => {
                        write_json_data(file, &data_batch, &mut first_json_entry);
                    },
                    "binary" => {
                        write_binary_data(file, &data_batch);
                    },
                    _ => {}
                }
            }
            
            // Sleep a shorter time to check for new data more frequently
            thread::sleep(Duration::from_millis(5));
        }
        
        // Finalize the recording
        let mut recording_file = state_clone.recording.recording_file.lock().unwrap();
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
    })
}

/// Rotates to a new recording segment when the max duration is reached.
fn handle_segment_rotation(
    state_clone: &Arc<AppState>,
    format_clone: &str,
    directory_clone: &str,
    first_json_entry: &mut bool,
    segment_start_time: &mut SystemTime,
    _max_duration: &Duration,
) {
    // Finalize the current segment file (flush + close JSON array if needed)
    {
        let mut recording_file = state_clone.recording.recording_file.lock().unwrap();
        if let Some((ref mut file, ref fmt)) = *recording_file {
            if fmt == "json" {
                let _ = file.write_all(b"]");
            }
            let _ = file.flush();
        }
    }

    // Create a new filename based on the same format & directory
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0));
    let timestamp = now.as_millis();
    let new_filename = match format_clone {
        "csv" => format!("serial_recording_{}.csv", timestamp),
        "json" => format!("serial_recording_{}.json", timestamp),
        _ => format!("serial_recording_{}.bin", timestamp),
    };

    let mut new_path = PathBuf::from(directory_clone);
    new_path.push(&new_filename);

    if let Ok(mut new_file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&new_path)
    {
        // CSV: write header; JSON: open array
        if format_clone == "csv" {
            let mut header = String::from("timestamp");
            for i in 0..8 {
                header.push_str(&format!(",channel_{}", i));
            }
            let _ = writeln!(new_file, "{}", header);
        } else if format_clone == "json" {
            let _ = new_file.write_all(b"[");
        }

        // Swap file handle and update filename in shared state
        *state_clone.recording.recording_file.lock().unwrap() =
            Some((new_file, format_clone.to_string()));
        *state_clone.recording.recording_filename.lock().unwrap() = Some(new_filename.clone());
        
        // Log the segment change
        println!("Recording segment changed to: {}", new_filename);
    }

    // Emit events to frontend just to notify filename change
    if let Some(app_handle) = state_clone.communication.app_handle.lock().unwrap().as_ref() {
        // Emit the existing event with full payload
        let _ = app_handle.emit("recording-file-changed", {
            json!({
                "filename": new_filename.clone(),
                "directory": directory_clone,
                "format": format_clone
            })
        });
        
        // Emit a more specific event for updating just the filename
        let _ = app_handle.emit("recording-filename-changed", new_filename.clone());
        
        println!("Emitted filename change events for segment rotation");
    }
    
    // Handle video recording segment rotation directly in the backend
    if let Some(app_handle) = state_clone.communication.app_handle.lock().unwrap().as_ref() {
        // Extract base filename for video recording (remove extension)
        let base_filename = new_filename.replace(&format!(".{}", format_clone), "");
        
        // First stop the current video recording
        println!("Stopping video recording for segment rotation");
        match crate::streaming::stop_video_recording(app_handle.clone()) {
            Ok(_) => println!("Successfully stopped video recording for segment rotation"),
            Err(e) => println!("Warning: Failed to stop video recording for segment rotation: {}", e),
        }
        
        // Small delay to ensure the video file is properly finalized
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // Start a new video recording segment
        println!("Starting new video recording segment with base filename: {}", base_filename);
        match crate::streaming::start_video_recording(
            app_handle.clone(),
            base_filename,
            directory_clone.to_string(),
        ) {
            Ok(_) => println!("Successfully started new video recording segment"),
            Err(e) => println!("Warning: Failed to start new video recording segment: {}", e),
        }
    }

    // Reset per-segment flags and timer
    *first_json_entry = true;
    *segment_start_time = SystemTime::now();
}

/// Writes data in CSV format to the specified file.
fn write_csv_data(file: &mut File, timestamped_data: &[(SystemTime, [f32; 8])]) {
    // Process each data point with its timestamp
    for (timestamp, channel_data) in timestamped_data {
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
}

/// Writes data in JSON format to the specified file.
fn write_json_data(file: &mut File, timestamped_data: &[(SystemTime, [f32; 8])], first_json_entry: &mut bool) {
    // Process each data point with its timestamp
    for (timestamp, channel_data) in timestamped_data {
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
            if *first_json_entry { "" } else { "," },
            timestamp_ms,
            serde_json::to_string(&values).unwrap()
        );
        *first_json_entry = false;
                        
        if let Err(e) = file.write_all(json_entry.as_bytes()) {
            eprintln!("Error writing to JSON file: {}", e);
            break;
        }
    }
}

/// Writes data in binary format to the specified file.
fn write_binary_data(file: &mut File, timestamped_data: &[(SystemTime, [f32; 8])]) {
    // Process each data point with its timestamp
    for (timestamp, channel_data) in timestamped_data {
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
}
