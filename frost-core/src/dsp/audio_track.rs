use serde::{Deserialize, Serialize};
use crate::dsp::audio_loader::GLOBAL_SAMPLE_BANK;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioClip {
    pub id: String,
    pub sample_path: String,
    pub start_tick: u64,
    pub duration_ticks: u64,
    pub offset_samples: usize,
    pub gain: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTrack {
    pub id: usize,
    pub name: String,
    pub clips: Vec<AudioClip>,
    pub muted: bool,
    pub volume: f32,
    pub pan: f32,
    pub channel_id: usize, // Routing to a mixer channel
}

impl AudioTrack {
    pub fn new(id: usize, channel_id: usize) -> Self {
        Self {
            id,
            name: format!("Audio Track {}", id + 1),
            clips: Vec::new(),
            muted: false,
            volume: 0.8,
            pan: 0.0,
            channel_id,
        }
    }

    pub fn get_sample_at(&self, current_tick: u64, bpm: f32, sample_rate: f32) -> (f32, f32) {
        if self.muted { return (0.0, 0.0); }
        
        let mut out_l = 0.0;
        let mut out_r = 0.0;

        for clip in &self.clips {
            if current_tick >= clip.start_tick && current_tick < clip.start_tick + clip.duration_ticks {
                let tick_offset = current_tick - clip.start_tick;
                
                // Convert tick_offset to sample index in the source file
                // seconds = (ticks / 960) * (60 / bpm)
                // samples = seconds * sample_rate
                let seconds = (tick_offset as f64 / 960.0) * (60.0 / bpm as f64);
                let source_sample_idx = (seconds * sample_rate as f64) as usize + clip.offset_samples;

                if let Some(sample) = GLOBAL_SAMPLE_BANK.get_sample(&clip.sample_path) {
                    let data = &sample.data;
                    let channels = sample.channels as usize;
                    let base_idx = source_sample_idx * channels;

                    if base_idx < data.len() {
                        if channels >= 2 {
                            out_l += data[base_idx] * clip.gain;
                            out_r += data[base_idx + 1] * clip.gain;
                        } else {
                            let s = data[base_idx] * clip.gain;
                            out_l += s;
                            out_r += s;
                        }
                    }
                }
            }
        }

        (out_l * self.volume, out_r * self.volume)
    }
}
