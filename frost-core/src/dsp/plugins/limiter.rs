use super::AudioPlugin;

pub struct Limiter {
    threshold: f32,
    ceiling: f32,
    attack: f32,
    release: f32,
    
    // State
    envelope: f32,
    sample_rate: f32,
}

impl Limiter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: 1.0,
            ceiling: 0.95,
            attack: 0.001,
            release: 0.05,
            envelope: 0.0,
            sample_rate,
        }
    }
}

impl AudioPlugin for Limiter {
    fn name(&self) -> &'static str { "Frost Brickwall Limiter" }

    fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let input_peak = left.abs().max(right.abs());
        
        let attack_coef = (-1.0 / (self.attack * self.sample_rate)).exp();
        let release_coef = (-1.0 / (self.release * self.sample_rate)).exp();
        
        if input_peak > self.envelope {
            self.envelope = attack_coef * (self.envelope - input_peak) + input_peak;
        } else {
            self.envelope = release_coef * (self.envelope - input_peak) + input_peak;
        }

        let mut gain = 1.0;
        if self.envelope > self.threshold {
            gain = self.threshold / self.envelope;
        }
        
        let out_l = (left * gain).clamp(-self.ceiling, self.ceiling);
        let out_r = (right * gain).clamp(-self.ceiling, self.ceiling);
        
        (out_l, out_r)
    }

    fn set_param(&mut self, id: u32, value: f32) {
        match id {
            0 => self.threshold = value.clamp(0.01, 1.0),
            1 => self.ceiling = value.clamp(0.5, 1.0),
            2 => self.attack = value.clamp(0.0001, 0.1),
            3 => self.release = value.clamp(0.01, 2.0),
            _ => (),
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        match id {
            0 => self.threshold,
            1 => self.ceiling,
            2 => self.attack,
            3 => self.release,
            _ => 0.0,
        }
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }
}
