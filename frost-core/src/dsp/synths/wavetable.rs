use std::f32::consts::PI;

pub struct WavetableOscillator {
    table: Vec<f32>,
    phase: f32,
    phase_inc: f32,
    sample_rate: f32,
}

impl WavetableOscillator {
    pub fn new(sample_rate: f32, table_size: usize) -> Self {
        Self {
            table: vec![0.0; table_size],
            phase: 0.0,
            phase_inc: 0.0,
            sample_rate,
        }
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.phase_inc = freq / self.sample_rate;
    }

    pub fn load_sine(&mut self) {
        let size = self.table.len();
        for i in 0..size {
            self.table[i] = (2.0 * PI * i as f32 / size as f32).sin();
        }
    }

    pub fn load_saw(&mut self) {
        let size = self.table.len();
        for i in 0..size {
            self.table[i] = 2.0 * (i as f32 / size as f32) - 1.0;
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let table_size = self.table.len() as f32;
        let index = self.phase * table_size;
        let i0 = index as usize;
        let i1 = (i0 + 1) % self.table.len();
        let frac = index - i0 as f32;

        // Linear interpolation
        let sample = self.table[i0] + (self.table[i1] - self.table[i0]) * frac;
        
        self.phase = (self.phase + self.phase_inc).fract();
        sample
    }
}
