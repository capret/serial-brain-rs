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
  let handle = api.register_android_plugin("com.plugin.record_stream", "RecordStreamPlugin")?;
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_record_stream)?;
  Ok(RecordStream(handle))
}

/// Access to the record-stream APIs.
pub struct RecordStream<R: Runtime>(PluginHandle<R>);

// Response type for boolean results
#[derive(serde::Deserialize)]
struct BooleanResponse {
  success: bool,
}



impl<R: Runtime> RecordStream<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    self
      .0
      .run_mobile_plugin("ping", payload)
      .map_err(|e| crate::Error::from(e))
  }

  pub fn start_record(&self, payload: StartRecordRequest) -> crate::Result<StartRecordResponse> {
    use serde_json::json;
    
    // Create a simple JSON object instead of a typed struct
    let params = json!({
      "file_path": payload.file_path
    });
    
    self
      .0
      .run_mobile_plugin("startRecord", params)
      .map_err(|e| crate::Error::from(e))
  }
  
  pub fn configure_record(&self, width: u32, height: u32, fps: f64) -> crate::Result<bool> {
    use serde_json::json;
    
    // Create a simple JSON object
    let params = json!({
      "width": width,
      "height": height,
      "fps": fps
    });
    
    let response: BooleanResponse = self
      .0
      .run_mobile_plugin("configureRecord", params)
      .map_err(|e| crate::Error::from(e))?;
    Ok(response.success)
  }
  
  pub fn push_frame(&self, rgb: Vec<u8>, width: u32, height: u32) -> crate::Result<bool> {
    use serde_json::json;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    
    // For mobile, encode the RGB buffer as base64 string to pass through JNI
    // Future improvement: use binary protocol if/when Tauri supports it
    let b64_rgb = STANDARD.encode(&rgb);
    
    // Create a simple JSON object with the RGB data and dimensions
    let params = json!({
      "rgb": b64_rgb,
      "width": width,
      "height": height
    });
    
    let response: BooleanResponse = self
      .0
      .run_mobile_plugin("pushFrame", params)
      .map_err(|e| crate::Error::from(e))?;
    Ok(response.success)
  }
  
  pub fn stop_record(&self) -> crate::Result<bool> {
    use serde_json::json;
    
    // Create an empty JSON object
    let params = json!({});
    
    let response: BooleanResponse = self
      .0
      .run_mobile_plugin("stopRecord", params)
      .map_err(|e| crate::Error::from(e))?;
    Ok(response.success)
  }
}
