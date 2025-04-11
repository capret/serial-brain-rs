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
use serialport;
use serde_json;
use tauri::Emitter;
use tauri::{self, AppHandle, State}; // Import the Emitter trait to bring the `emit` method into scope.

// Define a type alias for the eight channel data.
type ChannelData = [u16; 8];

/// Shared state for our serial connection.
struct SerialState {
    outbound_tx: Mutex<Option<Sender<String>>>,
    buffer: Mutex<VecDeque<ChannelData>>,
    fake_stream_running: Arc<AtomicBool>,
    fake_stream_handle: Mutex<Option<JoinHandle<()>>>,
}

impl SerialState {
    fn new() -> Self {
        Self {
            outbound_tx: Mutex::new(None),
            buffer: Mutex::new(VecDeque::with_capacity(50000)),
            fake_stream_running: Arc::new(AtomicBool::new(false)),
            fake_stream_handle: Mutex::new(None),
        }
    }

    fn add_data(&self, data: ChannelData) {
        let mut buf = self.buffer.lock().unwrap();
        if buf.len() >= 50000 {
            buf.pop_front();
        }
        buf.push_back(data);
    }

    fn get_data(&self, n: usize) -> Vec<ChannelData> {
        let buf = self.buffer.lock().unwrap();
        let mut collected: Vec<ChannelData> = buf.iter().rev().take(n).cloned().collect();
        collected.reverse();
        collected
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
                                        state_cloned.add_data(data_array);
                                        // Using emit (with the Emitter trait imported) to forward data to the front end.
                                        let _ = app_handle_cloned.emit("serial_data", data_array);
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
fn get_recent_data(n: usize, state: State<Arc<SerialState>>) -> Result<Vec<ChannelData>, String> {
    Ok(state.get_data(n))
}

// Command to toggle the fake data stream
#[tauri::command]
fn start_fake_data(
    app_handle: AppHandle,
    state: State<Arc<SerialState>>,
    config: Option<serde_json::Value>,
) -> Result<bool, String> {
    let state_clone = state.inner().clone();
    let app_handle_clone = app_handle.clone();
    let currently_running = state.fake_stream_running.load(Ordering::SeqCst);
    let new_state;

    if currently_running {
        println!("Stopping fake data stream...");
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            handle
                .join()
                .map_err(|_| "Failed to join fake stream thread".to_string())?;
        }
        println!("Fake data stream stopped.");
        new_state = false;
    } else {
        // Start the stream
        println!("Starting fake data stream...");
        state.fake_stream_running.store(true, Ordering::SeqCst);
        let running_flag = state.fake_stream_running.clone();

        // Parse configuration settings - extract values and convert to owned types
        let min_value = config.as_ref().and_then(|c| c.get("minValue").and_then(|v| v.as_i64())).unwrap_or(-10);
        let max_value = config.as_ref().and_then(|c| c.get("maxValue").and_then(|v| v.as_i64())).unwrap_or(10);
        let frequency = config.as_ref().and_then(|c| c.get("frequency").and_then(|v| v.as_f64())).unwrap_or(5.0);
        let channel_count = config.as_ref().and_then(|c| c.get("channelCount").and_then(|v| v.as_i64())).unwrap_or(4) as usize;
        
        // Extract waveform as String (owned type) instead of &str (reference)
        let waveform_str = config.as_ref()
            .and_then(|c| c.get("waveform").and_then(|v| v.as_str()))
            .unwrap_or("random")
            .to_string(); // Convert to owned String

        println!("Fake data settings: min={}, max={}, freq={}, channels={}, waveform={}", 
            min_value, max_value, frequency, channel_count, waveform_str);

        // Calculate sleep duration based on frequency
        let sleep_ms = (1000.0 / frequency) as u64;

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut t = 0.0; // Time variable for waveforms
            let step = 0.1; // Time step

            while running_flag.load(Ordering::SeqCst) {
                // Generate fake data based on the selected waveform
                let mut fake_data = [0u16; 8];

                for i in 0..channel_count.min(8) {
                    let phase_offset = (i as f64) * std::f64::consts::PI / 4.0; // Different phase for each channel
                    let value = match waveform_str.as_str() {
                        "sine" => {
                            let amplitude = (max_value - min_value) as f64 / 2.0;
                            let offset = min_value as f64 + amplitude;
                            (amplitude * ((t + phase_offset) * std::f64::consts::PI * 2.0).sin() + offset) as u16
                        },
                        "square" => {
                            let period = (t + phase_offset) % 1.0;
                            if period < 0.5 { min_value as u16 } else { max_value as u16 }
                        },
                        "triangle" => {
                            let period = (t + phase_offset) % 1.0;
                            let triangle = if period < 0.5 { period * 2.0 } else { 2.0 - period * 2.0 };
                            (min_value as f64 + triangle * (max_value - min_value) as f64) as u16
                        },
                        "sawtooth" => {
                            let period = (t + phase_offset) % 1.0;
                            (min_value as f64 + period * (max_value - min_value) as f64) as u16
                        },
                        _ => { // Random
                            rng.gen_range(min_value as u16..=max_value as u16)
                        }
                    };
                    
                    fake_data[i] = value;
                }

                // Fill any unused channels with 0
                for i in channel_count.min(8)..8 {
                    fake_data[i] = 0;
                }

                state_clone.add_data(fake_data);
                let _ = app_handle_clone.emit("serial_data", fake_data);
                
                // Update time variable
                t += step;
                
                // Sleep based on frequency
                thread::sleep(Duration::from_millis(1));
            }
            println!("Fake data stream thread finished.");
        });

        *state.fake_stream_handle.lock().unwrap() = Some(handle);
        println!("Fake data stream started.");
        new_state = true;
    }
    Ok(new_state) // Return the new state
}

#[tauri::command] // be able to stop fake_data_stream from the front end
fn stop_data_acquisition(state: State<Arc<SerialState>>) -> Result<(), String> {
    // Stop fake data stream if running
    let fake_running = state.fake_stream_running.load(Ordering::SeqCst);
    if fake_running {
        println!("Stopping fake data stream...");
        state.fake_stream_running.store(false, Ordering::SeqCst);
        if let Some(handle) = state.fake_stream_handle.lock().unwrap().take() {
            handle
                .join()
                .map_err(|_| "Failed to join fake stream thread".to_string())?;
        }
        println!("Fake data stream stopped.");
    }
    
    // Close serial connection if active
    let mut outbound_tx = state.outbound_tx.lock().unwrap();
    if outbound_tx.is_some() {
        println!("Closing serial connection...");
        *outbound_tx = None;
        println!("Serial connection closed.");
    }
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
            stop_data_acquisition,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
