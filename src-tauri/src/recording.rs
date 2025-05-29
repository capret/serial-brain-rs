//! Recording module
//! ------------------------------------------------------------
//! Provides an ergonomic, maintainable wrapper around data- and
//! video‑recording for the serial‑visualisation app.  The public
//! API is **unchanged** so existing UI calls continue to work:
//!
//! ```rust
//! start_recording(...)
//! stop_recording(...)
//! start_video_recording(...)
//! stop_video_recording(...)
//! ```
//!
//! Key internals:
//! * `Format` – strongly‑typed recording format enum
//! * `DataWriter` trait + concrete `CsvWriter`, `JsonWriter`, `BinaryWriter`
//! * `RecordingController` – background thread handling IO + rotation
//! * `video` sub‑module – thin wrapper around `tauri_plugin_record_stream`
//!
//! ----------------------------------------------------------------

use crate::state::AppState;
use serde_json::json;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{atomic::Ordering, Arc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Manager, Emitter};

// -------------------------------------------------------------------------------------------------
// Public helpers -------------------------------------------------------------------------------------------------

/// Starts recording serial data (and starts a matching video recording).
pub fn start_recording(
    format: String,
    directory: String,
    max_duration_minutes: u32,
    _auto_start: bool,
    app_handle: AppHandle,
) -> Result<String, String> {
    // Validate options -----------------------------------------------------------------------------
    let format = Format::try_from(format.as_str())?;
    let max_duration = Duration::from_secs(max_duration_minutes as u64 * 60);
    let base_name = base_filename();

    // Kick off video (non‑fatal on failure) ---------------------------------------------------------
    if let Err(e) = start_video_recording(app_handle.clone(), base_name.clone(), directory.clone()) {
        eprintln!("[Recording] ⚠️  Failed to start video recording: {e}");
    }

    // First data segment ---------------------------------------------------------------------------
    let mut first_path = PathBuf::from(&directory);
    first_path.push(format!("{base_name}.{}", format.extension()));
    let writer = new_segment_writer(&first_path, format)?;

    // Replace any running recording ---------------------------------------------------------------
    let state = app_handle.state::<Arc<AppState>>();
    if let Some(h) = state.recording.recording_handle.lock().unwrap().take() {
        let _ = h.join();
    }

    // Spawn controller thread ---------------------------------------------------------------------
    let handle = RecordingController::spawn(
        writer,
        format,
        directory.clone(),
        max_duration,
        app_handle.clone(),
    );
    *state.recording.recording_handle.lock().unwrap() = Some(handle);
    state.recording.recording_active.store(true, Ordering::SeqCst);

    Ok(first_path.file_name().unwrap().to_string_lossy().into())
}

/// Stops the current recording (data + video).
pub fn stop_recording(app_handle: AppHandle) -> Result<(), String> {
    let state = app_handle.state::<Arc<AppState>>();
    state
        .recording
        .recording_active
        .store(false, Ordering::SeqCst);

    if let Some(h) = state.recording.recording_handle.lock().unwrap().take() {
        let _ = h.join();
    }

    let _ = stop_video_recording(app_handle);
    Ok(())
}

// -- Public wrappers matching legacy signatures ---------------------------------------------------

pub fn start_video_recording(
    app_handle: AppHandle,
    base_filename: String,
    directory: String,
) -> Result<bool, String> {
    video::start_video_recording(app_handle, &base_filename, &directory)
}

pub fn stop_video_recording(app_handle: AppHandle) -> Result<bool, String> {
    video::stop_video_recording(app_handle)
}

// -------------------------------------------------------------------------------------------------
// Internal types -------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format { Csv, Json, Binary }
impl TryFrom<&str> for Format {
    type Error = String;
    fn try_from(v: &str) -> Result<Self, Self::Error> {
        match v.to_ascii_lowercase().as_str() {
            "csv" => Ok(Format::Csv),
            "json" => Ok(Format::Json),
            "binary" | "bin" => Ok(Format::Binary),
            _ => Err(format!("Invalid format '{v}'. Expected csv | json | binary")),
        }
    }
}
impl Format { fn extension(self) -> &'static str { match self { Self::Csv => "csv", Self::Json => "json", Self::Binary => "bin" } } }

// Data‑writer abstraction -------------------------------------------------------------------------
trait DataWriter: Send {
    fn write_batch(&mut self, batch: &[(SystemTime, [f32; 8])]) -> std::io::Result<()>;
    fn finalize(&mut self) -> std::io::Result<()>;
}

struct CsvWriter { file: BufWriter<File> }
struct JsonWriter { file: BufWriter<File>, first: bool }
struct BinaryWriter { file: BufWriter<File> }

impl CsvWriter { fn new(mut f: File) -> std::io::Result<Self> {
    writeln!(f, "timestamp,{}", (0..8).map(|i| format!("channel_{i}")).collect::<Vec<_>>().join(","))?;
    Ok(Self{file:BufWriter::new(f)}) }}
impl DataWriter for CsvWriter {
    fn write_batch(&mut self, batch:&[(SystemTime,[f32;8])]) -> std::io::Result<()> {
        for (t,ch) in batch {
            let ts = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis();
            writeln!(self.file, "{ts},{}", ch.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","))?;
        }
        self.file.flush()
    }
    fn finalize(&mut self)->std::io::Result<()> { self.file.flush() }
}

impl JsonWriter { fn new(mut f: File)->std::io::Result<Self>{ f.write_all(b"[")?; Ok(Self{file:BufWriter::new(f),first:true}) }}
impl DataWriter for JsonWriter {
    fn write_batch(&mut self, batch:&[(SystemTime,[f32;8])]) -> std::io::Result<()> {
        for (t,ch) in batch {
            if !self.first { self.file.write_all(b",")?; }
            self.first=false;
            let ts=t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis();
            let entry=json!({"timestamp":ts,"values":ch});
            self.file.write_all(entry.to_string().as_bytes())?;
        }
        self.file.flush()
    }
    fn finalize(&mut self)->std::io::Result<()> { self.file.write_all(b"]")?; self.file.flush() }
}

impl BinaryWriter { fn new(f: File)->std::io::Result<Self>{ Ok(Self{file:BufWriter::new(f)}) }}
impl DataWriter for BinaryWriter {
    fn write_batch(&mut self,batch:&[(SystemTime,[f32;8])]) -> std::io::Result<()> {
        for (t,ch) in batch {
            let ts = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
            self.file.write_all(&ts.to_le_bytes())?;
            self.file.write_all(&(ch.len() as u32).to_le_bytes())?;
            for v in ch { self.file.write_all(&(*v as f64).to_le_bytes())?; }
        }
        self.file.flush()
    }
    fn finalize(&mut self)->std::io::Result<()> { self.file.flush() }
}

fn new_segment_writer(path:&Path,fmt:Format)->Result<Box<dyn DataWriter>,String>{
    let file=OpenOptions::new().write(true).create(true).truncate(true).open(path)
        .map_err(|e|format!("Failed to create recording file: {e}"))?;
    match fmt {
        Format::Csv=>Ok(Box::new(CsvWriter::new(file).map_err(|e|e.to_string())?)),
        Format::Json=>Ok(Box::new(JsonWriter::new(file).map_err(|e|e.to_string())?)),
        Format::Binary=>Ok(Box::new(BinaryWriter::new(file).map_err(|e|e.to_string())?)),
    }
}

// Controller thread -------------------------------------------------------------------------------
struct RecordingController;
impl RecordingController {
    fn spawn(
        mut writer: Box<dyn DataWriter>,
        fmt: Format,
        directory: String,
        max_duration: Duration,
        app_handle: AppHandle,
    ) -> JoinHandle<()> {
        let state = app_handle.state::<Arc<AppState>>().inner().clone();
        thread::spawn(move || {
            let mut segment_start = SystemTime::now();
            loop {
                if !state.recording.recording_active.load(Ordering::SeqCst) { break; }

                // Rotate segment -----------------------------------------------------------
                if segment_start.elapsed().unwrap_or_default() >= max_duration {
                    let _=writer.finalize();
                    let base = base_filename();
                    let mut new_path=PathBuf::from(&directory);
                    new_path.push(format!("{base}.{}",fmt.extension()));
                    match new_segment_writer(&new_path,fmt) {
                        Ok(w)=>{ writer=w; segment_start=SystemTime::now();
                            // notify UI
                            if let Some(ui)=state.communication.app_handle.lock().unwrap().as_ref(){
                                let fname=new_path.file_name().unwrap().to_string_lossy();
                                let _=ui.emit("recording-filename-changed",fname.clone());
                            }
                            // rotate video
                            video::rotate_video_segment(&state,&directory,&base);
                        },
                        Err(e)=>eprintln!("[Recording] ⚠️  segment rotation failed: {e}"),
                    }
                    continue;
                }

                // Consume data -------------------------------------------------------------
                let batch=state.recording.get_recording_data();
                if batch.is_empty(){ thread::sleep(Duration::from_millis(10)); continue; }
                if let Err(e)=writer.write_batch(&batch){ eprintln!("[Recording] ⚠️  write error: {e}"); }
            }
            let _ = writer.finalize();
        })
    }
}

// Utility -----------------------------------------------------------------------------------------
fn base_filename() -> String {
    format!("serial_recording_{}", SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis())
}

// -------------------------------------------------------------------------------------------------
// Video sub‑module (private) -------------------------------------------------------------------------------------------------

mod video {
    use super::*;
    use tauri_plugin_record_stream as rstream;

    pub(super) fn start_video_recording(app_handle: AppHandle, base: &str, dir: &str) -> Result<bool,String>{
        let full=Path::new(dir).join(format!("{base}.mp4")).to_string_lossy().into_owned();
        // Directly call the plugin function without creating a new runtime
        let res = rstream::start_record(app_handle.clone(), rstream::StartRecordRequest{file_path:full}).map_err(|e|e.to_string())?;
        if res.success { app_handle.state::<Arc<AppState>>().recording.video_recording_active.store(true,Ordering::SeqCst);} 
        Ok(res.success)
    }

    pub(super) fn stop_video_recording(app_handle: AppHandle)->Result<bool,String>{
        app_handle.state::<Arc<AppState>>().recording.video_recording_active.store(false,Ordering::SeqCst);
        rstream::stop_record(app_handle).map_err(|e|e.to_string())
    }

    pub(super) fn rotate_video_segment(state:&AppState, dir:&str, new_base:&str){
        println!("[Recording] Rotating video segment {}", new_base);
        if let Some(app)=state.communication.app_handle.lock().unwrap().clone(){
            println!("[Recording] Stopping video recording");
            let ret=stop_video_recording(app.clone());
            if ret.is_err(){ eprintln!("[Recording] ⚠️  Failed to stop video recording: {}", ret.unwrap_err()); }
            // println!("[Recording] Waiting 400ms");
            // std::thread::sleep(Duration::from_millis(400));
            println!("[Recording] Starting video recording");
            let ret=start_video_recording(app,new_base,dir);
            if ret.is_err(){ eprintln!("[Recording] ⚠️  Failed to start video recording: {}", ret.unwrap_err()); }
        }}
}
