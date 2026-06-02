use super::summit::SummitSynth;
use super::eruption::EruptionSynth;
use super::nebula::NebulaSynth;
use super::sampler::SamplerSynth;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SynthType {
    Summit,   // Moog
    Eruption, // Korg
    Nebula,   // Arturia/Modern
    Sampler,  // Beatmaker
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SynthParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

pub struct SynthManager {
    active_type: SynthType,
    summit: SummitSynth,
    eruption: EruptionSynth,
    nebula: NebulaSynth,
    sampler: SamplerSynth,
    active_notes: Vec<u8>, // Voice stack for note-off tracking
}

impl SynthManager {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            active_type: SynthType::Summit,
            summit: SummitSynth::new(sample_rate),
            eruption: EruptionSynth::new(sample_rate),
            nebula: NebulaSynth::new(sample_rate),
            sampler: SamplerSynth::new(sample_rate),
            active_notes: Vec::with_capacity(16),
        }
    }

    pub fn set_type(&mut self, synth_type: SynthType) {
        self.active_type = synth_type;
    }

    pub fn note_on(&mut self, pitch: u8, velocity: f32) {
        // Push to voice stack (last-note priority)
        if let Some(pos) = self.active_notes.iter().position(|&n| n == pitch) {
            self.active_notes.remove(pos);
        }
        self.active_notes.push(pitch);
        match self.active_type {
            SynthType::Summit => self.summit.trigger_on(pitch, velocity),
            SynthType::Eruption => self.eruption.trigger_on(pitch, velocity),
            SynthType::Nebula => self.nebula.trigger_on(pitch, velocity),
            SynthType::Sampler => self.sampler.trigger_on(pitch, velocity),
        }
    }

    pub fn note_off(&mut self, pitch: u8) {
        // Remove from voice stack; only release if no notes remain
        if let Some(pos) = self.active_notes.iter().position(|&n| n == pitch) {
            self.active_notes.remove(pos);
        }
        if !self.active_notes.is_empty() {
            return;
        }
        match self.active_type {
            SynthType::Summit => self.summit.trigger_off(),
            SynthType::Eruption => self.eruption.trigger_off(),
            SynthType::Nebula => self.nebula.trigger_off(),
            SynthType::Sampler => self.sampler.trigger_off(),
        }
    }

    pub fn process(&mut self) -> f32 {
        match self.active_type {
            SynthType::Summit => self.summit.next_sample(),
            SynthType::Eruption => self.eruption.next_sample(),
            SynthType::Nebula => self.nebula.next_sample(),
            SynthType::Sampler => self.sampler.next_sample(),
        }
    }

    pub fn set_params(&mut self, p: SynthParams) {
        self.summit.set_params(p.attack, p.decay, p.sustain, p.release);
        self.eruption.set_params(p.attack, p.decay, p.sustain, p.release);
        self.nebula.set_params(p.attack, p.decay, p.sustain, p.release);
        self.sampler.set_params(p.attack, p.decay, p.sustain, p.release);
    }

    pub fn set_sampler_sample(&mut self, path: String) {
        self.sampler.set_sample(path);
    }
}
