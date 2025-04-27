#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let platform = tauri_plugin_os::platform();
    println!("Platform: {}", platform);
    serial_brain_rs_lib::run();
}
