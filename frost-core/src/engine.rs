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
}

impl MixerState {
    pub fn new() -> Self {
        let channels: Vec<MixerChannel> = (0..NUM_CHANNELS).map(MixerChannel::new).collect();
        let synths = (0..NUM_CHANNELS).map(|_| SynthManager::new(44100.0)).collect();
        Self {
            channels,
            master: MasterBus::new(),
            reverb: Reverb::new(44100.0),
            delay: Delay::new(44100.0),
            synths,
            clock: MasterClock::new(44100.0, 120.0),
            playlist: MidiPlaylist::new(),
            next_note_index: 0,
            active_notes: Vec::with_capacity(1024),
            sim_levels: vec![(0.0, 0.0); NUM_CHANNELS],
            synth_bank: SynthBank::new(),
            audio_tracks: (0..4).map(|i| AudioTrack::new(i, i % NUM_CHANNELS)).collect(),
        }
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
