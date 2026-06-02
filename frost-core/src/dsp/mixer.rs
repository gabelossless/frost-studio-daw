/// Mixer channel and master bus implementation.
/// Each MixerChannel processes audio through:
///   1. 3-band EQ (Low Shelf, Mid Peaking, High Shelf)
///   2. Pan (Constant Power law)
///   3. Volume fader (logarithmic gain)
///
/// The MasterBus sums all channel outputs and computes stereo RMS for metering.

use super::filter::{BiquadFilter, FilterType};
use super::limiter::Limiter;
use serde::{Deserialize, Serialize};

const SAMPLE_RATE: f32 = 44100.0;

/// EQ band parameters sent from the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqBandParams {
    pub freq_hz: f32,
    pub gain_db: f32,
    pub q: f32,
}

impl Default for EqBandParams {
    fn default() -> Self {
        Self {
            freq_hz: 1000.0,
            gain_db: 0.0,
            q: 0.707,
        }
    }
}

/// Full set of parameters for one mixer channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelParams {
    pub channel_id: usize,
    pub volume: f32,  // 0.0 – 1.0
    pub pan: f32,     // -1.0 (L) to 1.0 (R)
    pub muted: bool,
    pub soloed: bool,
    pub eq_low: EqBandParams,
    pub eq_mid: EqBandParams,
    pub eq_high: EqBandParams,
    pub send_1_amount: f32, // Reverb send (0.0 - 1.0)
    pub send_2_amount: f32, // Delay send (0.0 - 1.0)
    pub sidechain_source_id: Option<usize>, // Channel to duck AGAINST
    pub sidechain_ratio: f32,                // ducking depth (0.0 to 1.0)
}

impl Default for ChannelParams {
    fn default() -> Self {
        Self {
            channel_id: 0,
            volume: 0.75,
            pan: 0.0,
            muted: false,
            soloed: false,
            eq_low: EqBandParams { freq_hz: 100.0, gain_db: 0.0, q: 0.707 },
            eq_mid: EqBandParams { freq_hz: 1000.0, gain_db: 0.0, q: 0.707 },
            eq_high: EqBandParams { freq_hz: 8000.0, gain_db: 0.0, q: 0.707 },
            send_1_amount: 0.0,
            send_2_amount: 0.0,
            sidechain_source_id: None,
            sidechain_ratio: 0.0,
        }
    }
}

/// RMS level data emitted to the UI for metering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterLevel {
    pub channel_id: usize,
    pub rms_left: f32,   // 0.0 – 1.0
    pub rms_right: f32,  // 0.0 – 1.0
    pub peak_left: f32,  // 0.0 – 1.0 (instantaneous peak holds)
    pub peak_right: f32,
    pub playhead_beats: f32,
}

/// Single channel in the mixer. Owns its own EQ filters.
pub struct MixerChannel {
    pub params: ChannelParams,
    eq_low_l: BiquadFilter,
    eq_low_r: BiquadFilter,
    eq_mid_l: BiquadFilter,
    eq_mid_r: BiquadFilter,
    eq_high_l: BiquadFilter,
    eq_high_r: BiquadFilter,
    // RMS accumulation (over ~20ms window at 44100 Hz = ~882 samples)
    rms_accum_l: f32,
    rms_accum_r: f32,
    rms_count: usize,
    rms_window: usize,
    // Smoothed RMS output
    pub rms_left: f32,
    pub rms_right: f32,
    pub peak_left: f32,
    pub peak_right: f32,
    // Peak hold decay
    peak_decay: f32,
    // Insert plugins
    pub inserts: Vec<Box<dyn super::plugins::AudioPlugin>>,
}

impl MixerChannel {
    pub fn new(channel_id: usize) -> Self {
        let mut params = ChannelParams::default();
        params.channel_id = channel_id;
        Self {
            eq_low_l: BiquadFilter::new(FilterType::LowShelf, 100.0, 0.0, 0.707, SAMPLE_RATE),
            eq_low_r: BiquadFilter::new(FilterType::LowShelf, 100.0, 0.0, 0.707, SAMPLE_RATE),
            eq_mid_l: BiquadFilter::new(FilterType::Peaking, 1000.0, 0.0, 0.707, SAMPLE_RATE),
            eq_mid_r: BiquadFilter::new(FilterType::Peaking, 1000.0, 0.0, 0.707, SAMPLE_RATE),
            eq_high_l: BiquadFilter::new(FilterType::HighShelf, 8000.0, 0.0, 0.707, SAMPLE_RATE),
            eq_high_r: BiquadFilter::new(FilterType::HighShelf, 8000.0, 0.0, 0.707, SAMPLE_RATE),
            params,
            rms_accum_l: 0.0,
            rms_accum_r: 0.0,
            rms_count: 0,
            rms_window: 882, // ~20ms at 44100 Hz
            rms_left: 0.0,
            rms_right: 0.0,
            peak_left: 0.0,
            peak_right: 0.0,
            peak_decay: 0.9995, // Peak hold decay factor per sample
            inserts: Vec::new(),
        }
    }

    /// Apply new parameters from the UI to this channel.
    pub fn apply_params(&mut self, p: ChannelParams) {
        self.eq_low_l.set_params(p.eq_low.freq_hz, p.eq_low.gain_db, p.eq_low.q);
        self.eq_low_r.set_params(p.eq_low.freq_hz, p.eq_low.gain_db, p.eq_low.q);
        self.eq_mid_l.set_params(p.eq_mid.freq_hz, p.eq_mid.gain_db, p.eq_mid.q);
        self.eq_mid_r.set_params(p.eq_mid.freq_hz, p.eq_mid.gain_db, p.eq_mid.q);
        self.eq_high_l.set_params(p.eq_high.freq_hz, p.eq_high.gain_db, p.eq_high.q);
        self.eq_high_r.set_params(p.eq_high.freq_hz, p.eq_high.gain_db, p.eq_high.q);
        self.params = p;
    }

    /// Process one stereo sample pair through EQ → Pan → Volume.
    /// Returns ((main_l, main_r), (send1_l, send1_r), (send2_l, send2_r)).
    #[inline(always)]
    pub fn process(&mut self, left_in: f32, right_in: f32, sidechain_level: f32) -> ((f32, f32), (f32, f32), (f32, f32)) {
        if self.params.muted {
            self.update_rms(0.0, 0.0);
            return ((0.0, 0.0), (0.0, 0.0), (0.0, 0.0));
        }

        // 1. Stereo EQ
        let eq_l_l = self.eq_low_l.process(left_in);
        let eq_m_l = self.eq_mid_l.process(eq_l_l);
        let mut l = self.eq_high_l.process(eq_m_l);

        let eq_l_r = self.eq_low_r.process(right_in);
        let eq_m_r = self.eq_mid_r.process(eq_l_r);
        let mut r = self.eq_high_r.process(eq_m_r);

        // 2. Insert FX Chain
        for plugin in &mut self.inserts {
            let processed = plugin.process(l, r);
            l = processed.0;
            r = processed.1;
        }

        // 3. Pan — Constant Power panning law
        let pan_angle = (self.params.pan + 1.0) * 0.5;
        let pan_rad = pan_angle * std::f32::consts::FRAC_PI_2;
        let pan_l = pan_rad.cos();
        let pan_r = pan_rad.sin();

        // 4. Volume
        let gain = self.params.volume;
        
        // 5. Sidechain Ducking
        let reduction = 1.0 - (sidechain_level * self.params.sidechain_ratio).clamp(0.0, 1.0);

        let out_l = l * pan_l * gain * reduction;
        let out_r = r * pan_r * gain * reduction;

        self.update_rms(out_l, out_r);

        // Calculate sends (post-fader)
        let send1 = (out_l * self.params.send_1_amount, out_r * self.params.send_1_amount);
        let send2 = (out_l * self.params.send_2_amount, out_r * self.params.send_2_amount);

        ((out_l, out_r), send1, send2)
    }

    #[inline(always)]
    fn update_rms(&mut self, l: f32, r: f32) {
        self.rms_accum_l += l * l;
        self.rms_accum_r += r * r;
        self.rms_count += 1;

        // Update peak hold
        let abs_l = l.abs();
        let abs_r = r.abs();
        if abs_l > self.peak_left { self.peak_left = abs_l; }
        if abs_r > self.peak_right { self.peak_right = abs_r; }
        self.peak_left *= self.peak_decay;
        self.peak_right *= self.peak_decay;

        if self.rms_count >= self.rms_window {
            self.rms_left = (self.rms_accum_l / self.rms_window as f32).sqrt();
            self.rms_right = (self.rms_accum_r / self.rms_window as f32).sqrt();
            self.rms_accum_l = 0.0;
            self.rms_accum_r = 0.0;
            self.rms_count = 0;
        }
    }

    pub fn get_meter(&self) -> MeterLevel {
        MeterLevel {
            channel_id: self.params.channel_id,
            rms_left: self.rms_left.clamp(0.0, 1.0),
            rms_right: self.rms_right.clamp(0.0, 1.0),
            peak_left: self.peak_left.clamp(0.0, 1.0),
            peak_right: self.peak_right.clamp(0.0, 1.0),
            playhead_beats: 0.0, // Filled by host
        }
    }

    pub fn reset(&mut self) {
        self.eq_low_l.reset();
        self.eq_low_r.reset();
        self.eq_mid_l.reset();
        self.eq_mid_r.reset();
        self.eq_high_l.reset();
        self.eq_high_r.reset();
        self.rms_left = 0.0;
        self.rms_right = 0.0;
        self.peak_left = 0.0;
        self.peak_right = 0.0;
    }
}

/// The Master Bus sums all channel outputs and applies a final gain.
pub struct MasterBus {
    pub volume: f32,
    rms_accum_l: f32,
    rms_accum_r: f32,
    rms_count: usize,
    rms_window: usize,
    pub rms_left: f32,
    pub rms_right: f32,
    pub peak_left: f32,
    pub peak_right: f32,
    peak_decay: f32,
    pub limiter: Limiter,
}

impl MasterBus {
    pub fn new() -> Self {
        Self {
            volume: 1.0,
            rms_accum_l: 0.0,
            rms_accum_r: 0.0,
            rms_count: 0,
            rms_window: 882,
            rms_left: 0.0,
            rms_right: 0.0,
            peak_left: 0.0,
            peak_right: 0.0,
            peak_decay: 0.9995,
            limiter: Limiter::new(44100.0),
        }
    }

    #[inline(always)]
    pub fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let (lim_l, lim_r) = self.limiter.process(left_in, right_in);
        let out_l = lim_l * self.volume;
        let out_r = lim_r * self.volume;

        // RMS accumulation
        self.rms_accum_l += out_l * out_l;
        self.rms_accum_r += out_r * out_r;
        self.rms_count += 1;

        let abs_l = out_l.abs();
        let abs_r = out_r.abs();
        if abs_l > self.peak_left { self.peak_left = abs_l; }
        if abs_r > self.peak_right { self.peak_right = abs_r; }
        self.peak_left *= self.peak_decay;
        self.peak_right *= self.peak_decay;

        if self.rms_count >= self.rms_window {
            self.rms_left = (self.rms_accum_l / self.rms_window as f32).sqrt();
            self.rms_right = (self.rms_accum_r / self.rms_window as f32).sqrt();
            self.rms_accum_l = 0.0;
            self.rms_accum_r = 0.0;
            self.rms_count = 0;
        }

        (out_l, out_r)
    }

    pub fn get_meter(&self) -> MeterLevel {
        MeterLevel {
            channel_id: 255, // sentinel for master
            rms_left: self.rms_left.clamp(0.0, 1.0),
            rms_right: self.rms_right.clamp(0.0, 1.0),
            peak_left: self.peak_left.clamp(0.0, 1.0),
            peak_right: self.peak_right.clamp(0.0, 1.0),
            playhead_beats: 0.0,
        }
    }

    pub fn set_limiter_params(&mut self, threshold: f32, ceiling: f32, attack_ms: f32, release_ms: f32) {
        // We'll use a fixed sample rate of 44100.0 for now, 
        // in a real app we'd pass the actual sample rate.
        self.limiter.set_params(threshold, ceiling, attack_ms, release_ms, 44100.0);
    }
}

impl Default for MasterBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_mute() {
        let mut ch = MixerChannel::new(0);
        ch.params.muted = true;
        let ((l, r), _, _) = ch.process(1.0, 1.0, 0.0);
        assert_eq!(l, 0.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn test_center_pan_equal_power() {
        let mut ch = MixerChannel::new(0);
        ch.params.pan = 0.0;
        ch.params.volume = 1.0;
        let ((l, r), _, _) = ch.process(1.0, 1.0, 0.0);
        // At center pan, cos(pi/4) == sin(pi/4) ≈ 0.707
        let expected = std::f32::consts::FRAC_1_SQRT_2;
        assert!((l.abs() - expected).abs() < 0.01, "L was {}", l);
        assert!((r.abs() - expected).abs() < 0.01, "R was {}", r);
    }

    #[test]
    fn test_master_bus_gain() {
        let mut bus = MasterBus::new();
        bus.volume = 0.5;
        let (l, r) = bus.process(1.0, 1.0);
        assert!((l - 0.5).abs() < 0.01, "L was {}", l);
        assert!((r - 0.5).abs() < 0.01, "R was {}", r);
    }

    #[test]
    fn test_stereo_discrete_processing() {
        let mut ch = MixerChannel::new(0);
        ch.params.pan = 0.0; // Center pan
        ch.params.volume = 1.0;
        
        // Pass 1.0 into Left, 0.0 into Right
        let ((l, r), _, _) = ch.process(1.0, 0.0, 0.0);
        
        // If discrete, right output should be 0.0 at center pan because right_in is 0.0
        assert_eq!(r, 0.0, "Right channel received leaked signal!");
        assert!(l > 0.0, "Left channel should have discrete signal");

        // Pass 0.0 into Left, 1.0 into Right
        let ((l2, r2), _, _) = ch.process(0.0, 1.0, 0.0);
        assert_eq!(l2, 0.0, "Left channel received leaked signal!");
        assert!(r2 > 0.0, "Right channel should have discrete signal");
    }
}
