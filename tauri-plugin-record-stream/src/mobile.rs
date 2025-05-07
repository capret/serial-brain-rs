use serde::de::DeserializeOwned;
use tauri::plugin::{PluginApi, PluginHandle};
use tauri::{AppHandle, Runtime};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_record_stream);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<RecordStream<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin("com.plugin.record-stream", "RecordStreamPlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_record_stream)?;
  Ok(RecordStream(handle))
}

/// Access to the record-stream APIs.
pub struct RecordStream<R: Runtime>(PluginHandle<R>);

#[derive(serde::Serialize)]
struct PushFrameRequest {
  b64_png: String,
}

#[derive(serde::Serialize)]
struct ConfigureRequest {
  width: u32,
  height: u32,
  fps: f64,
}

#[derive(serde::Deserialize)]
struct BooleanResponse {
  success: bool,
}

impl<R: Runtime> RecordStream<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self
      .0
      .run_mobile_plugin("ping", payload)
      .map_err(Into::into)
  }

  pub fn start_record(&self, payload: StartRecordRequest) -> crate::Result<StartRecordResponse> {
    self
      .0
      .run_mobile_plugin("startRecord", payload)
      .map_err(Into::into)
  }
  
  pub fn configure_record(&self, width: u32, height: u32, fps: f64) -> crate::Result<bool> {
    let payload = ConfigureRequest { width, height, fps };
    let response: BooleanResponse = self
      .0
      .run_mobile_plugin("configureRecord", payload)
      .map_err(Into::into)?;
    Ok(response.success)
  }
  
  pub fn push_frame(&self, b64_png: String) -> crate::Result<bool> {
    let payload = PushFrameRequest { b64_png };
    let response: BooleanResponse = self
      .0
      .run_mobile_plugin("pushFrame", payload)
      .map_err(Into::into)?;
    Ok(response.success)
  }
  
  pub fn stop_record(&self) -> crate::Result<bool> {
    // For stop, we can just pass an empty object
    let response: BooleanResponse = self
      .0
      .run_mobile_plugin::<_, _>("stopRecord", ())
      .map_err(Into::into)?;
    Ok(response.success)
  }
}
