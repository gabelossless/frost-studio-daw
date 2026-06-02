/// Brickwall Limiter and Hard Clipper for the Master Bus.
/// This ensures the signal never exceeds 0 dBFS.

pub struct Limiter {
    threshold: f32, // 1.0 = 0dBFS
    ceiling: f32,   // 0.99 or 1.0
    attack_samples: f32,
    release_samples: f32,
    envelope: f32,
}

impl Limiter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: 1.0, 
            ceiling: 0.99, // Prevent inter-sample peaks slightly
            attack_samples: 0.001 * sample_rate, // 1ms
            release_samples: 0.1 * sample_rate,  // 100ms
            envelope: 0.0,
        }
    }

    pub fn set_params(&mut self, threshold: f32, ceiling: f32, attack_ms: f32, release_ms: f32, sample_rate: f32) {
        self.threshold = threshold;
        self.ceiling = ceiling;
        self.attack_samples = (attack_ms * 0.001 * sample_rate).max(1.0);
        self.release_samples = (release_ms * 0.001 * sample_rate).max(1.0);
    }

    /// Process a stereo sample pair.
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Detect peak
        let peak = left.abs().max(right.abs());
        
        // Update envelope (peak detection)
        if peak > self.envelope {
            self.envelope += (peak - self.envelope) / self.attack_samples;
        } else {
            self.envelope += (peak - self.envelope) / self.release_samples;
        }

        // Calculate gain reduction
        let gain = if self.envelope > self.threshold {
            self.threshold / self.envelope
        } else {
            1.0
        };

        // Apply gain and hard clip at ceiling just in case
        let mut out_l = left * gain;
        let mut out_r = right * gain;

        out_l = out_l.clamp(-self.ceiling, self.ceiling);
        out_r = out_r.clamp(-self.ceiling, self.ceiling);

        (out_l, out_r)
    }
}
