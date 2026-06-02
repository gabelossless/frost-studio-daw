/// Moog-style Transistor Ladder Filter (Zero-Delay Feedback approximation)
pub struct LadderFilter {
    sample_rate: f32,
    cutoff: f32,
    res: f32,
    s: [f32; 4], // State variables
}

impl LadderFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            cutoff: 1000.0,
            res: 0.0,
            s: [0.0; 4],
        }
    }

    pub fn set_params(&mut self, cutoff: f32, res: f32) {
        self.cutoff = cutoff;
        self.res = res;
    }

    #[inline(always)]
    pub fn process(&mut self, x: f32) -> f32 {
        let f = (2.0 * std::f32::consts::PI * self.cutoff / self.sample_rate).tanh();
        let k = 4.0 * self.res;
        
        // Zero-Delay Feedback approximation
        let g = f;
        let g_pow2 = g * g;
        let g_pow3 = g_pow2 * g;
        let g_pow4 = g_pow3 * g;
        
        let sigma = g_pow4 * k;
        let gamma = 1.0 / (1.0 + sigma);
        
        // Input with feedback
        let feedback = k * self.s[3];
        let input = (x - feedback) * gamma;
        
        // Cascade of 4 filter stages
        let mut out = input;
        for i in 0..4 {
            let v = (out - self.s[i]) * g;
            let res = v + self.s[i];
            self.s[i] = res + v;
            out = res;
        }
        
        out
    }
}
