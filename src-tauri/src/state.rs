use std::{
    collections::VecDeque,
    fs::File,
    sync::{atomic::AtomicBool, mpsc::Sender, Arc, Mutex},
    thread::JoinHandle,
    time::SystemTime,
};

use crate::types::ChannelData;

/// Shared state for serial connection.
pub struct SerialState {
    pub outbound_tx: Mutex<Option<Sender<String>>>,
    pub active_buffer: Mutex<VecDeque<ChannelData>>,
    pub read_buffer: Mutex<VecDeque<ChannelData>>,
    pub recording_buffer: Mutex<VecDeque<(SystemTime, ChannelData)>>, // Dedicated buffer for recording with timestamps
    pub buffer_lock: Mutex<()>,
    pub signal_stream_running: Arc<AtomicBool>,
    pub signal_stream_handle: Mutex<Option<JoinHandle<()>>>,
    pub camera_stream_running: Arc<AtomicBool>,
    pub camera_stream_handle: Mutex<Option<JoinHandle<()>>>,
    pub recording_active: Arc<AtomicBool>,
    pub recording_handle: Mutex<Option<JoinHandle<()>>>,
    pub recording_file: Mutex<Option<(File, String)>>,
}

impl SerialState {
    pub fn new() -> Self {
        SerialState {
            outbound_tx: Mutex::new(None),
            active_buffer: Mutex::new(VecDeque::new()),
            read_buffer: Mutex::new(VecDeque::new()),
            recording_buffer: Mutex::new(VecDeque::new()),
            buffer_lock: Mutex::new(()),
            signal_stream_running: Arc::new(AtomicBool::new(false)),
            signal_stream_handle: Mutex::new(None),
            camera_stream_running: Arc::new(AtomicBool::new(false)),
            camera_stream_handle: Mutex::new(None),
            recording_active: Arc::new(AtomicBool::new(false)),
            recording_handle: Mutex::new(None),
            recording_file: Mutex::new(None),
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
}
