use std::sync::Arc;
use serde::{Deserialize, Serialize};
use std::path::{Path};

use crate::dsp::mixer::{MixerChannel, MasterBus};
use crate::dsp::synths::manager::{SynthManager};
use crate::dsp::midi::{MasterClock, MidiPlaylist, NoteEvent};
use crate::dsp::effects::{Delay, Reverb};
use crate::dsp::plugins::AudioPlugin;
use crate::dsp::presets::{SynthBank};
use crate::dsp::audio_track::AudioTrack;

pub const NUM_CHANNELS: usize = 4;

pub struct MixerState {
    pub channels: Vec<MixerChannel>,
    pub master: MasterBus,
    pub reverb: Reverb,
    pub delay: Delay,
    pub synths: Vec<SynthManager>,
    pub clock: MasterClock,
    pub playlist: MidiPlaylist,
    pub next_note_index: usize,
    pub active_notes: Vec<(NoteEvent, u64)>,
    pub sim_levels: Vec<(f32, f32)>,
    pub synth_bank: SynthBank,
    pub audio_tracks: Vec<AudioTrack>,
    pub sample_rate: f32,
}

impl MixerState {
    pub fn new() -> Self {
        Self::with_sample_rate(44100.0)
    }

    pub fn with_sample_rate(sample_rate: f32) -> Self {
        let channels: Vec<MixerChannel> = (0..NUM_CHANNELS).map(|i| MixerChannel::new(i, sample_rate)).collect();
        let synths = (0..NUM_CHANNELS).map(|_| SynthManager::new(sample_rate)).collect();
        Self {
            channels,
            master: MasterBus::new(sample_rate),
            reverb: Reverb::new(sample_rate),
            delay: Delay::new(sample_rate),
            synths,
            clock: MasterClock::new(sample_rate, 120.0),
            playlist: MidiPlaylist::new(),
            next_note_index: 0,
            active_notes: Vec::with_capacity(1024),
            sim_levels: vec![(0.0, 0.0); NUM_CHANNELS],
            synth_bank: SynthBank::new(),
            audio_tracks: (0..4).map(|i| AudioTrack::new(i, i % NUM_CHANNELS)).collect(),
            sample_rate,
        }
    }

    /// Rebuild all sample-rate-dependent DSP at a new rate. Preserves user
    /// state (channel params, synth types, playlist, audio tracks, bank).
    pub fn set_sample_rate(&mut self, new_rate: f32) {
        if (new_rate - self.sample_rate).abs() < 0.5 {
            return;
        }
        let channel_params: Vec<_> = self.channels.iter().map(|c| c.params.clone()).collect();
        let synth_types: Vec<_> = self.synths.iter().map(|s| s.active_type()).collect();
        let mut rebuilt = Self::with_sample_rate(new_rate);
        rebuilt.playlist = std::mem::replace(&mut self.playlist, MidiPlaylist::new());
        rebuilt.audio_tracks = std::mem::replace(&mut self.audio_tracks, Vec::new());
        rebuilt.synth_bank = std::mem::replace(&mut self.synth_bank, SynthBank::new());
        for (i, ch) in rebuilt.channels.iter_mut().enumerate() {
            if let Some(p) = channel_params.get(i) {
                ch.params = p.clone();
                ch.apply_params(p.clone());
            }
        }
        for (i, s) in rebuilt.synths.iter_mut().enumerate() {
            if let Some(t) = synth_types.get(i) {
                s.set_type(*t);
            }
        }
        *self = rebuilt;
    }

    pub fn generate_frame(&mut self) -> (f32, f32) {
        let mut master_l = 0.0f32;
        let mut master_r = 0.0f32;
        let mut sum_send1_l = 0.0f32;
        let mut sum_send1_r = 0.0f32;
        let mut sum_send2_l = 0.0f32;
        let mut sum_send2_r = 0.0f32;

        if let Some(tick) = self.clock.tick() {
            while self.next_note_index < self.playlist.notes.len() {
                let note = &self.playlist.notes[self.next_note_index];
                if note.start_tick <= tick {
                    if let Some(s) = self.synths.get_mut(note.channel_id) {
                        s.note_on(note.pitch, note.velocity);
                    }
                    self.active_notes.push((*note, note.start_tick + note.duration_ticks));
                    self.next_note_index += 1;
                } else {
                    break;
                }
            }

            let synths = &mut self.synths;
            self.active_notes.retain(|(note, end_tick)| {
                if *end_tick <= tick {
                    if let Some(s) = synths.get_mut(note.channel_id) {
                        s.note_off(note.pitch);
                    }
                    false
                } else {
                    true
                }
            });
        }

        for i in 0..NUM_CHANNELS {
            let synth_sample = self.synths[i].process();
            
            let mut track_l = synth_sample;
            let mut track_r = synth_sample;

            // Add audio clips for this channel
            let current_tick = self.clock.current_tick;
            let bpm = self.clock.bpm;
            let sample_rate = self.clock.sample_rate;

            for track in &self.audio_tracks {
                if track.channel_id == i {
                    let (al, ar) = track.get_sample_at(current_tick, bpm, sample_rate);
                    track_l += al;
                    track_r += ar;
                }
            }

            let sidechain_level = if let Some(src_id) = self.channels[i].params.sidechain_source_id {
                if src_id < NUM_CHANNELS {
                    self.channels[src_id].rms_left.max(self.channels[src_id].rms_right)
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let ((out_l, out_r), s1, s2) = self.channels[i].process(track_l, track_r, sidechain_level);
            master_l += out_l;
            master_r += out_r;
            sum_send1_l += s1.0;
            sum_send1_r += s1.1;
            sum_send2_l += s2.0;
            sum_send2_r += s2.1;
        }

        let (rev_l, rev_r) = self.reverb.process(sum_send1_l, sum_send1_r);
        let (del_l, del_r) = self.delay.process(sum_send2_l, sum_send2_r);

        master_l += rev_l + del_l;
        master_r += rev_r + del_r;

        self.master.process(master_l, master_r)
    }
}

pub type SharedMixer = Arc<parking_lot::Mutex<MixerState>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct SampleNode {
    pub name: String,
    pub path: String,
    #[serde(rename = "isDir")]
    pub is_dir: bool,
    pub children: Option<Vec<SampleNode>>,
}

pub fn scan_dir_recursive(path: &Path) -> Option<Vec<SampleNode>> {
    let mut nodes = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name()?.to_string_lossy().to_string();
            let is_dir = path.is_dir();
            
            if is_dir {
                nodes.push(SampleNode {
                    name,
                    path: path.to_string_lossy().to_string(),
                    is_dir: true,
                    children: scan_dir_recursive(&path),
                });
            } else {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "wav" || ext_str == "mp3" || ext_str == "ogg" || ext_str == "flac" {
                        nodes.push(SampleNode {
                            name,
                            path: path.to_string_lossy().to_string(),
                            is_dir: false,
                            children: None,
                        });
                    }
                }
            }
        }
    }
    Some(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::synths::manager::SynthType;

    #[test]
    fn new_defaults_to_44100() {
        let m = MixerState::new();
        assert_eq!(m.sample_rate, 44100.0);
    }

    #[test]
    fn with_sample_rate_uses_provided_rate() {
        let m = MixerState::with_sample_rate(48000.0);
        assert_eq!(m.sample_rate, 48000.0);
        assert_eq!(m.clock.sample_rate, 48000.0);
    }

    #[test]
    fn set_sample_rate_rebuilds_dsp_and_preserves_state() {
        let mut m = MixerState::with_sample_rate(44100.0);
        m.channels[0].params.volume = 0.5;
        m.synths[0].set_type(SynthType::Eruption);

        m.set_sample_rate(96000.0);

        assert_eq!(m.sample_rate, 96000.0);
        assert_eq!(m.clock.sample_rate, 96000.0);
        // User state survives the rebuild
        assert_eq!(m.channels[0].params.volume, 0.5);
        assert_eq!(m.synths[0].active_type(), SynthType::Eruption);
    }

    #[test]
    fn set_sample_rate_ignores_small_deltas() {
        let mut m = MixerState::with_sample_rate(44100.0);
        m.set_sample_rate(44100.0);
        assert_eq!(m.sample_rate, 44100.0);
    }
}
