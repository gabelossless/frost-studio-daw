use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::default::get_probe;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSample {
    pub name: String,
    pub path: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_seconds: f64,
    #[serde(skip)]
    pub data: Arc<Vec<f32>>, // Interleaved float32 data
}

const MAX_CACHED_SAMPLES: usize = 256;

pub struct SampleBank {
    pub samples: RwLock<HashMap<String, AudioSample>>,
}

impl SampleBank {
    pub fn new() -> Self {
        Self {
            samples: RwLock::new(HashMap::new()),
        }
    }

    fn evict_if_needed(&self) {
        let mut samples = self.samples.write();
        if samples.len() >= MAX_CACHED_SAMPLES {
            // Remove 20% of oldest entries
            let to_remove = samples.len() / 5;
            let keys: Vec<String> = samples.keys().take(to_remove).cloned().collect();
            for key in keys {
                samples.remove(&key);
            }
        }
    }

    /// Load an audio file (WAV, MP3) and decode it into a floating point buffer
    pub fn load_sample(&self, file_path: &str) -> Result<AudioSample, String> {
        let path = Path::new(file_path);
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        let file = Box::new(File::open(path).map_err(|e| e.to_string())?);
        let mss = MediaSourceStream::new(file, Default::default());

        let mut hint = symphonia::core::probe::Hint::new();
        if let Some(ext) = path.extension() {
            hint.with_extension(&ext.to_string_lossy());
        }

        let probed = get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| format!("Failed to probe format: {}", e))?;

        let mut format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("No supported audio tracks found")?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2) as u16;

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| format!("Failed to create decoder: {}", e))?;

        let mut sample_buffer = None;
        let mut float_data = Vec::new();

        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::ResetRequired) => {
                    decoder.reset();
                    continue;
                }
                Err(SymphoniaError::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(_) => break, // Other errors (EOF usually ends up here or above)
            };

            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    if sample_buffer.is_none() {
                        let spec = *decoded.spec();
                        let duration = decoded.capacity() as u64;
                        sample_buffer = Some(SampleBuffer::<f32>::new(duration, spec));
                    }

                    if let Some(buf) = &mut sample_buffer {
                        buf.copy_interleaved_ref(decoded);
                        float_data.extend_from_slice(buf.samples());
                    }
                }
                Err(SymphoniaError::DecodeError(_)) => continue, // Recoverable error
                Err(e) => return Err(format!("Decode error: {}", e)),
            }
        }

        let frames = float_data.len() / (channels as usize);
        let duration_seconds = frames as f64 / sample_rate as f64;

        let sample = AudioSample {
            name,
            path: file_path.to_string(),
            sample_rate,
            channels,
            duration_seconds,
            data: Arc::new(float_data),
        };

        // Cache the sample (with LRU-like eviction)
        self.evict_if_needed();
        self.samples.write().insert(file_path.to_string(), sample.clone());

        Ok(sample)
    }

    pub fn get_sample(&self, path: &str) -> Option<AudioSample> {
        self.samples.read().get(path).cloned()
    }
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_SAMPLE_BANK: SampleBank = SampleBank::new();
}
