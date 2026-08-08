/// Digital Biquad Filter for 3-Band EQ
/// Supports: Low Shelf, Mid Peaking, High Shelf
/// Uses 32-bit floating-point math optimized for the audio thread.

use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    LowShelf,
    Peaking,
    HighShelf,
}

/// Transposed Direct Form II Biquad Filter
/// State variables use f32 for performance in the audio thread.
#[derive(Debug, Clone)]
pub struct BiquadFilter {
    filter_type: FilterType,
    // Coefficients (Current)
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // Coefficients (Target for Smoothing)
    target_b0: f32,
    target_b1: f32,
    target_b2: f32,
    target_a1: f32,
    target_a2: f32,
    // Delay state (Direct Form II Transposed)
    z1: f32,
    z2: f32,
    // Stored params for recalculation
    pub freq_hz: f32,
    pub gain_db: f32,
    pub q: f32,
    sample_rate: f32,
}

impl BiquadFilter {
    pub fn new(filter_type: FilterType, freq_hz: f32, gain_db: f32, q: f32, sample_rate: f32) -> Self {
        let mut filter = Self {
            filter_type,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            target_b0: 1.0,
            target_b1: 0.0,
            target_b2: 0.0,
            target_a1: 0.0,
            target_a2: 0.0,
            z1: 0.0,
            z2: 0.0,
            freq_hz,
            gain_db,
            q,
            sample_rate,
        };
        filter.update_coefficients();
        // Start current coefficients at target to avoid initial swoop
        filter.b0 = filter.target_b0;
        filter.b1 = filter.target_b1;
        filter.b2 = filter.target_b2;
        filter.a1 = filter.target_a1;
        filter.a2 = filter.target_a2;
        filter
    }

    /// Update filter parameters and recalculate coefficients.
    /// Call this from the UI thread; it's safe because Mutex guards the channel state.
    pub fn set_params(&mut self, freq_hz: f32, gain_db: f32, q: f32) {
        self.freq_hz = freq_hz;
        self.gain_db = gain_db;
        self.q = q;
        self.update_coefficients();
    }

    /// Retrieve the current parameters.
    pub fn get_params(&self) -> (f32, f32, f32) {
        (self.freq_hz, self.gain_db, self.q)
    }

    fn update_coefficients(&mut self) {
        let fs = self.sample_rate;
        let f0 = self.freq_hz.clamp(20.0, fs / 2.0 - 1.0);
        let gain_db = self.gain_db.clamp(-24.0, 24.0);
        let q = self.q.clamp(0.1, 10.0);

        let a = 10.0_f32.powf(gain_db / 40.0); // sqrt of linear gain
        let w0 = 2.0 * PI * f0 / fs;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        match self.filter_type {
            FilterType::Peaking => {
                // Peaking EQ filter (Robert Bristow-Johnson cookbook)
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha / a;

                self.target_b0 = b0 / a0;
                self.target_b1 = b1 / a0;
                self.target_b2 = b2 / a0;
                self.target_a1 = a1 / a0;
                self.target_a2 = a2 / a0;
            }
            FilterType::LowShelf => {
                // Low shelf filter
                // The radicand can go negative for boosted shelves with Q > 1;
                // clamp it so we never emit NaN coefficients into the audio thread.
                let alpha_s = sin_w0 / 2.0 * ((a + 1.0 / a) * (1.0 / q - 1.0) + 2.0).max(0.0).sqrt();
                let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * alpha_s);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * alpha_s);
                let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * alpha_s;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * alpha_s;

                self.target_b0 = b0 / a0;
                self.target_b1 = b1 / a0;
                self.target_b2 = b2 / a0;
                self.target_a1 = a1 / a0;
                self.target_a2 = a2 / a0;
            }
            FilterType::HighShelf => {
                // High shelf filter
                let alpha_s = sin_w0 / 2.0 * ((a + 1.0 / a) * (1.0 / q - 1.0) + 2.0).max(0.0).sqrt();
                let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * alpha_s);
                let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * alpha_s);
                let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * alpha_s;
                let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * alpha_s;

                self.target_b0 = b0 / a0;
                self.target_b1 = b1 / a0;
                self.target_b2 = b2 / a0;
                self.target_a1 = a1 / a0;
                self.target_a2 = a2 / a0;
            }
        }
    }

    /// Process a single sample through the filter.
    /// Uses Transposed Direct Form II for numerical stability and efficiency.
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        const SMOOTH: f32 = 0.05;
        // Smoothly interpolate coefficients to targets
        self.b0 += (self.target_b0 - self.b0) * SMOOTH;
        self.b1 += (self.target_b1 - self.b1) * SMOOTH;
        self.b2 += (self.target_b2 - self.b2) * SMOOTH;
        self.a1 += (self.target_a1 - self.a1) * SMOOTH;
        self.a2 += (self.target_a2 - self.a2) * SMOOTH;

        let output = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * output + self.z2;
        self.z2 = self.b2 * input - self.a2 * output;
        output
    }

    /// Reset filter state (e.g., on transport stop / discontinuity).
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unity_gain_passthrough() {
        // A 0dB peaking filter should pass audio unchanged
        let mut f = BiquadFilter::new(FilterType::Peaking, 1000.0, 0.0, 0.707, 44100.0);
        let input = 0.5f32;
        let output = f.process(input);
        // After filter settles, output should be close to input
        assert!((output - input).abs() < 0.01);
    }

    #[test]
    fn test_coefficient_normalization() {
        let f = BiquadFilter::new(FilterType::LowShelf, 200.0, 6.0, 0.707, 44100.0);
        // b0 should be reasonably bounded
        assert!(f.b0.is_finite());
        assert!(f.b1.is_finite());
        assert!(f.b2.is_finite());
    }

    #[test]
    fn test_shelf_q_above_one_never_nan() {
        // Regression: boosted shelves with Q > 1 previously produced a negative
        // sqrt radicand, injecting NaN into the audio thread.
        for filter_type in [FilterType::LowShelf, FilterType::HighShelf] {
            for gain_db in [-24.0, -12.0, 0.0, 6.0, 24.0] {
                let mut f = BiquadFilter::new(filter_type, 500.0, gain_db, 3.0, 44100.0);
                assert!(f.b0.is_finite(), "{:?} gain {} -> b0 not finite", filter_type, gain_db);
                assert!(f.a1.is_finite(), "{:?} gain {} -> a1 not finite", filter_type, gain_db);
                let out = f.process(0.5);
                assert!(out.is_finite(), "{:?} gain {} -> output not finite", filter_type, gain_db);
            }
        }
    }
}
