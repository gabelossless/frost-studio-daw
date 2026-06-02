use super::AudioPlugin;

pub struct Compressor {
    threshold: f32, // 0.0 to 1.0 (linear)
    ratio: f32,      // 1.0 to 20.0
    attack: f32,     // seconds
    release: f32,    // seconds
    knee: f32,       // 0.0 to 1.0
    makeup: f32,     // 0.0 to 4.0 (gain)
    
    // Internal state
    envelope: f32,
    sample_rate: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: 0.5,
            ratio: 4.0,
            attack: 0.01,
            release: 0.1,
            knee: 0.1,
            makeup: 1.0,
            envelope: 0.0,
            sample_rate,
        }
    }
}

impl AudioPlugin for Compressor {
    fn name(&self) -> &'static str { "Frost Compressor" }

    fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let input_level = (left.abs() + right.abs()) * 0.5;
        
        // Simple envelope follower
        let attack_coef = (-1.0 / (self.attack * self.sample_rate)).exp();
        let release_coef = (-1.0 / (self.release * self.sample_rate)).exp();
        
        if input_level > self.envelope {
            self.envelope = attack_coef * (self.envelope - input_level) + input_level;
        } else {
            self.envelope = release_coef * (self.envelope - input_level) + input_level;
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
            2 => self.attack = value.clamp(0.0001, 1.0),
            3 => self.release = value.clamp(0.001, 5.0),
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
