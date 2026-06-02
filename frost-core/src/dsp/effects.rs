use crate::dsp::plugins::AudioPlugin;

// Delay and basic Reverb effects for Send Channels

pub struct Delay {
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_idx: usize,
    sample_rate: f32,
    pub time_ms: f32,
    pub feedback: f32,
    pub mix: f32,
}

impl Delay {
    pub fn new(sample_rate: f32) -> Self {
        let max_delay_samples = (sample_rate * 2.0) as usize; // Max 2 seconds
        Self {
            buffer_l: vec![0.0; max_delay_samples],
            buffer_r: vec![0.0; max_delay_samples],
            write_idx: 0,
            sample_rate,
            time_ms: 300.0,
            feedback: 0.4,
            mix: 0.5,
        }
    }
}

impl AudioPlugin for Delay {
    fn name(&self) -> &'static str { "Frost Delay" }

    fn process(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let delay_samples = ((self.time_ms / 1000.0) * self.sample_rate) as usize;
        let mut read_idx = self.write_idx + self.buffer_l.len() - delay_samples;
        if read_idx >= self.buffer_l.len() {
            read_idx -= self.buffer_l.len();
        }

        let delay_l = self.buffer_l[read_idx];
        let delay_r = self.buffer_r[read_idx];

        // Write to buffer with feedback
        self.buffer_l[self.write_idx] = in_l + delay_l * self.feedback;
        self.buffer_r[self.write_idx] = in_r + delay_r * self.feedback;

        self.write_idx += 1;
        if self.write_idx >= self.buffer_l.len() {
            self.write_idx = 0;
        }

        (
            in_l * (1.0 - self.mix) + delay_l * self.mix,
            in_r * (1.0 - self.mix) + delay_r * self.mix,
        )
    }

    fn set_param(&mut self, id: u32, value: f32) {
        match id {
            0 => self.time_ms = value.clamp(1.0, 2000.0),
            1 => self.feedback = value.clamp(0.0, 0.95),
            2 => self.mix = value.clamp(0.0, 1.0),
            _ => (),
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        match id {
            0 => self.time_ms,
            1 => self.feedback,
            2 => self.mix,
            _ => 0.0,
        }
    }
    
    fn reset(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
        self.write_idx = 0;
    }
}

// Simple algorithmic reverb (Freeverb style simplified via Comb/Allpass filters)
// For demonstration, we'll implement a basic multi-tap delay as a pseudo-reverb
pub struct Reverb {
    delays: Vec<Delay>,
    pub mix: f32,
}

impl Reverb {
    pub fn new(sample_rate: f32) -> Self {
        // Create 4 parallel delays with prime-ish lengths for density
        let mut d1 = Delay::new(sample_rate); d1.time_ms = 29.0; d1.feedback = 0.7; d1.mix = 1.0;
        let mut d2 = Delay::new(sample_rate); d2.time_ms = 43.0; d2.feedback = 0.75; d2.mix = 1.0;
        let mut d3 = Delay::new(sample_rate); d3.time_ms = 61.0; d3.feedback = 0.8; d3.mix = 1.0;
        let mut d4 = Delay::new(sample_rate); d4.time_ms = 83.0; d4.feedback = 0.6; d4.mix = 1.0;

        Self {
            delays: vec![d1, d2, d3, d4],
            mix: 0.5,
        }
    }
}

impl AudioPlugin for Reverb {
    fn name(&self) -> &'static str { "Frost Reverb" }

    fn process(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let mut rev_l = 0.0;
        let mut rev_r = 0.0;

        for delay in self.delays.iter_mut() {
            let (l, r) = delay.process(in_l, in_r);
            rev_l += l;
            rev_r += r;
        }

        // Average output
        rev_l *= 0.25;
        rev_r *= 0.25;

        (
            in_l * (1.0 - self.mix) + rev_l * self.mix,
            in_r * (1.0 - self.mix) + rev_r * self.mix,
        )
    }

    fn set_param(&mut self, id: u32, value: f32) {
        match id {
            0 => self.mix = value.clamp(0.0, 1.0),
            _ => (),
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        match id {
            0 => self.mix,
            _ => 0.0,
        }
    }

    fn reset(&mut self) {
        for d in self.delays.iter_mut() {
            d.reset();
        }
    }
}
