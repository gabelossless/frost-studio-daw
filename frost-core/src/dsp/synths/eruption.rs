use super::wavetable::WavetableOscillator;
use super::sallen_key_filter::SallenKeyFilter;
use crate::dsp::envelope::AdsrEnvelope;

pub struct EruptionSynth {
    osc1: WavetableOscillator,
    osc2: WavetableOscillator,
    hpf: SallenKeyFilter,
    lpf: SallenKeyFilter,
    envelope: AdsrEnvelope,
    #[allow(dead_code)]
    lfo: WavetableOscillator,
}

impl EruptionSynth {
    pub fn new(sample_rate: f32) -> Self {
        let mut osc1 = WavetableOscillator::new(sample_rate, 2048);
        osc1.load_saw();
        let mut osc2 = WavetableOscillator::new(sample_rate, 2048);
        osc2.load_saw();

        let mut lfo = WavetableOscillator::new(sample_rate, 2048);
        lfo.load_sine();
        lfo.set_freq(0.2);

        Self {
            osc1,
            osc2,
            hpf: SallenKeyFilter::new(sample_rate, true),
            lpf: SallenKeyFilter::new(sample_rate, false),
            envelope: AdsrEnvelope::new(sample_rate),
            lfo,
        }
    }

    pub fn trigger_on(&mut self, pitch: u8, _velocity: f32) {
        let freq = 440.0 * 2.0_f32.powf((pitch as f32 - 69.0) / 12.0);
        self.osc1.set_freq(freq);
        self.osc2.set_freq(freq * 1.5); // Fifth interval
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
        
        let mix = (self.osc1.next_sample() + self.osc2.next_sample()) * 0.5;
        
        // Aggressive filter settings
        self.hpf.set_params(100.0 + env * 1000.0, 0.5);
        self.lpf.set_params(1000.0 + env * 8000.0, 0.8);
        
        let hp_out = self.hpf.process(mix);
        let lp_out = self.lpf.process(hp_out);
        
        lp_out * env
    }

    pub fn set_params(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.envelope.set_params(a, d, s, r);
    }
}
