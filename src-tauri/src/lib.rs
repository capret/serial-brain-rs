use std::sync::Arc;

mod commands;
mod reader;
mod state;
mod types;
use commands::{
    connect_serial, connect_socket, get_available_ports, get_recent_data, get_recording_status,
    get_signal_quality, send_serial, start_fake_data, start_recording, start_streaming,
    stop_data_acquisition, stop_recording, stop_streaming,
};
use state::SerialState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let serial_state = Arc::new(SerialState::new());
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_android_forward_service::init())
        .manage(serial_state)
        // Removed automatic frame stream on startup; streaming controlled via commands
        .invoke_handler(tauri::generate_handler![
            connect_serial,
            connect_socket,
            send_serial,
            get_recent_data,
            get_available_ports,
            start_fake_data,
            stop_data_acquisition,
            start_streaming,
            stop_streaming,
            start_recording,
            stop_recording,
            get_recording_status,
            get_signal_quality
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
