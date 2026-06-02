use super::AudioPlugin;

pub struct Delay {
    buffer_l: Vec<f32>,
    buffer_r: Vec<f32>,
    write_pos: usize,
    sample_rate: f32,
    
    // Params
    time_sec: f32,
    feedback: f32,
    mix: f32,
}

impl Delay {
    pub fn new(sample_rate: f32) -> Self {
        let max_delay = (sample_rate * 2.0) as usize; // 2 seconds max
        Self {
            buffer_l: vec![0.0; max_delay],
            buffer_r: vec![0.0; max_delay],
            write_pos: 0,
            sample_rate,
            time_sec: 0.5,
            feedback: 0.3,
            mix: 0.5,
        }
    }
}

impl AudioPlugin for Delay {
    fn name(&self) -> &'static str { "Frost Echo Delay" }

    fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let delay_samples = (self.time_sec * self.sample_rate) as usize;
        let buf_len = self.buffer_l.len();
        let read_pos = (self.write_pos + buf_len - delay_samples) % buf_len;

        let delayed_l = self.buffer_l[read_pos];
        let delayed_r = self.buffer_r[read_pos];

        self.buffer_l[self.write_pos] = left + delayed_l * self.feedback;
        self.buffer_r[self.write_pos] = right + delayed_r * self.feedback;

        self.write_pos = (self.write_pos + 1) % buf_len;

        let out_l = left * (1.0 - self.mix) + delayed_l * self.mix;
        let out_r = right * (1.0 - self.mix) + delayed_r * self.mix;

        (out_l, out_r)
    }

    fn set_param(&mut self, id: u32, value: f32) {
        match id {
            0 => self.time_sec = value.clamp(0.01, 2.0),
            1 => self.feedback = value.clamp(0.0, 0.95),
            2 => self.mix = value.clamp(0.0, 1.0),
            _ => (),
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        match id {
            0 => self.time_sec,
            1 => self.feedback,
            2 => self.mix,
            _ => 0.0,
        }
    }

    fn reset(&mut self) {
        self.buffer_l.fill(0.0);
        self.buffer_r.fill(0.0);
    }
}
