/// ADSR (Attack, Decay, Sustain, Release) Envelope Generator
/// Optimized for the audio thread using state machine logic.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct AdsrEnvelope {
    stage: EnvelopeStage,
    value: f32,
    
    // Time parameters in seconds
    attack_time: f32,
    decay_time: f32,
    sustain_level: f32,
    release_time: f32,
    
    // Per-sample increments
    attack_inc: f32,
    decay_inc: f32,
    release_inc: f32,
    
    sample_rate: f32,
}

impl AdsrEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        let mut env = Self {
            stage: EnvelopeStage::Idle,
            value: 0.0,
            attack_time: 0.01,
            decay_time: 0.1,
            sustain_level: 0.5,
            release_time: 0.2,
            attack_inc: 0.0,
            decay_inc: 0.0,
            release_inc: 0.0,
            sample_rate,
        };
        env.recalculate_rates();
        env
    }

    pub fn set_params(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.attack_time = a.max(0.001);
        self.decay_time = d.max(0.001);
        self.sustain_level = s.clamp(0.0, 1.0);
        self.release_time = r.max(0.001);
        self.recalculate_rates();
    }

    fn recalculate_rates(&mut self) {
        self.attack_inc = 1.0 / (self.attack_time * self.sample_rate);
        self.decay_inc = (1.0 - self.sustain_level) / (self.decay_time * self.sample_rate);
        self.release_inc = self.sustain_level / (self.release_time * self.sample_rate);
    }

    pub fn trigger_on(&mut self) {
        self.stage = EnvelopeStage::Attack;
    }

    pub fn trigger_off(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.stage = EnvelopeStage::Release;
        }
    }

    pub fn is_idle(&self) -> bool {
        self.stage == EnvelopeStage::Idle
    }

    #[inline(always)]
    pub fn next_sample(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => 0.0,
            EnvelopeStage::Attack => {
                self.value += self.attack_inc;
                if self.value >= 1.0 {
                    self.value = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
                self.value
            }
            EnvelopeStage::Decay => {
                self.value -= self.decay_inc;
                if self.value <= self.sustain_level {
                    self.value = self.sustain_level;
                    self.stage = EnvelopeStage::Sustain;
                }
                self.value
            }
            EnvelopeStage::Sustain => self.sustain_level,
            EnvelopeStage::Release => {
                self.value -= self.release_inc;
                if self.value <= 0.0 {
                    self.value = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
                self.value
            }
        }
    }
}
