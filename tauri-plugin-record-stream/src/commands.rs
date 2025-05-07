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
