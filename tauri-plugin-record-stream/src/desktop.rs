use opencv::core::AlgorithmHint;
use opencv::{core, prelude::*, videoio};
// use opencv::imgproc;
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

use std::time::{Duration, Instant};
use std::collections::VecDeque;

pub struct VideoRecorder {
    writer: Option<videoio::VideoWriter>,
    path: String,
    width: i32,
    height: i32,
    fps: f64,
    is_recording: bool,
    frame_count: usize,
    start_time: Option<Instant>,
    last_frame_time: Option<Instant>,
    last_frame: Option<core::Mat>,     // Buffer for the last frame
    frame_queue: VecDeque<(Instant, core::Mat)>, // Queue of timestamped frames
    next_frame_time: Option<Instant>,  // When the next frame should be written
    frame_interval: Duration,          // Time between frames at target FPS
    cleanup_thread: Option<thread::JoinHandle<()>>, // Thread for cleanup operations
}

impl VideoRecorder {
    pub fn new() -> Self {
        VideoRecorder {
            writer: None,
            path: String::new(),
            width: 0,
            height: 0,
            fps: 0.0,
            is_recording: false,
            frame_count: 0,
            start_time: None,
            last_frame_time: None,
            last_frame: None,
            frame_queue: VecDeque::new(),
            next_frame_time: None,
            frame_interval: Duration::from_millis(100), // Default 10fps
            cleanup_thread: None,
        }
    }

    pub fn start(&mut self, path: String, width: i32, height: i32, fps: f64) -> crate::Result<bool> {
        // If there's a cleanup thread from a previous recording, check if it's finished
        // but don't block waiting for it
        if let Some(thread) = self.cleanup_thread.take() {
            if thread.is_finished() {
                // Thread is done, join it to clean up resources
                let _ = thread.join();
            } else {
                // Thread still running, don't block, just keep the reference
                self.cleanup_thread = Some(thread);
            }
        }
        
        if self.is_recording {
            return Ok(false); // Already recording
        }

        self.path = path.clone();
        self.width = width;
        self.height = height;
        self.fps = fps;
        self.is_recording = true;
        self.frame_count = 0;
        self.start_time = Some(Instant::now());
        self.last_frame_time = None;
        self.last_frame = None;
        self.frame_queue.clear();
        
        // Calculate frame interval based on target FPS
        // Frame interval is how much time should elapse between frames
        let frame_interval_ms = 1000.0 / fps;
        self.frame_interval = Duration::from_millis(frame_interval_ms as u64);
        
        // Set the first frame time to now
        let now = Instant::now();
        self.next_frame_time = Some(now + self.frame_interval);

        // Determine codec based on file extension
        let fourcc = if path.to_lowercase().ends_with(".mp4") {
            videoio::VideoWriter::fourcc('m', 'p', '4', 'v')?
        } else if path.to_lowercase().ends_with(".avi") {
            videoio::VideoWriter::fourcc('M', 'J', 'P', 'G')?
        } else {
            // Default to MP4 if extension not recognized
            videoio::VideoWriter::fourcc('m', 'p', '4', 'v')?
        };

        // Use the actual target FPS for the video writer
        // We'll manually control frame timing with our buffer system
        let writer = videoio::VideoWriter::new(
            &path, 
            fourcc, 
            fps,  // Use the exact FPS specified
            core::Size::new(width, height), 
            true
        )?;
        
        if !writer.is_opened()? {
            return Err(crate::Error::VideoWriterError(format!("Failed to open video writer for {}", path)));
        }

        println!("[record_plugin] Started recording to {} at {}x{} {}fps with frame buffering", 
                 path, width, height, fps);
        self.writer = Some(writer);
        Ok(true)
    }

    pub fn push_frame(&mut self, rgb: Vec<u8>, width: u32, height: u32) -> crate::Result<crate::FrameAnalysisResponse> {
        use opencv::imgproc;

        // Default response for early returns
        let failed_response = crate::FrameAnalysisResponse {
            success: false,
            is_covered: false,
            edge_count: 0,
        };

        if !self.is_recording || self.writer.is_none() {
            return Ok(failed_response); // Not recording
        }

        // basic validation
        if rgb.is_empty() || width == 0 || height == 0 {
            return Ok(failed_response);
        }

        let img_width = width as i32;
        let img_height = height as i32;

        // Build Mat from raw bytes (RGB)
        let src_mat = unsafe {
            // Create a Mat of the right size
            let mut mat = core::Mat::new_rows_cols(img_height, img_width, core::CV_8UC3)?;
            
            // Copy the raw byte data directly to the Mat
            let byte_size = (img_width * img_height * 3) as usize;
            if rgb.len() >= byte_size {
                let mat_data = mat.data_mut();
                if !mat_data.is_null() {
                    std::ptr::copy_nonoverlapping(rgb.as_ptr(), mat_data, byte_size);
                }
            }
            mat
        };

        // Convert to BGR for OpenCV writer and analysis
        let mut bgr_mat = core::Mat::default();
        imgproc::cvt_color(&src_mat, &mut bgr_mat, imgproc::COLOR_RGB2BGR, 0, AlgorithmHint::ALGO_HINT_DEFAULT)?;

        // Create a grayscale version for edge detection
        let mut gray = core::Mat::default();
        imgproc::cvt_color(&bgr_mat, &mut gray, imgproc::COLOR_BGR2GRAY, 0, AlgorithmHint::ALGO_HINT_DEFAULT)?;

        // Perform edge detection using Canny
        let mut edges = core::Mat::default();
        imgproc::canny(&gray, &mut edges, 50.0, 150.0, 3, false)?;

        // Count edge pixels - if very few edges, camera might be covered
        let edge_count = core::count_non_zero(&edges)?;
        let is_covered = edge_count < 500; // Threshold for considering camera covered
        
        // Log the edge detection results
        // println!("[record_plugin] Edge detection: {} edges found, covered: {}", edge_count, is_covered);

        // src_mat is no longer needed
        std::mem::drop(src_mat);

        // If incoming frame size doesn't match configured video size, resize
        if img_width != self.width || img_height != self.height {
            let mut resized = core::Mat::default();
            imgproc::resize(
                &bgr_mat,
                &mut resized,
                core::Size::new(self.width, self.height),
                0.0,
                0.0,
                imgproc::INTER_LINEAR,
            )?;
            bgr_mat = resized;
        }

        // Get current time for frame timestamp
        let now = Instant::now();

        // Add frame to the queue with its timestamp
        let frame_copy = bgr_mat.clone();
        self.frame_queue.push_back((now, frame_copy));

        // Also update last frame reference
        self.last_frame = Some(bgr_mat);

        // Update timing information
        self.last_frame_time = Some(now);

        // Process frame queue and write frames as needed
        self.process_frame_queue()?;

        // Return success with edge detection results
        Ok(crate::FrameAnalysisResponse {
            success: true,
            is_covered,
            edge_count,
        })
    }
    
    fn process_frame_queue(&mut self) -> crate::Result<()> {
        if self.writer.is_none() {
            return Ok(());
        }
        
        let now = Instant::now();
        if let Some(initial_next_time) = self.next_frame_time {
            let mut current_frame_time = initial_next_time;
            while current_frame_time <= now {
                let frame_to_write = if !self.frame_queue.is_empty() {
                    let (_frame_time, frame) = self.frame_queue.pop_front().unwrap();

                    Some(frame)
                } else if let Some(ref last_frame) = self.last_frame {
                    // No new frames available, duplicate the last one
                    // println!("[record_plugin] No new frame available, duplicating previous frame");
                    Some(last_frame.clone())
                } else {
                    // No frames at all yet
                    None
                };
                
                // Write the frame if we have one
                if let Some(frame) = frame_to_write {
                    if let Some(writer) = &mut self.writer {
                        writer.write(&frame)?;
                        self.frame_count += 1;
                        
                        // Log statistics periodically
                        if self.frame_count % 30 == 0 {
                            // Print overall stats
                            if let Some(start) = self.start_time {
                                let elapsed = now.duration_since(start).as_secs_f64();
                                let _target_fps = 1.0 / self.frame_interval.as_secs_f64();
                                let _current_fps = if elapsed > 0.0 { self.frame_count as f64 / elapsed } else { 0.0 };
                                // println!("[record_plugin] Frame {}: Target FPS: {:.1}, Actual average FPS: {:.2}", 
                                //          self.frame_count, target_fps, current_fps);
                            }
                        }
                    }
                }
                current_frame_time += self.frame_interval;
                
                self.next_frame_time = Some(current_frame_time);
                if now.duration_since(current_frame_time) > Duration::from_secs(1) {
                    // println!("[record_plugin] Warning: Over 1 second behind in frame processing. Skipping ahead.");
                    self.next_frame_time = Some(now + self.frame_interval);
                    break;
                }
            }
        } else {
            // Initialize next frame time if not set
            self.next_frame_time = Some(now + self.frame_interval);
        }
        
        Ok(())
    }

    pub fn stop(&mut self) -> crate::Result<bool> {
        if !self.is_recording {
            return Ok(false); // Not recording
        }

        self.is_recording = false;
        
        // Get data needed for cleanup thread
        let frame_count = self.frame_count;
        let writer = self.writer.take();
        let path = self.path.clone();
        let start_time_elapsed = self.start_time.map(|start| start.elapsed().as_secs_f64());
        
        // Spawn a thread to handle cleanup operations without blocking
        let cleanup_handle = thread::spawn(move || {
            // Calculate actual recorded FPS based on frame count and elapsed time
            if let Some(elapsed_secs) = start_time_elapsed {
                let actual_fps = if elapsed_secs > 0.0 {
                    frame_count as f64 / elapsed_secs
                } else {
                    0.0
                };
                
                println!("[record_plugin] Recording stopped. Frames: {}, Duration: {:.2}s, Actual FPS: {:.2}", 
                         frame_count, elapsed_secs, actual_fps);
            }
            
            // Explicitly drop the writer to ensure file is properly closed
            // This can be slow for large files, which is why we do it in a separate thread
            drop(writer);
            println!("[record_plugin] Finalized recording file: {}", path);
        });
        
        // Store the thread handle so we can join it later if needed
        self.cleanup_thread = Some(cleanup_handle);
        
        // Reset other state
        self.start_time = None;
        self.last_frame_time = None;

        Ok(true)
    }
}

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<RecordStream<R>> {
  let recorder = Arc::new(Mutex::new(VideoRecorder::new()));
  Ok(RecordStream {
    app: app.clone(),
    recorder,
  })
}

/// Access to the record-stream APIs.
pub struct RecordStream<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    recorder: Arc<Mutex<VideoRecorder>>,
}

impl<R: Runtime> RecordStream<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }

  pub fn start_record(&self, payload: StartRecordRequest) -> crate::Result<StartRecordResponse> {
    let mut recorder = self.recorder.lock().map_err(|_| crate::Error::MutexError)?;
    
    // Default values if not provided
    let width = 320; // Default width - more reasonable for modern cameras
    let height = 240; // Default height - more reasonable for modern cameras
    let fps = 20.0; // Default fps - reduced to avoid playback acceleration
  
    match recorder.start(payload.file_path, width as i32, height as i32, fps) {
      Ok(true) => {
        println!("[record_plugin] Recording started successfully");
        Ok(StartRecordResponse { success: true })
      },
      Ok(false) => {
        println!("[record_plugin] Already recording");
        Ok(StartRecordResponse { success: false })
      },
      Err(e) => {
        println!("[record_plugin] Failed to start recording: {:?}", e);
        Err(e)
      }
    }
  }
  
  pub fn push_frame(&self, rgb: Vec<u8>, width: u32, height: u32) -> crate::Result<crate::FrameAnalysisResponse> {
    let mut recorder = self.recorder.lock().map_err(|_| crate::Error::MutexError)?;
    recorder.push_frame(rgb, width, height)
  }
  
  pub fn stop_record(&self) -> crate::Result<bool> {
    let mut recorder = self.recorder.lock().map_err(|_| crate::Error::MutexError)?;
    recorder.stop()
  }
}
