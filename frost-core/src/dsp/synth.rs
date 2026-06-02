/// Polyphonic Wavetable Synthesizer for Frost Studio
/// Supports Sine, Square, Saw waves + ADSR envelope.

use super::envelope::{AdsrEnvelope};
use std::f32::consts::PI;

const NUM_VOICES: usize = 8;
const SAMPLE_RATE: f32 = 44100.0;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
}

#[allow(dead_code)]
struct SynthVoice {
    osc_phase: f32,
    osc_phase_inc: f32,
    envelope: AdsrEnvelope,
    active_note: Option<u8>, // MIDI pitch
    waveform: Waveform,
}

impl SynthVoice {
    fn new() -> Self {
        Self {
            osc_phase: 0.0,
            osc_phase_inc: 0.0,
            envelope: AdsrEnvelope::new(SAMPLE_RATE),
            active_note: None,
            waveform: Waveform::Sine,
        }
    }

    fn trigger(&mut self, pitch: u8, _velocity: f32, waveform: Waveform) {
        let freq = 440.0 * 2.0_f32.powf((pitch as f32 - 69.0) / 12.0);
        self.osc_phase_inc = freq / SAMPLE_RATE;
        self.active_note = Some(pitch);
        self.waveform = waveform;
        self.envelope.trigger_on();
    }

    fn release(&mut self) {
        self.envelope.trigger_off();
    }

    fn is_active(&self) -> bool {
        !self.envelope.is_idle()
    }

    #[inline(always)]
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() { return 0.0; }

        let env_gain = self.envelope.next_sample();
        
        let raw_sample = match self.waveform {
            Waveform::Sine => (self.osc_phase * 2.0 * PI).sin(),
            Waveform::Square => if self.osc_phase < 0.5 { 1.0 } else { -1.0 },
            Waveform::Saw => (self.osc_phase * 2.0) - 1.0,
        };

        self.osc_phase = (self.osc_phase + self.osc_phase_inc).fract();
        
        raw_sample * env_gain
    }
}

