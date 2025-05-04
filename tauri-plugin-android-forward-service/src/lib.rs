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
use desktop::AndroidForwardService;
#[cfg(mobile)]
use mobile::AndroidForwardService;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the android-forward-service APIs.
pub trait AndroidForwardServiceExt<R: Runtime> {
  fn android_forward_service(&self) -> &AndroidForwardService<R>;
}

impl<R: Runtime, T: Manager<R>> crate::AndroidForwardServiceExt<R> for T {
  fn android_forward_service(&self) -> &AndroidForwardService<R> {
    self.state::<AndroidForwardService<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("android-forward-service")
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      #[cfg(mobile)]
      let android_forward_service = mobile::init(app, api)?;
      #[cfg(desktop)]
      let android_forward_service = desktop::init(app, api)?;
      app.manage(android_forward_service);
      Ok(())
    })
    .build()
}
