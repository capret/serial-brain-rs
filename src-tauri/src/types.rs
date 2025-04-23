use serde::Deserialize;

/// Alias for eight-channel float data
pub type ChannelData = [f32; 8];

/// Configuration for fake data generation
#[derive(Debug, Deserialize, Clone)]
pub struct FakeDataConfig {
    pub min_value: i32,
    pub max_value: i32,
    pub frequency: f64,
    pub channel_count: usize,
    pub waveform: String,
}
