use crate::state::AppState;
use crate::types::FakeDataConfig;
use encoding_rs::GBK;
use rand::Rng;
use serialport::{DataBits, Parity, SerialPort, StopBits};
use std::{
    convert::TryInto,
    io::{self, Read},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
        Arc,
    },
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter};

// Packet header and length
const DATA_HEADER: [u8; 4] = [0xAA, 0xFF, 0xF1, 0x20];
const DATA_PACKET_LENGTH: usize = 38;

// Unified reader trait
pub trait DataReader {
    fn setup(&mut self) -> Result<(), String>;
    fn read_data(&mut self) -> Result<Vec<u8>, String>;
    fn close(&mut self);
}

// Serial port binary reader
pub struct SerialBinaryReader {
    port_name: String,
    baud_rate: u32,
    stop_bits: u8,
    parity: String,
    data_bits: u8,
    rx: Receiver<String>,
    port: Option<Box<dyn SerialPort>>,
}

impl SerialBinaryReader {
    pub fn new(
        port_name: String,
        baud_rate: u32,
        stop_bits: u8,
        parity: String,
        data_bits: u8,
        rx: Receiver<String>,
    ) -> Self {
        Self {
            port_name,
            baud_rate,
            stop_bits,
            parity,
            data_bits,
            rx,
            port: None,
        }
    }
}

impl DataReader for SerialBinaryReader {
    fn setup(&mut self) -> Result<(), String> {
        let p = serialport::new(self.port_name.clone(), self.baud_rate)
            .stop_bits(match self.stop_bits {
                1 => StopBits::One,
                2 => StopBits::Two,
                _ => StopBits::One,
            })
            .parity(match self.parity.as_str() {
                "none" => Parity::None,
                "odd" => Parity::Odd,
                "even" => Parity::Even,
                _ => Parity::None,
            })
            .data_bits(match self.data_bits {
                5 => DataBits::Five,
                6 => DataBits::Six,
                7 => DataBits::Seven,
                8 => DataBits::Eight,
                _ => DataBits::Eight,
            })
            .timeout(Duration::from_millis(100))
            .open()
            .map_err(|e| format!("Failed to open port {}: {}", self.port_name, e))?;
        self.port = Some(p);
        println!("Port opened: {}", self.port_name);
        Ok(())
    }
    fn read_data(&mut self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        if let Ok(msg) = self.rx.try_recv() {
            if let Some(p) = self.port.as_mut() {
                let _ = p.write_all(msg.as_bytes());
            }
        }
        if let Some(p) = self.port.as_mut() {
            let mut buf = [0u8; 1024];
            match p.read(&mut buf) {
                Ok(n) if n > 0 => out.extend_from_slice(&buf[..n]),
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
                Err(e) => return Err(format!("Serial read error: {}", e)),
                _ => {}
            }
        }
        Ok(out)
    }
    fn close(&mut self) {
        let _ = self.port.take();
    }
}

// TCP socket binary reader
pub struct SocketBinaryReader {
    host: String,
    port: u16,
    listener: Option<TcpListener>,
    stream: Option<TcpStream>,
    accepted: bool,
    app: Option<AppHandle>,
}

impl SocketBinaryReader {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            listener: None,
            stream: None,
            accepted: false,
            app: None,
        }
    }
    
    // Set the app handle for emitting events
    pub fn with_app_handle(mut self, app_handle: AppHandle) -> Self {
        self.app = Some(app_handle);
        self
    }
}

impl DataReader for SocketBinaryReader {
    fn setup(&mut self) -> Result<(), String> {
        let addr = format!("{}:{}", self.host, self.port);
        
        // Just bind directly to the address
        // Note: In standard library we don't have direct access to SO_REUSEADDR
        // but we're still moving the accept logic to read_data which should help
        let listener = TcpListener::bind(&addr)
            .map_err(|e| format!("Failed to bind {}:{} - {}", self.host, self.port, e))?;
            
        // Set non-blocking mode so accept() won't block
        listener.set_nonblocking(true)
            .map_err(|e| format!("Failed to set non-blocking mode: {}", e))?;
            
        self.listener = Some(listener);
        let bind_msg = format!("[SOCKET] Bound to {}:{} and listening", self.host, self.port);
        println!("{}", bind_msg);
        
        // Don't wait for client in setup - we'll accept in read_data
        self.accepted = false;
        Ok(())
    }
    
    fn read_data(&mut self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        
        // If we don't have a client connection yet, try to accept one
        if !self.accepted {
            if let Some(listener) = &self.listener {
                match listener.accept() {
                    Ok((stream, addr)) => {
                        let addr_str = format!("[SOCKET] Connected from {}", addr);
                        println!("{}", addr_str);
                        // Emit socket status event to frontend
                        if let Some(app) = &self.app {
                            let _ = app.emit("socket_status", addr_str.clone());
                        }
                        // Set non-blocking mode
                        stream.set_nonblocking(true)
                            .map_err(|e| format!("Failed to set client non-blocking: {}", e))?;
                        // Store the client stream
                        self.stream = Some(stream);
                        self.accepted = true;
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // No connection available yet, not an error
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        println!("[SOCKET] Accept error: {}", e);
                        // Continue and try again next time
                    }
                }
            }
        }
        
        // If we have a client connection, try to read data
        if let Some(s) = self.stream.as_mut() {
            let mut buf = [0u8; 1024];
            match s.read(&mut buf) {
                Ok(n) if n > 0 => out.extend_from_slice(&buf[..n]),
                Ok(0) => {
                    // Connection closed by peer
                    let disconnect_msg = "[SOCKET] Client disconnected";
                    println!("{}", disconnect_msg);
                    if let Some(app) = &self.app {
                        let _ = app.emit("socket_status", disconnect_msg.to_string());
                    }
                    self.stream = None;
                    self.accepted = false;
                }
                Ok(_) => { /* Read 0 bytes, nothing to do */ }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No data available, not an error
                }
                Err(e) => {
                    println!("[SOCKET] Read error: {}", e);
                    // Reset connection on error
                    self.stream = None;
                    self.accepted = false;
                    return Err(format!("Socket read error: {}", e));
                }
            }
        }
        
        Ok(out)
    }
    
    fn close(&mut self) {
        let _ = self.stream.take();
        let _ = self.listener.take();
        self.accepted = false;
    }
}

// Fake data reader
pub struct FakeBinaryReader {
    config: FakeDataConfig,
    t: f64,
}

impl FakeBinaryReader {
    pub fn new(config: FakeDataConfig) -> Self {
        Self { config, t: 0.0 }
    }
}

impl DataReader for FakeBinaryReader {
    fn setup(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn read_data(&mut self) -> Result<Vec<u8>, String> {
        let mut packet = Vec::with_capacity(DATA_PACKET_LENGTH);
        packet.extend_from_slice(&DATA_HEADER);
        for i in 0..self.config.channel_count.min(8).max(1) {
            let phase = self.t + (i as f64 * 0.2);
            let amplitude = (self.config.max_value - self.config.min_value) as f64;
            let offset = self.config.min_value as f64;
            let value = match self.config.waveform.as_str() {
                "sine" => {
                    let raw_value = (phase * 2.0 * std::f64::consts::PI).sin() * amplitude / 2.0
                        + amplitude / 2.0
                        + offset;
                    // Apply inverse operation to make it consistent with real data processing
                    (raw_value / (0.5364 / 12.0)) as i32
                }
                "square" => {
                    let sq = if (phase % 1.0) < 0.5 { 0.0 } else { 1.0 };
                    let raw_value = sq * amplitude + offset;
                    // Apply inverse operation to make it consistent with real data processing
                    (raw_value / (0.5364 / 12.0)) as i32
                }
                "triangle" => {
                    let tri_phase = phase % 1.0;
                    let tri = if tri_phase < 0.5 {
                        tri_phase * 2.0
                    } else {
                        2.0 - tri_phase * 2.0
                    };
                    let raw_value = tri * amplitude + offset;
                    // Apply inverse operation to make it consistent with real data processing
                    (raw_value / (0.5364 / 12.0)) as i32
                }
                "sawtooth" => {
                    let st = phase % 1.0;
                    let raw_value = st * amplitude + offset;
                    // Apply inverse operation to make it consistent with real data processing
                    (raw_value / (0.5364 / 12.0)) as i32
                }
                "random" => {
                    let raw_value = rand::thread_rng().gen_range(self.config.min_value..=self.config.max_value) as f64;
                    // Apply inverse operation to make it consistent with real data processing
                    (raw_value / (0.5364 / 12.0)) as i32
                }
                _ => {
                    let raw_value = rand::thread_rng().gen_range(self.config.min_value..=self.config.max_value) as f64;
                    // Apply inverse operation to make it consistent with real data processing
                    (raw_value / (0.5364 / 12.0)) as i32
                },
            };
            packet.extend_from_slice(&value.to_le_bytes());
        }
        // pad zeros
        for _ in self.config.channel_count..8 {
            packet.extend_from_slice(&0i32.to_le_bytes());
        }
        let (sc1, sc2) = compute_checksum(&packet);
        packet.push(sc1);
        packet.push(sc2);
        self.t += 0.001;
        thread::sleep(Duration::from_millis(
            (1000.0 / self.config.frequency).round() as u64,
        ));
        Ok(packet)
    }
    fn close(&mut self) {}
}

// Compute two checksums like Python version
fn compute_checksum(data: &[u8]) -> (u8, u8) {
    let sum1: u16 = data[..36].iter().map(|b| *b as u16).sum();
    let sc1 = (sum1 % 256) as u8;
    let mut prefix_acc: u16 = 0;
    let mut sum2: u16 = 0;
    for &b in &data[..36] {
        prefix_acc = (prefix_acc + b as u16) % 256;
        sum2 = (sum2 + prefix_acc) % 256;
    }
    (sc1, sum2 as u8)
}

// Parse buffer and emit data
fn process_buffer(buffer: &mut Vec<u8>, state: &AppState, app: &AppHandle) {
    let mut info_bytes = Vec::new();
    let header_len = DATA_HEADER.len();
    let mut i = 0;
    while i + header_len <= buffer.len() {
        if &buffer[i..i + header_len] == &DATA_HEADER {
            if buffer.len() - i < DATA_PACKET_LENGTH {
                break; // incomplete packet
            }
            let packet = buffer[i..i + DATA_PACKET_LENGTH].to_vec();
            let (c1, c2) = compute_checksum(&packet);
            if packet[36] == c1 && packet[37] == c2 {
                let mut data = [0f32; 8];
                for j in 0..8 {
                    let idx = 4 + j * 4;
                    let v = i32::from_le_bytes(packet[idx..idx + 4].try_into().unwrap());
                    // Convert raw value to real voltage using the formula: raw_value * 0.5364 / 12
                    data[j] = (v as f32) * 0.5364 / 12.0; // hard code for real unit calculation
                }
                // Update both buffer and signal quality with the new data
                state.buffer.add_data(data);
                state.signal_quality.add_data(data);
                state.recording.add_data(data);
                let _ = app.emit("serial_data", data);
            }
            i += DATA_PACKET_LENGTH;
        } else {
            info_bytes.push(buffer[i]);
            i += 1;
        }
    }
    // Remove processed bytes, keep leftover in buffer
    buffer.drain(..i);
    // Emit collected invalid data once
    if !info_bytes.is_empty() {
        let (decoded, _, _) = GBK.decode(&info_bytes);
        let _ = app.emit("serial_info", decoded.into_owned());
    }
}

// Main loop for any reader
pub fn reader_loop<R: DataReader + Send + 'static>(
    mut rd: R,
    running: Arc<AtomicBool>,
    state: Arc<AppState>,
    app: AppHandle,
) {
    let app_clone = app.clone();
    match rd.setup() {
        Ok(_) => {
            let msg = "[READER-LOOP] Setup successful";
            println!("{}", msg);
            let _ = app_clone.emit("socket_status", msg.to_string());
        },
        Err(e) => {
            let error_msg = format!("[READER-LOOP] Setup failed: {}", e);
            println!("{}", error_msg);
            let _ = app_clone.emit("socket_status", error_msg);
            return;
        }
    }
    let mut buf = Vec::new();
    while running.load(Ordering::SeqCst) {
        match rd.read_data() {
            Ok(data) => {
                if !data.is_empty() {
                    buf.extend(data);
                    process_buffer(&mut buf, &state, &app);
                }
            }
            Err(_) => break,
        }
    }
    rd.close();
}
