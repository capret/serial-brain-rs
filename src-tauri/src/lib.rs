use std::sync::Arc;

mod commands;
mod reader;
mod state;
mod types;
mod file_utils;
mod streaming;
mod recording;
mod mdns;
use commands::{
    connect_serial, connect_socket, discover_streaming_devices, get_available_ports, get_app_state,
    set_default_stream_url, get_recording_filename, push_video_frame, 
    send_serial, start_fake_data, start_recording, start_stream_recording, start_streaming, 
    stop_data_acquisition, stop_recording, stop_stream_recording, stop_streaming, 
    toggle_fake_data, toggle_fake_signal,
};
use file_utils::get_file_stats;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(AppState::new());
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_blec::init())
        .plugin(tauri_plugin_android_forward_service::init())
        .plugin(tauri_plugin_record_stream::init())
        .manage(app_state.clone())
        .setup(move | app| {
            // Store the app handle in the serial state for event emission
            let app_handle = app.handle();
            // Store in communication state for event emission
            *app_state.communication.app_handle.lock().unwrap() = Some(app_handle.clone());
            
            // Start the background mDNS scanner thread
            println!("[Setup] Starting background mDNS scanner...");
            if let Err(e) = mdns::start_background_scanner(app_handle.clone()) {
                println!("[Setup] Error starting background mDNS scanner: {}", e);
            }
            
            Ok(())
        })
        // Removed automatic frame stream on startup; streaming controlled via commands
        .invoke_handler(tauri::generate_handler![
            connect_serial,
            connect_socket,
            discover_streaming_devices,
            get_available_ports,
            get_app_state,
            set_default_stream_url,
            get_recording_filename,
            get_file_stats,
            push_video_frame,
            send_serial,
            start_fake_data,
            start_stream_recording,
            stop_stream_recording,
            stop_data_acquisition,
            start_streaming,
            stop_streaming,
            start_recording,
            stop_recording,
            toggle_fake_data,
            toggle_fake_signal
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
