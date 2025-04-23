use std::{
    thread,
    time::Duration,
    sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc::Receiver},
    io::{self, Read},
    net::{TcpListener, TcpStream},
    convert::TryInto,
};
use tauri::{AppHandle, Emitter};
use rand::Rng;
use serialport::{SerialPort, StopBits, Parity, DataBits};
use crate::state::SerialState;
use crate::types::FakeDataConfig;
use encoding_rs::GBK;

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
        Self { port_name, baud_rate, stop_bits, parity, data_bits, rx, port: None }
    }
}

impl DataReader for SerialBinaryReader {
    fn setup(&mut self) -> Result<(), String> {
        let p = serialport::new(self.port_name.clone(), self.baud_rate)
            .stop_bits(match self.stop_bits { 1 => StopBits::One, 2 => StopBits::Two, _ => StopBits::One })
            .parity(match self.parity.as_str() { "none" => Parity::None, "odd" => Parity::Odd, "even" => Parity::Even, _ => Parity::None })
            .data_bits(match self.data_bits { 5 => DataBits::Five, 6 => DataBits::Six, 7 => DataBits::Seven, 8 => DataBits::Eight, _ => DataBits::Eight })
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
}

impl SocketBinaryReader {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port, listener: None, stream: None }
    }
}

impl DataReader for SocketBinaryReader {
    fn setup(&mut self) -> Result<(), String> {
        let l = TcpListener::bind((&self.host[..], self.port))
            .map_err(|e| format!("Failed to bind {}:{} - {}", self.host, self.port, e))?;
        l.set_nonblocking(true).ok();
        self.listener = Some(l);
        // accept one client
        for stream in self.listener.as_ref().unwrap().incoming() {
            match stream {
                Ok(s) => { self.stream = Some(s); break; }
                Err(_) => { thread::sleep(Duration::from_millis(10)); }
            }
        }
        Ok(())
    }
    fn read_data(&mut self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        if let Some(s) = self.stream.as_mut() {
            let mut buf = [0u8; 1024];
            match s.read(&mut buf) {
                Ok(n) if n > 0 => out.extend_from_slice(&buf[..n]),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(format!("Socket read error: {}", e)),
                _ => {}
            }
        }
        Ok(out)
    }
    fn close(&mut self) {
        let _ = self.stream.take();
        let _ = self.listener.take();
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
    fn setup(&mut self) -> Result<(), String> { Ok(()) }
    fn read_data(&mut self) -> Result<Vec<u8>, String> {
        let mut packet = Vec::with_capacity(DATA_PACKET_LENGTH);
        packet.extend_from_slice(&DATA_HEADER);
        for i in 0..self.config.channel_count.min(8).max(1) {
            let phase = self.t + (i as f64 * 0.2);
            let amplitude = (self.config.max_value - self.config.min_value) as f64;
            let offset = self.config.min_value as f64;
            let value = match self.config.waveform.as_str() {
                "sine" => ((phase * 2.0 * std::f64::consts::PI).sin() * amplitude / 2.0 + amplitude / 2.0 + offset) as i32,
                "square" => {
                    let sq = if (phase % 1.0) < 0.5 { 0.0 } else { 1.0 };
                    (sq * amplitude + offset) as i32
                },
                "triangle" => {
                    let tri_phase = phase % 1.0;
                    let tri = if tri_phase < 0.5 {
                        tri_phase * 2.0
                    } else {
                        2.0 - tri_phase * 2.0
                    };
                    (tri * amplitude + offset) as i32
                },
                "sawtooth" => {
                    let st = phase % 1.0;
                    (st * amplitude + offset) as i32
                },
                "random" => rand::thread_rng().gen_range(self.config.min_value..=self.config.max_value),
                _ => rand::thread_rng().gen_range(self.config.min_value..=self.config.max_value),
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
        thread::sleep(Duration::from_millis((1000.0/self.config.frequency).round() as u64));
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
fn process_buffer(buffer: &mut Vec<u8>, state: &SerialState, app: &AppHandle) {
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
                    data[j] = v as f32;
                }
                state.add_data(data);
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
pub fn reader_loop<R: DataReader + Send + 'static>(mut rd: R, running: Arc<AtomicBool>, state: Arc<SerialState>, app: AppHandle) {
    if rd.setup().is_err() {
        return;
    }
    let mut buf = Vec::new();
    while running.load(Ordering::SeqCst) {
        match rd.read_data() {
            Ok(data) => {
                if !data.is_empty() {
                    buf.extend(data);
                    process_buffer(&mut buf, &state, &app);
                }
            },
            Err(_) => break,
        }
    }
    rd.close();
}
