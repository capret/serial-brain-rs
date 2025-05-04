use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<AndroidForwardService<R>> {
  Ok(AndroidForwardService(app.clone()))
}

/// Access to the android-forward-service APIs.
pub struct AndroidForwardService<R: Runtime>(AppHandle<R>);

impl<R: Runtime> AndroidForwardService<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }

  pub fn start_service(&self) -> crate::Result<()> {
    // Foreground service not needed on desktop; no-op
    println!("[Desktop] Foreground service would start here on Android");
    eprintln!("[Desktop] Foreground service start called");
    Ok(())
  }

  pub fn stop_service(&self) -> crate::Result<()> {
    println!("[Desktop] Foreground service would stop here on Android");
    eprintln!("[Desktop] Foreground service stop called");
    Ok(())
  }
}
