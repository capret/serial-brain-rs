use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::RecordStream;
#[cfg(mobile)]
use mobile::RecordStream;

/// Start recording from the stream
pub async fn start_record<R: Runtime>(app: tauri::AppHandle<R>, payload: StartRecordRequest) -> Result<StartRecordResponse> {
  app.record_stream().start_record(payload)
}

/// Response from frame analysis
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct FrameAnalysisResponse {
  pub success: bool,
  pub is_covered: bool,
  pub edge_count: i32,
}

/// Push a frame to the current recording and analyze if camera is covered 
pub fn push_frame<R: Runtime>(app: tauri::AppHandle<R>, rgb: Vec<u8>, width: u32, height: u32) -> Result<FrameAnalysisResponse> {
  app.record_stream().push_frame(rgb, width, height)
}

/// Stop the current recording
pub fn stop_record<R: Runtime>(app: tauri::AppHandle<R>) -> Result<bool> {
  app.record_stream().stop_record()
}

/// Configure recording parameters
pub fn configure_record<R: Runtime>(_app: tauri::AppHandle<R>, _width: u32, _height: u32, _fps: f64) -> Result<bool> {
  #[cfg(mobile)]
  return _app.record_stream().configure_record(_width, _height, _fps);
  
  #[cfg(desktop)]
  {
    // Desktop doesn't need separate configure step - will use these values when starting recording
    Ok(true)
  }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the record-stream APIs.
pub trait RecordStreamExt<R: Runtime> {
  fn record_stream(&self) -> &RecordStream<R>;
}

impl<R: Runtime, T: Manager<R>> crate::RecordStreamExt<R> for T {
  fn record_stream(&self) -> &RecordStream<R> {
    self.state::<RecordStream<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("record-stream")
    .invoke_handler(tauri::generate_handler![
      commands::ping, 
      commands::start_record,
      commands::push_frame,
      commands::stop_record,
      commands::configure_record
    ])
    .setup(|app, api| {
      #[cfg(mobile)]
      let record_stream = mobile::init(app, api)?;
      #[cfg(desktop)]
      let record_stream = desktop::init(app, api)?;
      app.manage(record_stream);
      Ok(())
    })
    .build()
}
