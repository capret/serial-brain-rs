use std::sync::Arc;
use tauri;

mod types;
mod state;
mod reader;
mod commands;
use commands::{connect_serial, connect_socket, 
                send_serial, get_recent_data, 
                get_available_ports, start_fake_data, 
                stop_data_acquisition};
use state::SerialState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let serial_state = Arc::new(SerialState::new());
    tauri::Builder::default()
        .manage(serial_state)
        .invoke_handler(tauri::generate_handler![
            connect_serial,
            connect_socket,
            send_serial,
            get_recent_data,
            get_available_ports,
            start_fake_data,
            stop_data_acquisition
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
