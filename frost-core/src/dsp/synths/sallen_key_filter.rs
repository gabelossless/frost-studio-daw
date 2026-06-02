/// Sallen-Key Filter (Resonant HP/LP)
/// Used in the Korg MS-20
pub struct SallenKeyFilter {
    sample_rate: f32,
    cutoff: f32,
    res: f32,
    is_hp: bool,
    s: [f32; 2],
}

impl SallenKeyFilter {
    pub fn new(sample_rate: f32, is_hp: bool) -> Self {
        Self {
            sample_rate,
            cutoff: 1000.0,
            res: 0.0,
            is_hp,
            s: [0.0; 2],
        }
    }

    pub fn set_params(&mut self, cutoff: f32, res: f32) {
        self.cutoff = cutoff;
        self.res = res;
    }

    #[inline(always)]
    pub fn process(&mut self, x: f32) -> f32 {
        let cutoff = self.cutoff.min(self.sample_rate * 0.45);
        let alpha = (2.0 * std::f32::consts::PI * cutoff / self.sample_rate).tan();
        let k = 2.0 * self.res;
        
        // Basic 12dB Sallen-Key approximation
        // This is a simplified version; real MS-20 filters have specific non-linearities.
        let out = if self.is_hp {
            // Simplified HPF
            let input = x - k * self.s[0];
            let out = (input - self.s[1]) * alpha;
            let res = out + self.s[1];
            self.s[1] = res + out;
            self.s[0] = res;
            x - res
        } else {
            // Simplified LPF
            let input = x - k * self.s[0];
            let out = (input - self.s[1]) * alpha;
            let res = out + self.s[1];
            self.s[1] = res + out;
            self.s[0] = res;
            res
        };
        
        out
    }
}
