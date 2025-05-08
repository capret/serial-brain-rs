use base64::{Engine as _, engine::general_purpose::STANDARD};
use opencv::{core, prelude::*, videoio};
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};
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
        }
    }

    pub fn start(&mut self, path: String, width: i32, height: i32, fps: f64) -> crate::Result<bool> {
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

    pub fn push_frame(&mut self, b64_png: String) -> crate::Result<bool> {
        if !self.is_recording || self.writer.is_none() {
            return Ok(false); // Not recording
        }

        // Decode base64 PNG
        let png_data = STANDARD.decode(b64_png)?;
        
        // Convert PNG to image
        let mut img = image::load_from_memory(&png_data)?
            .to_rgb8();
        
        // Get actual image dimensions
        let img_width = img.width() as i32;
        let img_height = img.height() as i32;
        
        // Make sure we don't have zero dimensions
        if img_width <= 0 || img_height <= 0 {
            return Err(crate::Error::VideoWriterError(format!("Invalid image dimensions: {}x{}", img_width, img_height)));
        }
        
        // Check if image dimensions match the expected video dimensions
        // This is crucial for proper recording - we need to ensure consistent frame sizes
        if img_width != self.width || img_height != self.height {
            println!("[record_plugin] Image dimensions ({}x{}) don't match video dimensions ({}x{}). Resizing...",
                img_width, img_height, self.width, self.height);
                
            // Resize the image to match the video dimensions
            // Create a new buffer with the correct dimensions
            let resized = image::imageops::resize(
                &img, 
                self.width as u32, 
                self.height as u32, 
                image::imageops::FilterType::Triangle
            );
            img = resized;
        }
        
        // Get the raw image data after potential resize
        let img_data = img.as_raw();
        
        // Calculate expected sizes based on dimensions
        let expected_size = (self.width * self.height * 3) as usize;
        let actual_size = img_data.len();
        
        // Verify sizes match expectations
        if expected_size != actual_size {
            println!("[record_plugin] Warning after resize: Expected size {}, actual size {}", 
                    expected_size, actual_size);
            return Err(crate::Error::VideoWriterError(
                format!("Size mismatch after resize: expected {}, got {}", expected_size, actual_size)
            ));
        }
        
        // Create a new matrix with proper dimensions
        let mat = unsafe {
            // Create a matrix of the right size
            let mut m = core::Mat::new_rows_cols(self.height, self.width, core::CV_8UC3)?;
            
            // Get a pointer to the matrix data - this returns a raw pointer
            let data_ptr = m.data_mut();
            
            // Make sure the pointer is valid
            if !data_ptr.is_null() {
                // Copy data safely - both pointers are known to be valid for their respective sizes
                let src_ptr = img_data.as_ptr();
                
                // Use copy_nonoverlapping which is safer for non-overlapping memory regions
                std::ptr::copy_nonoverlapping(src_ptr, data_ptr, expected_size);
            } else {
                return Err(crate::Error::VideoWriterError("Failed to get valid matrix pointer".into()));
            }
            
            m
        };
        
        // Instead of using cvt_color which has inconsistent signatures,
        // let's just manually swap R and B channels which is what we need
        // RGB to BGR conversion
        let height = mat.rows();
        let width = mat.cols();
        let mut bgr_mat = mat.clone();
        
        // Manual RGB to BGR conversion (swap R and B channels)
        for y in 0..height {
            for x in 0..width {
                let pixel = mat.at_2d::<core::Vec3b>(y, x)?;
                let mut new_pixel = pixel.clone();
                // Swap R and B (BGR format needs B in 0, G in 1, R in 2)
                new_pixel[0] = pixel[2]; // B <- R
                new_pixel[2] = pixel[0]; // R <- B
                let bgr_ptr = bgr_mat.at_2d_mut::<core::Vec3b>(y, x)?;
                // Copy each value individually
                bgr_ptr[0] = new_pixel[0];
                bgr_ptr[1] = new_pixel[1];
                bgr_ptr[2] = new_pixel[2];
            }
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
        
        Ok(true)
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
                    let (frame_time, frame) = self.frame_queue.pop_front().unwrap();

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
        
        // Calculate actual recorded FPS based on frame count and elapsed time
        if let Some(start_time) = self.start_time {
            let elapsed_secs = start_time.elapsed().as_secs_f64();
            let _actual_fps = if elapsed_secs > 0.0 {
                self.frame_count as f64 / elapsed_secs
            } else {
                0.0
            };
            
            // println!("[record_plugin] Recording stopped. Frames: {}, Duration: {:.2}s, Actual FPS: {:.2}", 
            //          self.frame_count, elapsed_secs, actual_fps);
        }
        
        self.writer = None;
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
  
  pub fn push_frame(&self, b64_png: String) -> crate::Result<bool> {
    let mut recorder = self.recorder.lock().map_err(|_| crate::Error::MutexError)?;
    recorder.push_frame(b64_png)
  }
  
  pub fn stop_record(&self) -> crate::Result<bool> {
    let mut recorder = self.recorder.lock().map_err(|_| crate::Error::MutexError)?;
    recorder.stop()
  }
}
