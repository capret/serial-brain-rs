use tauri::{AppHandle, command, Runtime};

use crate::models::*;
use crate::Result;
use crate::AndroidForwardServiceExt;

#[command]
pub(crate) async fn ping<R: Runtime>(
    app: AppHandle<R>,
    payload: PingRequest,
) -> Result<PingResponse> {
    app.android_forward_service().ping(payload)
}

#[command]
pub(crate) async fn start_forward_service<R: Runtime>(
    app: AppHandle<R>,
) -> Result<()> {
    app.android_forward_service().start_service()
}

#[command]
pub(crate) async fn stop_forward_service<R: Runtime>(
    app: AppHandle<R>,
) -> Result<()> {
    app.android_forward_service().stop_service()
}
