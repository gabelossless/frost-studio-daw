#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
}

