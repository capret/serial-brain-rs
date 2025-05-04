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
}
