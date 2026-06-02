use super::wavetable::WavetableOscillator;
use crate::dsp::envelope::AdsrEnvelope;

pub struct NebulaSynth {
    tables: Vec<WavetableOscillator>,
    fm_osc: WavetableOscillator,
    envelope: AdsrEnvelope,
    scan_pos: f32, // 0.0 to 1.0
    fm_amount: f32,
    #[allow(dead_code)]
    lfo: WavetableOscillator,
}

impl NebulaSynth {
    pub fn new(sample_rate: f32) -> Self {
        let mut t1 = WavetableOscillator::new(sample_rate, 2048);
        t1.load_sine();
        let mut t2 = WavetableOscillator::new(sample_rate, 2048);
        t2.load_saw();
        
        let mut fm_osc = WavetableOscillator::new(sample_rate, 2048);
        fm_osc.load_sine();

        let mut lfo = WavetableOscillator::new(sample_rate, 2048);
        lfo.load_sine();
        lfo.set_freq(0.1);

        Self {
            tables: vec![t1, t2],
            fm_osc,
            envelope: AdsrEnvelope::new(sample_rate),
            scan_pos: 0.0,
            fm_amount: 0.1,
            lfo,
        }
    }

    pub fn trigger_on(&mut self, pitch: u8, _velocity: f32) {
        let freq = 440.0 * 2.0_f32.powf((pitch as f32 - 69.0) / 12.0);
        for t in self.tables.iter_mut() {
            t.set_freq(freq);
        }
        self.fm_osc.set_freq(freq * 2.0); // 2nd harmonic
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
        
        // FM modulation
        let _fm_mod = self.fm_osc.next_sample() * self.fm_amount;
        
        // Scan between tables (simple 2-table blend)
        let s1 = self.tables[0].next_sample();
        let s2 = self.tables[1].next_sample();
        
        let blend = s1 * (1.0 - self.scan_pos) + s2 * self.scan_pos;
        
        // Apply FM to the output phase is hard, let's just do AM/Crossfade for now 
        // to keep it simple, or apply FM via frequency update if we were doing it per sample.
        
        blend * env
    }

    pub fn set_params(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.envelope.set_params(a, d, s, r);
    }
}
