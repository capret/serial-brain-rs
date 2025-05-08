use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::RecordStreamExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.record_stream().ping(payload)
}

#[command]
pub(crate) async fn start_record<R: Runtime>(
    app: AppHandle<R>,
    payload: StartRecordRequest,
) -> Result<StartRecordResponse> {
    println!("[record_plugin] Recording Stream is called");
    app.record_stream().start_record(payload)
}

#[command]
pub(crate) fn push_frame<R: Runtime>(
    app: AppHandle<R>,
    rgb: Vec<u8>,
    width: u32, 
    height: u32,
) -> Result<crate::FrameAnalysisResponse> {
    app.record_stream().push_frame(rgb, width, height)
}

#[command]
pub(crate) fn stop_record<R: Runtime>(
    app: AppHandle<R>,
) -> Result<bool> {
    println!("[record_plugin] Stopping recording");
    app.record_stream().stop_record()
}

#[command]
pub(crate) fn configure_record<R: Runtime>(
    app: AppHandle<R>,
    width: u32,
    height: u32,
    fps: f64,
) -> Result<bool> {
    println!("[record_plugin] Configuring recording: {}x{} @ {}fps", width, height, fps);
    crate::configure_record(app, width, height, fps)
}
