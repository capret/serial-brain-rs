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
    .invoke_handler(tauri::generate_handler![commands::ping])
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
