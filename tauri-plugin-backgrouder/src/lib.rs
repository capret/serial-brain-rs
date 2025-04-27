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
use desktop::Backgrouder;
#[cfg(mobile)]
use mobile::Backgrouder;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the backgrouder APIs.
pub trait BackgrouderExt<R: Runtime> {
  fn backgrouder(&self) -> &Backgrouder<R>;
}

impl<R: Runtime, T: Manager<R>> crate::BackgrouderExt<R> for T {
  fn backgrouder(&self) -> &Backgrouder<R> {
    self.state::<Backgrouder<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("backgrouder")
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      #[cfg(mobile)]
      let backgrouder = mobile::init(app, api)?;
      #[cfg(desktop)]
      let backgrouder = desktop::init(app, api)?;
      app.manage(backgrouder);
      Ok(())
    })
    .build()
}
