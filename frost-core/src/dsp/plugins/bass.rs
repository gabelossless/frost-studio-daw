use super::AudioPlugin;
use crate::dsp::synths::wavetable::WavetableOscillator;
use crate::dsp::synths::ladder_filter::LadderFilter;
use crate::dsp::envelope::AdsrEnvelope;

pub struct BassSynthPlugin {
    osc: WavetableOscillator,
    sub_osc: WavetableOscillator,
    filter: LadderFilter,
    env: AdsrEnvelope,
    active: bool,
    note_freq: f32,
}

impl BassSynthPlugin {
    pub fn new(sample_rate: f32) -> Self {
        let mut osc = WavetableOscillator::new(sample_rate, 2048);
        osc.load_saw();
        let mut sub = WavetableOscillator::new(sample_rate, 2048);
        sub.load_sine();
        
        Self {
            osc,
            sub_osc: sub,
            filter: LadderFilter::new(sample_rate),
            env: AdsrEnvelope::new(sample_rate),
            active: false,
            note_freq: 440.0,
        }
    }
}

impl AudioPlugin for BassSynthPlugin {
    fn name(&self) -> &'static str { "Frost Bass Processor" }

    fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // This is a "Synth-as-Plugin" where it can layer with the input or just play
        // For simplicity here, let's make it a sub-harmonic enhancer or solo bass
        
        let env_val = self.env.next_sample();
        let sig = (self.osc.next_sample() * 0.5 + self.sub_osc.next_sample() * 0.8) * env_val;
        let filtered = self.filter.process(sig);
        
        // Mix with input (as a parallel processor)
        let out_l = left + filtered;
        let out_r = right + filtered;
        
        (out_l, out_r)
    }

    fn set_param(&mut self, id: u32, value: f32) {
        match id {
            0 => { // Pitch / Freq
                self.note_freq = value;
                self.osc.set_freq(value);
                self.sub_osc.set_freq(value * 0.5);
            }
            1 => { // Filter Cutoff
                self.filter.set_params(value.clamp(20.0, 5000.0), 0.5);
            }
            2 => { // Trigger
                if value > 0.5 && !self.active {
                    self.env.trigger_on();
                    self.active = true;
                } else if value <= 0.5 && self.active {
                    self.env.trigger_off();
                    self.active = false;
                }
            }
            _ => (),
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        match id {
            0 => self.note_freq,
            _ => 0.0,
        }
    }

    fn reset(&mut self) {
        self.env.trigger_off();
    }
}
