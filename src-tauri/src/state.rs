use std::{
    collections::VecDeque,
    fs::File,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
    thread::JoinHandle,
    time::SystemTime,
};

use tauri::{AppHandle};
use libmdns::Responder;

use crate::types::ChannelData;

// ==== Communication State ====
/// Manages serial/socket communication channels
pub struct CommunicationState {
    pub outbound_tx: Mutex<Option<Sender<String>>>,
    pub app_handle: Mutex<Option<AppHandle>>, // For emitting events
}

impl CommunicationState {
    pub fn new() -> Self {
        Self {
            outbound_tx: Mutex::new(None),
            app_handle: Mutex::new(None),
        }
    }
}

// ==== Buffer State ====
/// Manages data buffers for visualization and processing
pub struct BufferState {
    pub active_buffer: Mutex<VecDeque<ChannelData>>,
    pub read_buffer: Mutex<VecDeque<ChannelData>>,
    pub buffer_lock: Mutex<()>,
}

impl BufferState {
    pub fn new() -> Self {
        Self {
            active_buffer: Mutex::new(VecDeque::new()),
            read_buffer: Mutex::new(VecDeque::new()),
            buffer_lock: Mutex::new(()),
        }
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
    }

    pub fn get_data(&self) -> Vec<ChannelData> {
        let _guard = self.buffer_lock.lock().unwrap();

        let mut read_buf = self.read_buffer.lock().unwrap();
        let result: Vec<ChannelData> = read_buf.iter().cloned().collect();

        read_buf.clear();

        result
    }
}

// ==== Signal Quality State ====
/// Manages signal quality monitoring and analysis
pub struct SignalQualityState {
    pub quality_check_buffer: Arc<Mutex<VecDeque<ChannelData>>>,
    pub signal_quality: Arc<Mutex<Vec<bool>>>, // Signal quality indicators for each channel (true = good, false = bad)
}

impl SignalQualityState {
    pub fn new() -> Self {
        Self {
            quality_check_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(500))),
            signal_quality: Arc::new(Mutex::new(vec![true; 8])), // Initialize with 8 channels, all good quality
        }
    }

    pub fn add_data(&self, data: ChannelData) {
        // Update quality check buffer
        let mut quality_buf = self.quality_check_buffer.lock().unwrap();
        if quality_buf.len() >= 500 {
            quality_buf.pop_front();
        }
        quality_buf.push_back(data);
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
}

// ==== Stream State ====
/// Manages data streaming (both signal and camera)
pub struct StreamState {
    pub signal_stream_running: Arc<AtomicBool>,
    pub signal_stream_handle: Mutex<Option<JoinHandle<()>>>,
    pub camera_stream_running: Arc<AtomicBool>,
    pub camera_stream_handle: Mutex<Option<JoinHandle<()>>>,
    pub fake_data_enabled: Arc<AtomicBool>, // Flag to track if fake data is enabled
}

impl StreamState {
    pub fn new() -> Self {
        Self {
            signal_stream_running: Arc::new(AtomicBool::new(false)),
            signal_stream_handle: Mutex::new(None),
            camera_stream_running: Arc::new(AtomicBool::new(false)),
            camera_stream_handle: Mutex::new(None),
            fake_data_enabled: Arc::new(AtomicBool::new(false)), // Initialize fake data as disabled
        }
    }
}

// ==== Recording State ====
/// Manages recording functionality (both signal and video)
pub struct RecordingState {
    pub recording_buffer: Mutex<VecDeque<(SystemTime, ChannelData)>>, // Dedicated buffer for recording with timestamps
    pub recording_active: Arc<AtomicBool>,
    pub recording_handle: Mutex<Option<JoinHandle<()>>>,
    pub recording_file: Mutex<Option<(File, String)>>,
    pub recording_filename: Mutex<Option<String>>, // Store current recording filename
    pub video_recording_active: Arc<AtomicBool>, // Flag for video recording
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            recording_buffer: Mutex::new(VecDeque::new()),
            recording_active: Arc::new(AtomicBool::new(false)),
            recording_handle: Mutex::new(None),
            recording_file: Mutex::new(None),
            recording_filename: Mutex::new(None),
            video_recording_active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn add_data(&self, data: ChannelData) {
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

    // Get recording data with timestamps
    pub fn get_recording_data(&self) -> Vec<(SystemTime, ChannelData)> {
        let mut recording_buf = self.recording_buffer.lock().unwrap();
        let result: Vec<(SystemTime, ChannelData)> = recording_buf.drain(..).collect();
        result
    }
}

// ==== MDNS State ====
/// Manages mDNS service for discovery on local network
pub struct MdnsState {
    pub responder: Mutex<Option<Responder>>,
    pub service: Mutex<Option<libmdns::Service>>,
    pub active: Mutex<bool>,
    pub host: Mutex<String>,
    pub port: Mutex<u16>,
    pub discovered_devices: Mutex<Vec<crate::mdns::MdnsDevice>>,
}

impl MdnsState {
    pub fn new() -> Self {
        Self {
            responder: Mutex::new(None),
            service: Mutex::new(None),
            active: Mutex::new(false),
            host: Mutex::new(String::new()),
            port: Mutex::new(0),
            discovered_devices: Mutex::new(Vec::new()),
        }
    }

    /// Check if the mDNS service is active
    pub fn is_active(&self) -> bool {
        *self.active.lock().unwrap()
    }
}


/// Composite state struct that manages all application state
pub struct AppState {
    pub communication: CommunicationState,
    pub buffer: BufferState,
    pub signal_quality: SignalQualityState,
    pub stream: StreamState,
    pub recording: RecordingState,
    pub mdns: MdnsState,
}


impl AppState {
    pub fn new() -> Self {
        Self {
            communication: CommunicationState::new(),
            buffer: BufferState::new(),
            signal_quality: SignalQualityState::new(),
            stream: StreamState::new(),
            recording: RecordingState::new(),
            mdns: MdnsState::new(),
            }
    }
    // Forward methods to appropriate sub-states for backward compatibility
    pub fn get_data(&self) -> Vec<ChannelData> {
        self.buffer.get_data()
    }
    pub fn check_signal_quality(&self) -> Vec<bool> {
        self.signal_quality.check_signal_quality()
    }
}

