use crate::engine::MixerState;

pub struct Exporter;

impl Exporter {
    pub fn export_to_wav(
        mixer_state: &mut MixerState,
        path: &str,
        duration_beats: f32,
    ) -> Result<(), String> {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44100,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        let mut writer = hound::WavWriter::create(path, spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

        // Reset clock to start
        mixer_state.clock.stop();
        mixer_state.clock.start();

        // Calculate total samples
        // duration_beats * (60 / bpm) * sample_rate
        let bpm = mixer_state.clock.bpm;
        let total_samples = (duration_beats * (60.0 / bpm) * 44100.0) as usize;

        for _ in 0..total_samples {
            let (l, r) = mixer_state.generate_frame();
            writer.write_sample(l).map_err(|e| e.to_string())?;
            writer.write_sample(r).map_err(|e| e.to_string())?;
        }

        writer.finalize().map_err(|e| format!("Failed to finalize WAV: {}", e))?;
        
        // Reset clock back to zero
        mixer_state.clock.stop();

        Ok(())
    }
}
