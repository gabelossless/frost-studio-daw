use crate::dsp::envelope::AdsrEnvelope;
use crate::dsp::audio_loader::GLOBAL_SAMPLE_BANK;

pub struct SamplerSynth {
    sample_rate: f32,
    envelope: AdsrEnvelope,
    active_sample_path: Option<String>,
    playback_pos: f64,
    playback_speed: f64,
    active_pitch: u8,
}

impl SamplerSynth {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            envelope: AdsrEnvelope::new(sample_rate),
            active_sample_path: None,
            playback_pos: 0.0,
            playback_speed: 1.0,
            active_pitch: 60,
        }
    }

    pub fn set_sample(&mut self, path: String) {
        self.active_sample_path = Some(path);
    }

    pub fn active_sample_path(&self) -> Option<String> {
        self.active_sample_path.clone()
    }

    pub fn trigger_on(&mut self, pitch: u8, _velocity: f32) {
        // Calculate speed relative to Middle C (60)
        self.active_pitch = pitch;
        self.playback_speed = 2.0_f64.powf((pitch as f64 - 60.0) / 12.0);
        self.playback_pos = 0.0;
        self.envelope.trigger_on();
    }

    pub fn trigger_off(&mut self) {
        self.envelope.trigger_off();
    }

    pub fn is_active(&self) -> bool {
        !self.envelope.is_idle()
    }

    pub fn next_sample(&mut self) -> f32 {
        if !self.is_active() { return 0.0; }

        let path = if let Some(p) = &self.active_sample_path { p } else { return 0.0; };
        let bank = &GLOBAL_SAMPLE_BANK;
        
        if let Some(sample) = bank.get_sample(path) {
            let data = &sample.data;
            let len = data.len();
            if len == 0 { return 0.0; }

            // Adjusted speed based on recording sample rate vs current sample rate
            let speed_ratio = (sample.sample_rate as f64) / (self.sample_rate as f64);
            let final_speed = self.playback_speed * speed_ratio;

            let idx = self.playback_pos;
            let i = idx.floor() as usize;
            let fract = (idx - i as f64) as f32;

            // Simple mono pick (averaging if stereo or just picking L)
            // Assuming interleaved data: [L, R, L, R, ...]
            let channels = sample.channels as usize;
            
            let get_frame = |frame_idx: usize| -> f32 {
                if frame_idx >= (len / channels) { return 0.0; }
                let base = frame_idx * channels;
                if channels >= 2 {
                    (data[base] + data[base + 1]) * 0.5
                } else {
                    data[base]
                }
            };

            let s0 = get_frame(i);
            let s1 = get_frame(i + 1);
            
            let interpolated = s0 + (s1 - s0) * fract;
            let env_gain = self.envelope.next_sample();

            self.playback_pos += final_speed;

            // Loop or stop? For now, just stop at end
            if self.playback_pos >= (len / channels) as f64 {
                // self.envelope.force_off(); // Maybe just let it play out?
                // For a sampler, we often want it to stay "active" until envelope finish
                // but if we hit the end of the buffer, we just repeat last sample or 0
            }

            interpolated * env_gain
        } else {
            0.0
        }
    }

    pub fn set_params(&mut self, a: f32, d: f32, s: f32, r: f32) {
        self.envelope.set_params(a, d, s, r);
    }
}
