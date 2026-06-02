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

fn run<T: cpal::Sample + cpal::SizedSample + cpal::FromSample<f32>>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    mixer_state: Arc<Mutex<MixerState>>,
) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
    let channels = config.channels as usize;
    let err_fn = |err| eprintln!("An error occurred on the audio stream: {}", err);

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
                    
                    if channels >= 2 {
                        frame[0] = T::from_sample(out_l);
                        frame[1] = T::from_sample(out_r);
                    } else if channels == 1 {
                        frame[0] = T::from_sample((out_l + out_r) * 0.5);
                    } else {
                        frame[0] = T::from_sample(out_l);
                        frame[1] = T::from_sample(out_r);
                        for i in 2..channels {
                            frame[i] = T::from_sample(0.0);
                        }
                    }
                }
            } else {
                for frame in data.chunks_mut(channels) {
                     for sample in frame.iter_mut() {
                         *sample = T::from_sample(0.0);
                     }
                }
            }
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}
