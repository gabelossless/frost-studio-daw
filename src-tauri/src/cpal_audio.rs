use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use parking_lot::Mutex;
use crate::MixerState;

pub fn get_available_hosts() -> Vec<String> {
    cpal::available_hosts().iter().map(|h| format!("{:?}", h)).collect()
}

pub fn get_available_devices(host_name: &str) -> Result<Vec<String>, String> {
    let hosts = cpal::available_hosts();
    let host_id_opt = hosts.iter().find(|h| format!("{:?}", h).to_lowercase() == host_name.to_lowercase());
    
    let host_id = match host_id_opt {
        Some(h) => *h,
        None => return Err(format!("Host '{}' not found", host_name)),
    };

    let host = cpal::host_from_id(host_id).map_err(|e| e.to_string())?;
    let devices = host.output_devices().map_err(|e| e.to_string())?;
    
    let mut names = Vec::new();
    for d in devices {
        if let Ok(name) = d.name() {
            names.push(name);
        }
    }
    Ok(names)
}

pub fn start_audio_engine(
    mixer: Arc<Mutex<MixerState>>,
    host_name: Option<&str>,
    device_name: Option<&str>,
    buffer_size: Option<u32>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let host = if let Some(h_name) = host_name {
        let hosts = cpal::available_hosts();
        let id_opt = hosts.iter().find(|h| format!("{:?}", h).to_lowercase() == h_name.to_lowercase());
        let id = id_opt.ok_or_else(|| format!("Host '{}' not found or not supported", h_name))?;
        cpal::host_from_id(*id)?
    } else {
        cpal::default_host()
    };

    let device = if let Some(d_name) = device_name {
        let mut devices = host.output_devices()?;
        devices.find(|d| d.name().map(|n| n == d_name).unwrap_or(false))
            .ok_or_else(|| format!("Device '{}' not found on host '{:?}'", d_name, host.id()))?
    } else {
        host.default_output_device().ok_or("No default output device found")?
    };

    let default_config = device.default_output_config()?;
    let mut config: cpal::StreamConfig = default_config.clone().into();
    
    if let Some(size) = buffer_size {
        config.buffer_size = cpal::BufferSize::Fixed(size);
    } else {
        config.buffer_size = cpal::BufferSize::Default;
    }

    // Rebuild the engine's DSP at the device's actual sample rate.
    // Not on the audio thread, so a blocking lock is fine here.
    {
        let mut guard = mixer.lock();
        guard.set_sample_rate(config.sample_rate.0 as f32);
    }

    println!("Starting CPAL audio engine. Host: {:?}, Device: {}, Config: {:?}", host.id(), device.name().unwrap_or_default(), config);

    let sample_format = default_config.sample_format();

    let stream = match sample_format {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config, mixer),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config, mixer),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config, mixer),
        _ => return Err("Unsupported sample format".into()),
    }?;

    stream.play()?;
    Ok(stream)
}

/// Write one stereo pair into a frame, handling every channel layout.
/// Channels beyond the stereo pair are zero-filled so multichannel
/// devices never receive uninitialized/garbage samples.
#[inline(always)]
fn write_output_frame<T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>>(
    frame: &mut [T],
    out_l: f32,
    out_r: f32,
) {
    match frame.len() {
        0 => {}
        1 => frame[0] = T::from_sample((out_l + out_r) * 0.5),
        2 => {
            frame[0] = T::from_sample(out_l);
            frame[1] = T::from_sample(out_r);
        }
        n => {
            frame[0] = T::from_sample(out_l);
            frame[1] = T::from_sample(out_r);
            for sample in frame[2..n].iter_mut() {
                *sample = T::from_sample(0.0);
            }
        }
    }
}

fn run<T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mixer_state: Arc<Mutex<MixerState>>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("An error occurred on the audio stream: {}", err);

    // Held across callbacks so a lock miss can repeat the last valid sample
    // instead of clicking to zero. Faded to silence if contention persists.
    let mut last_l: f32 = 0.0;
    let mut last_r: f32 = 0.0;
    let mut lock_misses: u32 = 0;

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            if let Some(mut mixer) = mixer_state.try_lock() {
                let is_playing = mixer.clock.is_playing;
                for frame in data.chunks_mut(channels) {
                    let (out_l, out_r) = if is_playing {
                        mixer.generate_frame()
                    } else {
                        (0.0, 0.0)
                    };
                    write_output_frame(frame, out_l, out_r);
                    last_l = out_l;
                    last_r = out_r;
                }
                lock_misses = 0;
            } else {
                // The UI thread briefly holds the mixer lock. Holding the last
                // valid frame avoids a hard click-to-zero dropout; if the
                // contention persists, fade to silence instead of outputting DC.
                for frame in data.chunks_mut(channels) {
                    write_output_frame(frame, last_l, last_r);
                }
                lock_misses += 1;
                if lock_misses > 64 {
                    last_l = 0.0;
                    last_r = 0.0;
                }
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}
