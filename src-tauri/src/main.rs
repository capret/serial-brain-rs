#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use rand::Rng;
use serde::Deserialize;
use serialport;
use tauri::Emitter;
use tauri::{self, AppHandle, State}; // Import the Emitter trait to bring the `emit` method into scope.

// Define a type alias for the eight channel data.
type ChannelData = [f32; 8];

#[derive(Debug, Deserialize)]
struct FakeDataConfig {
    min_value: i32,
    max_value: i32,
    frequency: f64,
    channel_count: usize,
    waveform: String,
}

/// Shared state for our serial connection.
struct SerialState {
    outbound_tx: Mutex<Option<Sender<String>>>,
    active_buffer: Mutex<VecDeque<ChannelData>>,
    read_buffer: Mutex<VecDeque<ChannelData>>,
    buffer_lock: Mutex<()>,
    fake_stream_running: Arc<AtomicBool>,
    fake_stream_handle: Mutex<Option<JoinHandle<()>>>,
}

impl SerialState {
    fn new() -> Self {
        Self {
            outbound_tx: Mutex::new(None),
            active_buffer: Mutex::new(VecDeque::with_capacity(50000)),
            read_buffer: Mutex::new(VecDeque::with_capacity(50000)),
            buffer_lock: Mutex::new(()),
            fake_stream_running: Arc::new(AtomicBool::new(false)),
            fake_stream_handle: Mutex::new(None),
        }
    }

    fn add_data(&self, data: ChannelData) {
        // Acquire the lock to ensure consistency during writes
        let _guard = self.buffer_lock.lock().unwrap();
        
        // Add data to both buffers to ensure no data loss
        // Active buffer
        let mut active_buf = self.active_buffer.lock().unwrap();
        if active_buf.len() >= 50000 {
            active_buf.pop_front();
        }
        active_buf.push_back(data);
        
        // Read buffer
        let mut read_buf = self.read_buffer.lock().unwrap();
        if read_buf.len() >= 50000 {
            read_buf.pop_front();
        }
        read_buf.push_back(data);
    }

    fn get_data(&self) -> Vec<ChannelData> {
        // Acquire the lock to prevent data modifications during reading
        let _guard = self.buffer_lock.lock().unwrap();
        
        // Get read buffer and copy its contents
        let mut read_buf = self.read_buffer.lock().unwrap();
        let result: Vec<ChannelData> = read_buf.iter().cloned().collect();
        
        // Clear the read buffer after readings
        read_buf.clear();
        
        result
    }
}

#[tauri::command]
fn connect_serial(
    app_handle: AppHandle,
    port: String,
    baud_rate: u32,
    stop_bits: u8,
    state: State<Arc<SerialState>>,
) -> Result<(), String> {
    let stop_bit_setting = match stop_bits {
        1 => serialport::StopBits::One,
        2 => serialport::StopBits::Two,
        _ => return Err("Invalid stop bits value. Use 1 or 2.".into()),
    };

    let port_result = serialport::new(port.clone(), baud_rate)
        .stop_bits(stop_bit_setting)
        .timeout(Duration::from_millis(100))
        .open();

    let mut port = port_result.map_err(|e| format!("Failed to open port {}: {}", port, e))?;

    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    {
        let mut outbound = state.outbound_tx.lock().unwrap();
        *outbound = Some(tx);
    }

    let state_cloned = state.inner().clone();
    let app_handle_cloned = app_handle.clone();

    std::thread::spawn(move || {
        let mut reader = BufReader::new(port.try_clone().expect("Failed to clone port"));
        let mut line = String::new();
        loop {
            if let Ok(out_msg) = rx.try_recv() {
                let _ = port.write_all(out_msg.as_bytes());
            }

            line.clear();
            match reader.read_line(&mut line) {
                Ok(bytes) if bytes > 0 => {
                    let parts: Vec<&str> = line.trim().split(',').collect();

                    if parts.len() == 9 {
                        let channels_result: Result<Vec<u16>, _> =
                            parts[..8].iter().map(|s| s.parse()).collect();
                        if let Ok(channels) = channels_result {
                            if let Ok(expected_checksum) = parts[8].parse::<u16>() {
                                let sum: u16 = channels.iter().sum();
                                if sum % 256 == expected_checksum {
                                    if let Ok(data_array) = <[u16; 8]>::try_from(channels) {
                                        state_cloned.add_data(data_array.map(|x| x as f32));
                                        // Using emit (with the Emitter trait imported) to forward data to the front end.
                                        let _ = app_handle_cloned.emit("serial_data", data_array.map(|x| x as f32));
                                    }
                                } else {
                                    eprintln!(
                                        "Checksum validation failed for line: {}",
                                        line.trim()
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn send_serial(message: String, state: State<Arc<SerialState>>) -> Result<(), String> {
    if let Some(tx) = state.outbound_tx.lock().unwrap().as_ref() {
        tx.send(message)
            .map_err(|e| format!("Failed to send message: {}", e))
    } else {
        Err("Serial port is not connected.".into())
    }
}

#[tauri::command]
fn get_recent_data(state: State<Arc<SerialState>>) -> Result<Vec<ChannelData>, String> {
    Ok(state.get_data())
}

// Command to start the fake data stream with configuration
#[tauri::command]
fn start_fake_data(
    app_handle: AppHandle,
    config: FakeDataConfig,
    state: State<Arc<SerialState>>,
) -> Result<bool, String> {
    let state_clone = state.inner().clone();
    let app_handle_clone = app_handle.clone();
    
    // If already running, stop it first
    if state.fake_stream_running.load(Ordering::SeqCst) {
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
    
    // Start the stream with the provided configuration
    println!("Starting fake data stream with config: {:?}", config);
    state.fake_stream_running.store(true, Ordering::SeqCst);
    let running_flag = state.fake_stream_running.clone();
    
    // Calculate sleep duration from frequency (in Hz)
    let sleep_ms = (1000.0 / config.frequency).round() as u64;
    
    // Ensure channel count is valid (between 1 and 8)
    let channel_count = config.channel_count.min(8).max(1);
    
    // Store config values for use in the thread
    let min_value = config.min_value;
    let max_value = config.max_value;
    let waveform = config.waveform.clone();

    let handle = thread::spawn(move || {
        let mut rng = rand::thread_rng();
        let mut counter: f64 = 0.0;
        let step = 0.01; // Phase increment per iteration
        
        while running_flag.load(Ordering::SeqCst) {
            // Generate fake data based on the selected waveform
            let mut fake_data: [f32; 8] = [0.0; 8];
            
            for i in 0..channel_count {
                let phase = counter + (i as f64 * 0.2); // Slight phase shift for each channel
                let amplitude = (max_value - min_value) as f64;
                let offset = min_value as f64;
                
                let value = match waveform.as_str() {
                    "sine" => {
                        let sin_val = (phase * 2.0 * std::f64::consts::PI).sin();
                        (sin_val * amplitude / 2.0 + amplitude / 2.0 + offset) as f32
                    },
                    "square" => {
                        let square_val = if (phase % 1.0) < 0.5 { 0.0 } else { 1.0 };
                        (square_val * amplitude + offset) as f32
                    },
                    "triangle" => {
                        let tri_phase = phase % 1.0;
                        let tri_val = if tri_phase < 0.5 {
                            tri_phase * 2.0
                        } else {
                            2.0 - tri_phase * 2.0
                        };
                        (tri_val * amplitude + offset) as f32
                    },
                    "sawtooth" => {
                        let saw_val = phase % 1.0;
                        (saw_val * amplitude + offset) as f32
                    },
                    "random" => rng.gen_range(min_value..=max_value) as f32,
                    _ => rng.gen_range(min_value..=max_value) as f32,
                };
                
                fake_data[i] = value;
            }
            
            // Fill remaining channels with zeros if not all 8 channels are used
            for i in channel_count..8 {
                fake_data[i] = 0.0;
            }

            state_clone.add_data(fake_data);
            let _ = app_handle_clone.emit("serial_data", fake_data);
            thread::sleep(Duration::from_millis(sleep_ms));
            
            counter += step;
            if counter >= 1.0 {
                counter = 0.0;
            }
        }
        println!("Fake data stream thread finished.");
    });

    *state.fake_stream_handle.lock().unwrap() = Some(handle);
    println!("Fake data stream started.");
    
    Ok(true) // Return the new state
}

#[tauri::command]
fn stop_data_acquisition(state: State<Arc<SerialState>>) -> Result<(), String> {
    // Stop fake data stream if running
    if state.fake_stream_running.load(Ordering::SeqCst) {
        println!("Stopping fake data stream...");
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            handle
                .join()
                .map_err(|_| "Failed to join fake stream thread".to_string())?;
        }
        println!("Fake data stream stopped.");
    }
    
    // Close serial connection if exists (by setting the sender to None)
    let mut outbound = state.outbound_tx.lock().unwrap();
    *outbound = None;
    
    // Note: Proper cleanup of other acquisition types (e.g., TCP) would be added here
    
    Ok(())
}

#[tauri::command]
fn get_available_ports() -> Result<Vec<String>, String> {
    match serialport::available_ports() {
        Ok(ports) => {
            let port_names = ports.into_iter().map(|p| p.port_name).collect();
            Ok(port_names)
        }
        Err(e) => Err(format!("Failed to list serial ports: {}", e)),
    }
}

fn main() {
    let serial_state = Arc::new(SerialState::new());

    tauri::Builder::default()
        .manage(serial_state)
        .invoke_handler(tauri::generate_handler![
            connect_serial,
            send_serial,
            get_recent_data,
            get_available_ports,
            start_fake_data,
            stop_data_acquisition
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
