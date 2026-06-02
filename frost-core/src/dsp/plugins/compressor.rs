use super::AudioPlugin;

pub struct Compressor {
    threshold: f32,
    ratio: f32,
    attack: f32,
    release: f32,
    makeup: f32,
    
    // Pre-computed coefficients
    attack_coef: f32,
    release_coef: f32,
    
    // Internal state
    envelope: f32,
    sample_rate: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        let mut c = Self {
            threshold: 0.5,
            ratio: 4.0,
            attack: 0.01,
            release: 0.1,
            makeup: 1.0,
            attack_coef: 0.0,
            release_coef: 0.0,
            envelope: 0.0,
            sample_rate,
        };
        c.update_coefficients();
        c
    }

    fn update_coefficients(&mut self) {
        self.attack_coef = (-1.0 / (self.attack * self.sample_rate)).exp();
        self.release_coef = (-1.0 / (self.release * self.sample_rate)).exp();
    }
}

impl AudioPlugin for Compressor {
    fn name(&self) -> &'static str { "Frost Compressor" }

    fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let input_level = (left.abs() + right.abs()) * 0.5;
        
        if input_level > self.envelope {
            self.envelope = self.attack_coef * (self.envelope - input_level) + input_level;
        } else {
            self.envelope = self.release_coef * (self.envelope - input_level) + input_level;
        }

        // Compression logic (simplified VCA style)
        let mut gain_reduction = 1.0;
        if self.envelope > self.threshold {
            let over_db = 20.0 * (self.envelope / self.threshold).log10();
            let reduced_db = over_db / self.ratio;
            gain_reduction = 10.0f32.powf((reduced_db - over_db) / 20.0);
        }

        let out_l = left * gain_reduction * self.makeup;
        let out_r = right * gain_reduction * self.makeup;
        
        (out_l, out_r)
    }

    fn set_param(&mut self, id: u32, value: f32) {
        match id {
            0 => self.threshold = value.clamp(0.001, 1.0),
            1 => self.ratio = value.clamp(1.0, 50.0),
            2 => { self.attack = value.clamp(0.0001, 1.0); self.update_coefficients(); },
            3 => { self.release = value.clamp(0.001, 5.0); self.update_coefficients(); },
            4 => self.makeup = value.clamp(0.0, 10.0),
            _ => (),
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        match id {
            0 => self.threshold,
            1 => self.ratio,
            2 => self.attack,
            3 => self.release,
            4 => self.makeup,
            _ => 0.0,
        }
    }

    fn reset(&mut self) {
        self.envelope = 0.0;
    }
}
