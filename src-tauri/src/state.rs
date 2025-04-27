use std::{
    collections::VecDeque,
    sync::{
        atomic::AtomicBool,
        mpsc::Sender,
        Arc, Mutex,
    },
    thread::JoinHandle,
};

use crate::types::ChannelData;

/// Shared state for serial connection.
pub struct SerialState {
    pub outbound_tx: Mutex<Option<Sender<String>>>,
    pub active_buffer: Mutex<VecDeque<ChannelData>>,
    pub read_buffer: Mutex<VecDeque<ChannelData>>,
    pub buffer_lock: Mutex<()> ,
    pub signal_stream_running: Arc<AtomicBool>,
    pub signal_stream_handle: Mutex<Option<JoinHandle<()>>>,
    pub camera_stream_running: Arc<AtomicBool>,
    pub camera_stream_handle: Mutex<Option<JoinHandle<()>>>,
}

impl SerialState {
    pub fn new() -> Self {
        Self {
            outbound_tx: Mutex::new(None),
            active_buffer: Mutex::new(VecDeque::with_capacity(2000)),
            read_buffer: Mutex::new(VecDeque::with_capacity(2000)),
            buffer_lock: Mutex::new(()),
            signal_stream_running: Arc::new(AtomicBool::new(false)),
            signal_stream_handle: Mutex::new(None),
            camera_stream_running: Arc::new(AtomicBool::new(false)),
            camera_stream_handle: Mutex::new(None),
        }
    }

    pub fn add_data(&self, data: ChannelData) {
        let _guard = self.buffer_lock.lock().unwrap();

        let mut active_buf = self.active_buffer.lock().unwrap();
        if active_buf.len() >= 2000 {
            active_buf.pop_front();
        }
        active_buf.push_back(data);

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
