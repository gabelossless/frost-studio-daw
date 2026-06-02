use super::wavetable::WavetableOscillator;
use super::ladder_filter::LadderFilter;
use crate::dsp::envelope::AdsrEnvelope;

pub struct SummitSynth {
    osc1: WavetableOscillator,
    osc2: WavetableOscillator,
    filter: LadderFilter,
    envelope: AdsrEnvelope,
    lfo: WavetableOscillator,
}

impl SummitSynth {
    pub fn new(sample_rate: f32) -> Self {
        let mut osc1 = WavetableOscillator::new(sample_rate, 2048);
        osc1.load_saw();
        let mut osc2 = WavetableOscillator::new(sample_rate, 2048);
        osc2.load_saw();
        let mut lfo = WavetableOscillator::new(sample_rate, 2048);
        lfo.load_sine();
        lfo.set_freq(0.5);

        Self {
            osc1,
            osc2,
            filter: LadderFilter::new(sample_rate),
            envelope: AdsrEnvelope::new(sample_rate),
            lfo,
        }
    }

    pub fn trigger_on(&mut self, pitch: u8, _velocity: f32) {
        let freq = 440.0 * 2.0_f32.powf((pitch as f32 - 69.0) / 12.0);
        self.osc1.set_freq(freq);
        self.osc2.set_freq(freq * 1.002); // Slight detune
        self.envelope.trigger_on();
    }

    pub fn trigger_off(&mut self) {
        self.envelope.trigger_off();
    }

    pub fn is_active(&self) -> bool {
        !self.envelope.is_idle()
    }

    pub fn next_sample(&mut self) -> f32 {
        if !self.is_active() { return 0.0; }

        let env = self.envelope.next_sample();
        let lfo_val = self.lfo.next_sample() * 0.1;
        
        let mix = (self.osc1.next_sample() + self.osc2.next_sample()) * 0.5;
        
        // Filter cutoff modulation via envelope and LFO
        let cutoff = (env * 5000.0 + 200.0) * (1.0 + lfo_val);
        self.filter.set_params(cutoff.clamp(20.0, 20000.0), 0.4);
        
        let filtered = self.filter.process(mix);
        filtered * env
    }

    pub fn set_params(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.envelope.set_params(a, d, s, r);
    }
}
