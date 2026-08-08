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

    pub fn get_sample_at(&self, current_tick: u64, bpm: f32, _sample_rate: f32) -> (f32, f32) {
        if self.muted { return (0.0, 0.0); }
        
        let mut out_l = 0.0;
        let mut out_r = 0.0;

        for clip in &self.clips {
            if current_tick >= clip.start_tick && current_tick < clip.start_tick + clip.duration_ticks {
                let tick_offset = current_tick - clip.start_tick;

                // Musical time elapsed inside the clip:
                // seconds = (ticks / 960) * (60 / bpm)
                let seconds = (tick_offset as f64 / 960.0) * (60.0 / bpm as f64);

                if let Some(sample) = GLOBAL_SAMPLE_BANK.get_sample(&clip.sample_path) {
                    let channels = sample.channels as usize;
                    let data = &sample.data;
                    if channels == 0 || data.is_empty() { continue; }
                    let total_frames = data.len() / channels;

                    // Advance through the SOURCE at its own sample rate so clips
                    // play back at the correct pitch regardless of the engine rate.
                    let pos = seconds * sample.sample_rate as f64 + clip.offset_samples as f64;
                    if pos < 0.0 { continue; }
                    let i = pos.floor() as usize;
                    let frac = (pos - i as f64) as f32;
                    if i >= total_frames { continue; }
                    let i1 = (i + 1).min(total_frames - 1);

                    let frame_at = |f: usize| -> (f32, f32) {
                        let base = f * channels;
                        if channels >= 2 {
                            (data[base], data[base + 1])
                        } else {
                            (data[base], data[base])
                        }
                    };

                    let (l0, r0) = frame_at(i);
                    let (l1, r1) = frame_at(i1);
                    let l = l0 + (l1 - l0) * frac;
                    let r = r0 + (r1 - r0) * frac;

                    out_l += l * clip.gain;
                    out_r += r * clip.gain;
                }
            }
        }

        (out_l * self.volume, out_r * self.volume)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::audio_loader::{AudioSample, GLOBAL_SAMPLE_BANK};
    use std::sync::Arc;

    fn ramp_sample(rate: u32) -> AudioSample {
        let frames: Vec<f32> = (0..48000).map(|i| i as f32).collect();
        AudioSample {
            name: "ramp".into(),
            path: format!("test://ramp-{rate}.wav"),
            sample_rate: rate,
            channels: 1,
            duration_seconds: 1.0,
            data: Arc::new(frames),
        }
    }

    #[test]
    fn clip_playback_uses_source_sample_rate_and_interpolates() {
        let sample = ramp_sample(48000);
        GLOBAL_SAMPLE_BANK.samples.write().insert(sample.path.clone(), sample.clone());

        let track = AudioTrack {
            id: 0,
            name: "t".into(),
            clips: vec![AudioClip {
                id: "c".into(),
                sample_path: sample.path.clone(),
                start_tick: 0,
                duration_ticks: 960 * 4,
                offset_samples: 0,
                gain: 1.0,
            }],
            muted: false,
            volume: 1.0,
            pan: 0.0,
            channel_id: 0,
        };

        // At 120 BPM, tick 960 == 1 beat == 0.5 s. A 48 kHz source with a ramp
        // data[i] = i must read back 0.5 * 48000 = 24000 regardless of engine rate.
        for engine_rate in [44100.0f32, 48000.0, 96000.0] {
            let (l, r) = track.get_sample_at(960, 120.0, engine_rate);
            assert!((l - 24000.0).abs() < 1.0, "engine {engine_rate}: l was {l}");
            assert!((r - 24000.0).abs() < 1.0, "engine {engine_rate}: r was {r}");
        }
    }

    #[test]
    fn clip_playback_interpolates_between_frames() {
        let sample = ramp_sample(48000);
        GLOBAL_SAMPLE_BANK.samples.write().insert(sample.path.clone(), sample.clone());

        let track = AudioTrack {
            id: 0,
            name: "t".into(),
            clips: vec![AudioClip {
                id: "c".into(),
                sample_path: sample.path.clone(),
                start_tick: 0,
                duration_ticks: 960 * 4,
                offset_samples: 0,
                gain: 1.0,
            }],
            muted: false,
            volume: 1.0,
            pan: 0.0,
            channel_id: 0,
        };

        // pos = tick / 960 * (60 / bpm) * src_sr.
        // With bpm = 128 the read position lands between frames (3000/128 = 23.4375
        // source frames per tick), so linear interpolation must be exercised.
        let (l, _) = track.get_sample_at(1, 128.0, 48000.0);
        let expected = 3000.0 / 128.0; // 23.4375
        assert!((l - expected).abs() < 1e-3, "interpolated read was {l}, expected {expected}");
        assert!(l > 0.0, "interpolated read should be audible");
    }
}
