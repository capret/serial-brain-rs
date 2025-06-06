use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_android_forward_service);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<AndroidForwardService<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("com.plugin.android_forward_service", "SerialForegroundServicePlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_android_forward_service)?;
  Ok(AndroidForwardService(handle))
}

/// Access to the android-forward-service APIs.
pub struct AndroidForwardService<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> AndroidForwardService<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self
      .0
      .run_mobile_plugin("ping", payload)
      .map_err(Into::into)
  }

  pub fn start_service(&self) -> crate::Result<()> {
    self
      .0
      .run_mobile_plugin(
        "startRecordingService",
        StartServiceRequest {},
      )
      .map(|_: ()| ())
      .map_err(Into::into)
  }

  pub fn stop_service(&self) -> crate::Result<()> {
    self
      .0
      .run_mobile_plugin(
        "stopRecordingService",
        StopServiceRequest {},
      )
      .map(|_: ()| ())
      .map_err(Into::into)
  }
}
