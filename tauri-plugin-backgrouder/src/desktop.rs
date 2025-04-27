use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<Backgrouder<R>> {
  Ok(Backgrouder(app.clone()))
}

/// Access to the backgrouder APIs.
pub struct Backgrouder<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Backgrouder<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
