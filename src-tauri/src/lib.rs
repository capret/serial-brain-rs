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
    connect_serial, connect_socket, discover_streaming_devices, get_available_ports, get_discovered_devices,
    get_recent_data, get_recording_status, get_signal_config_state, get_signal_quality, get_streaming_view_state, 
    is_video_recording_active, push_video_frame, record_video_stream, send_serial, start_fake_data, 
    start_recording, start_stream_recording, start_streaming, stop_data_acquisition, stop_recording, 
    stop_stream_recording, stop_streaming, stop_video_recording, toggle_fake_data,
};
use file_utils::get_file_stats;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let serial_state = Arc::new(AppState::new());
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_blec::init())
        .plugin(tauri_plugin_android_forward_service::init())
        .plugin(tauri_plugin_record_stream::init())
        .manage(serial_state.clone())
        .setup(move | app| {
            // Store the app handle in the serial state for event emission
            let app_handle = app.handle();
            // Store in communication state for event emission
            *serial_state.communication.app_handle.lock().unwrap() = Some(app_handle.clone());
            Ok(())
        })
        // Removed automatic frame stream on startup; streaming controlled via commands
        .invoke_handler(tauri::generate_handler![
            connect_serial,
            connect_socket,
            discover_streaming_devices,
            get_available_ports,
            get_discovered_devices,
            get_recent_data,
            get_recording_status,
            get_signal_quality,
            get_file_stats,
            record_video_stream,
            stop_video_recording,
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
            is_video_recording_active,
            get_streaming_view_state,
            toggle_fake_data,
            get_signal_config_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
