const COMMANDS: &[&str] = &[
    "ping",
    "start_forward_service",
    "stop_forward_service",
];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
