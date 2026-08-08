use super::AudioPlugin;

pub struct Reverb {
    // Array of Comb Filters for the reverb tail
    combs_l: Vec<CombFilter>,
    combs_r: Vec<CombFilter>,
    allpasses_l: Vec<AllPassFilter>,
    allpasses_r: Vec<AllPassFilter>,
    
    // Params
    room_size: f32,
    damping: f32,
    mix: f32,
}

struct CombFilter {
    buffer: Vec<f32>,
    pos: usize,
    feedback: f32,
    last: f32,
}

impl CombFilter {
    fn new(size: usize) -> Self {
        Self { buffer: vec![0.0; size], pos: 0, feedback: 0.5, last: 0.0 }
    }
    fn process(&mut self, input: f32, damping: f32) -> f32 {
        let output = self.buffer[self.pos];
        self.last = output * (1.0 - damping) + self.last * damping;
        self.buffer[self.pos] = input + self.last * self.feedback;
        self.pos = (self.pos + 1) % self.buffer.len();
        output
    }
}

struct AllPassFilter {
    buffer: Vec<f32>,
    pos: usize,
}

impl AllPassFilter {
    fn new(size: usize) -> Self {
        Self { buffer: vec![0.0; size], pos: 0 }
    }
    fn process(&mut self, input: f32) -> f32 {
        let buf_val = self.buffer[self.pos];
        let output = -input + buf_val;
        self.buffer[self.pos] = input + buf_val * 0.5;
        self.pos = (self.pos + 1) % self.buffer.len();
        output
    }
}

impl Reverb {
    pub fn new(_sample_rate: f32) -> Self {
        // Freeverb-ish constants
        let mut slf = Self {
            combs_l: vec![CombFilter::new(1116), CombFilter::new(1188), CombFilter::new(1277), CombFilter::new(1356)],
            combs_r: vec![CombFilter::new(1116+23), CombFilter::new(1188+23), CombFilter::new(1277+23), CombFilter::new(1356+23)],
            allpasses_l: vec![AllPassFilter::new(556), AllPassFilter::new(441)],
            allpasses_r: vec![AllPassFilter::new(556+23), AllPassFilter::new(441+23)],
            room_size: 0.7,
            damping: 0.2,
            mix: 0.3,
        };
        slf.update_params();
        slf
    }
    
    fn update_params(&mut self) {
        for c in &mut self.combs_l { c.feedback = self.room_size; }
        for c in &mut self.combs_r { c.feedback = self.room_size; }
    }
}

impl AudioPlugin for Reverb {
    fn name(&self) -> &'static str { "Frost Reverb Room" }

    fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let input = (left + right) * 0.015;
        let mut out_l = 0.0;
        let mut out_r = 0.0;
        
        for c in &mut self.combs_l { out_l += c.process(input, self.damping); }
        for c in &mut self.combs_r { out_r += c.process(input, self.damping); }
        
        for a in &mut self.allpasses_l { out_l = a.process(out_l); }
        for a in &mut self.allpasses_r { out_r = a.process(out_r); }
        
        let l = left + out_l * self.mix;
        let r = right + out_r * self.mix;
        (l, r)
    }

    fn set_param(&mut self, id: u32, value: f32) {
        match id {
            0 => { self.room_size = value.clamp(0.0, 0.98); self.update_params(); },
            1 => self.damping = value.clamp(0.0, 1.0),
            2 => self.mix = value.clamp(0.0, 1.0),
            _ => (),
        }
    }

    fn get_param(&self, id: u32) -> f32 {
        match id {
            0 => self.room_size,
            1 => self.damping,
            2 => self.mix,
            _ => 0.0,
        }
    }

    fn reset(&mut self) {
        for c in &mut self.combs_l { c.buffer.fill(0.0); }
        for c in &mut self.combs_r { c.buffer.fill(0.0); }
    }
}
