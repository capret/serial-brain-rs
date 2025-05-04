use std::{
    collections::VecDeque,
    fs::File,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use crate::types::ChannelData;

/// Shared state for serial connection.
pub struct SerialState {
    pub outbound_tx: Mutex<Option<Sender<String>>>,
    pub active_buffer: Mutex<VecDeque<ChannelData>>,
    pub read_buffer: Mutex<VecDeque<ChannelData>>,
    pub recording_buffer: Mutex<VecDeque<(SystemTime, ChannelData)>>, // Dedicated buffer for recording with timestamps
    pub quality_check_buffer: Arc<Mutex<VecDeque<ChannelData>>>, // Dedicated buffer for signal quality calculation
    pub signal_quality: Arc<Mutex<Vec<bool>>>, // Signal quality indicators for each channel (true = good, false = bad)
    pub buffer_lock: Mutex<()>,
    pub signal_stream_running: Arc<AtomicBool>,
    pub signal_stream_handle: Mutex<Option<JoinHandle<()>>>,
    pub camera_stream_running: Arc<AtomicBool>,
    pub camera_stream_handle: Mutex<Option<JoinHandle<()>>>,
    pub recording_active: Arc<AtomicBool>,
    pub recording_handle: Mutex<Option<JoinHandle<()>>>,
    pub recording_file: Mutex<Option<(File, String)>>,
    pub quality_check_running: Arc<AtomicBool>,
    pub quality_check_handle: Mutex<Option<JoinHandle<()>>>,
    
}

impl SerialState {
    pub fn new() -> Self {
        let state = SerialState {
            outbound_tx: Mutex::new(None),
            active_buffer: Mutex::new(VecDeque::new()),
            read_buffer: Mutex::new(VecDeque::new()),
            recording_buffer: Mutex::new(VecDeque::new()),
            quality_check_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(500))),
            signal_quality: Arc::new(Mutex::new(vec![true; 8])), // Initialize with 8 channels, all good quality
            buffer_lock: Mutex::new(()),
            signal_stream_running: Arc::new(AtomicBool::new(false)),
            signal_stream_handle: Mutex::new(None),
            camera_stream_running: Arc::new(AtomicBool::new(false)),
            camera_stream_handle: Mutex::new(None),
            recording_active: Arc::new(AtomicBool::new(false)),
            recording_handle: Mutex::new(None),
            recording_file: Mutex::new(None),
            quality_check_running: Arc::new(AtomicBool::new(false)),
            quality_check_handle: Mutex::new(None),
        };
        
        state
    }

    pub fn add_data(&self, data: ChannelData) {
        let _guard = self.buffer_lock.lock().unwrap();

        // Update active buffer for visualization
        let mut active_buf = self.active_buffer.lock().unwrap();
        if active_buf.len() >= 2000 {
            active_buf.pop_front();
        }
        active_buf.push_back(data);

        // Update read buffer for regular data retrieval
        let mut read_buf = self.read_buffer.lock().unwrap();
        if read_buf.len() >= 2000 {
            read_buf.pop_front();
        }
        read_buf.push_back(data);
        
        // Update quality check buffer
        let mut quality_buf = self.quality_check_buffer.lock().unwrap();
        if quality_buf.len() >= 500 {
            quality_buf.pop_front();
        }
        quality_buf.push_back(data);
        
        // Add to recording buffer if recording is active
        if self.recording_active.load(std::sync::atomic::Ordering::SeqCst) {
            let mut recording_buf = self.recording_buffer.lock().unwrap();
            let timestamp = SystemTime::now();
            recording_buf.push_back((timestamp, data));
            
            // Limit the recording buffer size to prevent memory issues
            // This is a large size to ensure we don't lose data during recording
            if recording_buf.len() >= 10000 {
                recording_buf.pop_front();
            }
        }
    }

    pub fn get_data(&self) -> Vec<ChannelData> {
        let _guard = self.buffer_lock.lock().unwrap();

        let mut read_buf = self.read_buffer.lock().unwrap();
        let result: Vec<ChannelData> = read_buf.iter().cloned().collect();

        read_buf.clear();

        result
    }

    // New method to get recording data with timestamps
    pub fn get_recording_data(&self) -> Vec<(SystemTime, ChannelData)> {
        let _guard = self.buffer_lock.lock().unwrap();

        let mut recording_buf = self.recording_buffer.lock().unwrap();
        let result: Vec<(SystemTime, ChannelData)> = recording_buf.drain(..).collect();

        result
    }
    
    // Get the current signal quality indicators for all channels
    pub fn get_signal_quality(&self) -> Vec<bool> {
        let signal_quality = self.signal_quality.lock().unwrap();
        signal_quality.clone()
    }
    
    // Check signal quality of the channel data (runs on-demand, not in a separate thread)
    pub fn check_signal_quality(&self) -> Vec<bool> {
        // Get a copy of the quality check buffer
        let quality_data: Vec<ChannelData> = {
            let buffer = self.quality_check_buffer.lock().unwrap();
            buffer.iter().cloned().collect()
        };
        
        // If we don't have enough data, return current quality status
        if quality_data.len() < 10 {
            return self.get_signal_quality();
        }
        
        // Calculate statistics for each channel
        let mut signal_quality_guard = self.signal_quality.lock().unwrap();
        
        // For each of the 8 channels
        for channel in 0..8 {
            // Extract this channel's data
            let values: Vec<f32> = quality_data.iter().map(|data| data[channel]).collect();
            
            // Calculate mean
            let sum: f32 = values.iter().sum();
            let mean = sum / values.len() as f32;
            
            // Calculate standard deviation
            let var_sum: f32 = values.iter()
                .map(|&v| (v - mean).powi(2))
                .sum();
            let std_dev = (var_sum / values.len() as f32).sqrt();
            
            // Update quality flag based on criteria
            // Flag as poor quality (false) if std_dev > 2.235e4 or mean > 1.341e5
            signal_quality_guard[channel] = std_dev <= 2.235e4 && mean <= 1.341e5;
        }
        
        // Return a copy of the updated quality indicators
        signal_quality_guard.clone()
    }
    
    // Clear the quality check buffer
    pub fn clear_quality_buffer(&self) {
        let mut buffer = self.quality_check_buffer.lock().unwrap();
        buffer.clear();
    }
}
