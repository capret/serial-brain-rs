const COMMANDS: &[&str] = &["ping", "start_record", "push_frame", "stop_record", "configure_record"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
