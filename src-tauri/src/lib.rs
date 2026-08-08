mod cpal_audio;

use frost_core::dsp::mixer::{ChannelParams, MeterLevel};
use frost_core::dsp::synths::manager::{SynthParams, SynthType};
use frost_core::dsp::midi::{NoteEvent};
use frost_core::dsp::exporter::Exporter;
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use std::path::{PathBuf};

use frost_core::vst::{scan_vst3_plugins, VstPluginInfo};
use frost_core::dsp::presets::{SynthPreset};
use frost_core::dsp::plugins::{AudioPlugin, compressor::Compressor, eq::ParametricEQ, limiter::Limiter as FrostLimiter, bass::BassSynthPlugin};
use frost_core::dsp::audio_loader::{GLOBAL_SAMPLE_BANK, AudioSample};

use frost_core::dsp::audio_track::{AudioTrack};
use frost_core::{MixerState, SharedMixer, NUM_CHANNELS, SampleNode, scan_dir_recursive};

// ────────────────────────────────────────────────────────────────────────────
// Tauri Commands
// ────────────────────────────────────────────────────────────────────────────

/// Set full parameter state for one mixer channel.
#[tauri::command]
fn set_channel_params(
    state: State<'_, SharedMixer>,
    params: ChannelParams,
) -> Result<(), String> {
    let mut mixer = state.lock();
    let id = params.channel_id;
    if id < NUM_CHANNELS {
        mixer.channels[id].apply_params(params);
        Ok(())
    } else {
        Err(format!("Channel {} out of range", id))
    }
}

/// Set the master volume (0.0–1.0).
#[tauri::command]
fn set_master_volume(
    state: State<'_, SharedMixer>,
    volume: f32,
) -> Result<(), String> {
    let mut mixer = state.lock();
    mixer.master.volume = volume.clamp(0.0, 1.5);
    Ok(())
}

#[tauri::command]
fn set_master_limiter_params(
    state: State<'_, SharedMixer>,
    threshold: f32,
    ceiling: f32,
    attack_ms: f32,
    release_ms: f32,
) -> Result<(), String> {
    let mut mixer = state.lock();
    mixer.master.set_limiter_params(threshold, ceiling, attack_ms, release_ms);
    Ok(())
}

#[tauri::command]
fn trigger_note_on(
    channel_id: usize,
    note: u8,
    velocity: u8,
    state: State<'_, SharedMixer>,
) -> Result<(), String> {
    let mut mixer = state.lock();
    if let Some(synth) = mixer.synths.get_mut(channel_id) {
        synth.note_on(note, velocity as f32 / 127.0);
        Ok(())
    } else {
        Err(format!("No synth on channel {}", channel_id))
    }
}

#[tauri::command]
fn trigger_note_off(
    channel_id: usize,
    note: u8,
    state: State<'_, SharedMixer>,
) -> Result<(), String> {
    let mut mixer = state.lock();
    if let Some(synth) = mixer.synths.get_mut(channel_id) {
        synth.note_off(note);
        Ok(())
    } else {
        Err(format!("No synth on channel {}", channel_id))
    }
}

#[tauri::command]
async fn sync_midi_data(
    notes: Vec<NoteEvent>,
    state: State<'_, SharedMixer>,
) -> Result<(), String> {
    let mut mixer = state.lock();
    mixer.playlist.update(notes);
    // Reset cursor when playlist changes
    mixer.next_note_index = 0;
    mixer.active_notes.clear();
    Ok(())
}

#[tauri::command]
fn set_tempo(
    state: State<'_, SharedMixer>,
    tempo: f32,
) -> Result<(), String> {
    let mut mixer = state.lock();
    mixer.clock.set_bpm(tempo);
    Ok(())
}

#[tauri::command]
fn set_synth_params(
    state: State<'_, SharedMixer>,
    params: SynthParams,
) -> Result<(), String> {
    let mut mixer = state.lock();
    for synth in mixer.synths.iter_mut() {
        synth.set_params(params);
    }
    Ok(())
}

#[tauri::command]
fn set_synth_type(
    channel_id: usize,
    synth_type: String,
    state: State<'_, SharedMixer>,
) -> Result<(), String> {
    let mut mixer = state.lock();
    let stype = match synth_type.as_str() {
        "Summit" => SynthType::Summit,
        "Eruption" => SynthType::Eruption,
        "Nebula" => SynthType::Nebula,
        "Sampler" => SynthType::Sampler,
        _ => return Err(format!("Unknown synth type: {}", synth_type)),
    };

    if let Some(synth) = mixer.synths.get_mut(channel_id) {
        synth.set_type(stype);
        Ok(())
    } else {
        Err(format!("Channel {} out of range", channel_id))
    }
}

#[tauri::command]
fn set_transport(
    playing: bool,
    state: State<'_, SharedMixer>,
) -> Result<(), String> {
    let mut mixer = state.lock();
    if playing {
        mixer.clock.start();
    } else {
        // Pause: keep the clock position and note cursor so playback resumes
        // seamlessly. Voices are NOT torn down — releasing them here would
        // also wipe the user's synth type, sampler sample, and params.
        mixer.clock.stop();
    }
    Ok(())
}

/// Full transport stop: rewind the clock, reset the sequencer cursor, clear
/// active notes, and force-release any ringing voices (without wiping the
/// user's synth configuration).
#[tauri::command]
fn reset_transport(state: State<'_, SharedMixer>) -> Result<(), String> {
    let mut mixer = state.lock();
    mixer.clock.stop();
    mixer.clock.reset();
    mixer.next_note_index = 0;
    mixer.active_notes.clear();
    for s in mixer.synths.iter_mut() {
        s.release_all();
    }
    Ok(())
}

/// Get the current meter levels for all channels + master.
#[tauri::command]
fn get_meter_levels(state: State<'_, SharedMixer>) -> Vec<MeterLevel> {
    let mixer = state.lock();
    let mut levels: Vec<MeterLevel> = mixer
        .channels
        .iter()
        .map(|ch| ch.get_meter())
        .collect();
    levels.push(mixer.master.get_meter());
    levels
}

#[tauri::command]
fn process_audio_tick(
    state: State<'_, SharedMixer>,
    app: AppHandle,
) -> Result<(), String> {
    let mixer = state.lock();
    let playhead = mixer.clock.get_position_beats();

    // Emitting meter levels that are continuously updated by the native cpal audio thread
    let mut levels: Vec<MeterLevel> = mixer.channels.iter().map(|ch| {
        let mut m = ch.get_meter();
        m.playhead_beats = playhead;
        m
    }).collect();
    
    let mut master_meter = mixer.master.get_meter();
    master_meter.playhead_beats = playhead;
    levels.push(master_meter);

    let _ = app.emit("meter-levels", &levels);

    Ok(())
}

/// Get initial channel parameter defaults so the UI can populate itself.
#[tauri::command]
fn get_channel_defaults() -> Vec<ChannelParams> {
    (0..NUM_CHANNELS)
        .map(|i| {
            let mut p = ChannelParams::default();
            p.channel_id = i;
            p
        })
        .collect()
}

#[tauri::command]
fn get_available_vst3_plugins() -> Vec<VstPluginInfo> {
    scan_vst3_plugins()
}

#[tauri::command]
fn get_synth_presets(synth_type: String, state: State<'_, SharedMixer>) -> Result<Vec<SynthPreset>, String> {
    // Prefer on-disk presets (src-tauri/presets/<synth>/), falling back to the
    // in-memory factory bank so the command works even without a source tree.
    let dir = {
        let mut p = std::env::current_dir().unwrap_or_default();
        p.push("src-tauri");
        p.push("presets");
        p.push(synth_type.to_lowercase());
        p
    };
    let from_disk = frost_core::dsp::presets::load_presets_from_dir(&dir);
    if !from_disk.is_empty() {
        return Ok(from_disk);
    }

    let mixer = state.lock();
    match synth_type.as_str() {
        "Summit" => Ok(mixer.synth_bank.summit_presets.clone()),
        "Eruption" => Ok(mixer.synth_bank.eruption_presets.clone()),
        "Nebula" => Ok(mixer.synth_bank.nebula_presets.clone()),
        _ => Err(format!("Unknown synth type: {}", synth_type)),
    }
}

#[tauri::command]
fn add_native_plugin(channel_id: usize, plugin_type: String, state: State<'_, SharedMixer>) -> Result<Vec<String>, String> {
    let mut mixer = state.lock();
    let sr = mixer.sample_rate;
    let channel = mixer.channels.get_mut(channel_id).ok_or("Invalid channel ID")?;
    
    let plugin: Box<dyn AudioPlugin> = match plugin_type.as_str() {
        "Compressor" => Box::new(Compressor::new(sr)),
        "EQ" => Box::new(ParametricEQ::new(sr)),
        "Limiter" => Box::new(FrostLimiter::new(sr)),
        "Bass" => Box::new(BassSynthPlugin::new(sr)),
        "Delay" => Box::new(frost_core::dsp::effects::Delay::new(sr)),
        "Reverb" => Box::new(frost_core::dsp::effects::Reverb::new(sr)),
        "Saturator" | "Distortion" => Box::new(frost_core::dsp::plugins::distortion::DistortionPlugin::new()),
        _ => return Err(format!("Unknown plugin type: {}", plugin_type)),
    };
    
    channel.inserts.push(plugin);
    let names = channel.inserts.iter().map(|p| p.name().to_string()).collect();
    Ok(names)
}

#[tauri::command]
fn remove_native_plugin(channel_id: usize, index: usize, state: State<'_, SharedMixer>) -> Result<Vec<String>, String> {
    let mut mixer = state.lock();
    let channel = mixer.channels.get_mut(channel_id).ok_or("Invalid channel ID")?;
    if index < channel.inserts.len() {
        channel.inserts.remove(index);
    }
    let names = channel.inserts.iter().map(|p| p.name().to_string()).collect();
    Ok(names)
}

#[tauri::command]
fn get_plugins(channel_id: usize, state: State<'_, SharedMixer>) -> Result<Vec<String>, String> {
    let mixer = state.lock();
    let channel = mixer.channels.get(channel_id).ok_or("Invalid channel ID")?;
    let names = channel.inserts.iter().map(|p| p.name().to_string()).collect();
    Ok(names)
}

#[tauri::command]
fn set_plugin_param(channel_id: usize, plugin_index: usize, param_id: u32, value: f32, state: State<'_, SharedMixer>) -> Result<(), String> {
    let mut mixer = state.lock();
    let channel = mixer.channels.get_mut(channel_id).ok_or("Invalid channel ID")?;
    let plugin = channel.inserts.get_mut(plugin_index).ok_or("Invalid plugin index")?;
    plugin.set_param(param_id, value);
    Ok(())
}

#[tauri::command]
fn get_sample_waveform(path: String, buckets: u32) -> Result<Vec<f32>, String> {
    let sample = if let Some(s) = GLOBAL_SAMPLE_BANK.get_sample(&path) {
        s
    } else {
        GLOBAL_SAMPLE_BANK.load_sample(&path)?
    };

    let data = &sample.data;
    if data.is_empty() {
        return Ok(vec![]);
    }

    let channels = sample.channels as usize;
    let total_frames = data.len() / channels;
    let mut result = Vec::with_capacity(buckets as usize);

    if total_frames == 0 {
        return Ok(vec![]);
    }

    let interval = (total_frames as f32 / buckets as f32).max(1.0);

    for i in 0..buckets {
        let start_frame = (i as f32 * interval) as usize;
        let end_frame = (((i + 1) as f32 * interval) as usize).min(total_frames);
        
        if start_frame >= end_frame {
            result.push(0.0);
            continue;
        }

        let mut max_abs: f32 = 0.0;
        for f in start_frame..end_frame {
            for c in 0..channels {
                let idx = f * channels + c;
                if idx < data.len() {
                    let val = data[idx].abs();
                    if val > max_abs {
                        max_abs = val;
                    }
                }
            }
        }
        result.push(max_abs);
    }

    Ok(result)
}

#[tauri::command]
async fn load_sample_to_memory(path: String) -> Result<AudioSample, String> {
    // Decoding is CPU intensive, runs natively.
    GLOBAL_SAMPLE_BANK.load_sample(&path)
}

#[tauri::command]
fn preview_sample(_path: String, _state: State<'_, SharedMixer>) -> Result<(), String> {
    // For now, let's just trigger it on the first available sampler if one exists
    // or eventually implement a dedicated preview bus
    Ok(())
}

#[tauri::command]
fn set_sampler_sample(channel_id: usize, path: String, state: State<'_, SharedMixer>) -> Result<(), String> {
    let mut mixer = state.lock();
    if let Some(synth) = mixer.synths.get_mut(channel_id) {
        synth.set_sampler_sample(path);
        Ok(())
    } else {
        Err(format!("Channel {} out of range", channel_id))
    }
}

#[tauri::command]
fn sync_audio_tracks(tracks: Vec<AudioTrack>, state: State<'_, SharedMixer>) -> Result<(), String> {
    let mut mixer = state.lock();
    mixer.audio_tracks = tracks;
    Ok(())
}

#[tauri::command]
fn scan_sample_folder(path: Option<String>) -> Vec<SampleNode> {
    let dir_path = match path {
        Some(p) => PathBuf::from(p),
        None => {
            let mut p = std::env::current_dir().unwrap_or_default();
            p.push("FrostSamples");
            if !p.exists() {
                std::fs::create_dir_all(&p).ok();
            }
            p
        }
    };

    if dir_path.exists() {
        scan_dir_recursive(&dir_path).unwrap_or_default()
    } else {
        vec![]
    }
}

#[tauri::command]
async fn export_project(
    path: String,
    duration_beats: f32,
    state: State<'_, SharedMixer>,
) -> Result<(), String> {
    let mut mixer = state.lock();
    Exporter::export_to_wav(&mut mixer, &path, duration_beats)
}

// ────────────────────────────────────────────────────────────────────────────
// Audio Engine Management
// ────────────────────────────────────────────────────────────────────────────

pub enum AudioMessage {
    SetDevice {
        host: String,
        device: String,
        buffer_size: Option<u32>,
    },
}

pub struct AudioEngineState {
    pub tx: Mutex<std::sync::mpsc::Sender<AudioMessage>>,
    pub current_host: Mutex<String>,
    pub current_device: Mutex<String>,
    pub buffer_size: Mutex<Option<u32>>,
}

impl AudioEngineState {
    pub fn new(tx: std::sync::mpsc::Sender<AudioMessage>) -> Self {
        Self {
            tx: Mutex::new(tx),
            current_host: Mutex::new("Default".to_string()),
            current_device: Mutex::new("Default".to_string()),
            buffer_size: Mutex::new(None),
        }
    }
}

#[tauri::command]
fn get_audio_hosts() -> Vec<String> {
    cpal_audio::get_available_hosts()
}

#[tauri::command]
fn get_audio_devices(host: String) -> Result<Vec<String>, String> {
    cpal_audio::get_available_devices(&host)
}

#[tauri::command]
fn set_audio_device(
    host: String,
    device: String,
    buffer_size: Option<u32>,
    engine_state: State<'_, AudioEngineState>,
) -> Result<(), String> {
    let tx = engine_state.tx.lock();
    
    tx.send(AudioMessage::SetDevice {
        host: host.clone(),
        device: device.clone(),
        buffer_size,
    }).map_err(|e| e.to_string())?;

    *engine_state.current_host.lock() = host;
    *engine_state.current_device.lock() = device;
    *engine_state.buffer_size.lock() = buffer_size;

    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// App Entry Point
// ────────────────────────────────────────────────────────────────────────────

/// Owns the CPAL stream on a dedicated thread (bypassing Send/Sync issues with
/// `cpal::Stream`). Restarts the engine on device changes and falls back to the
/// last-known-good configuration if a restart fails, so audio never dies
/// permanently because of one bad device selection.
fn audio_engine_thread(mixer: SharedMixer, rx: std::sync::mpsc::Receiver<AudioMessage>, app: AppHandle) {
    // Kept alive for the lifetime of the thread: dropping a `cpal::Stream` stops
    // audio, so we must not let it go out of scope while the engine is running.
    let mut _stream: Option<cpal::Stream> = None;
    let mut last_good: Option<(Option<String>, Option<String>, Option<u32>)> = None;

    // Start initial stream on the default host/device.
    match cpal_audio::start_audio_engine(Arc::clone(&mixer), None, None, None) {
        Ok(s) => _stream = Some(s),
        Err(e) => {
            let _ = app.emit("audio-engine-error", format!("Failed to start audio engine: {e}"));
        }
    }

    while let Ok(msg) = rx.recv() {
        match msg {
            AudioMessage::SetDevice { host, device, buffer_size } => {
                // Release the old stream so the new one can claim the device.
                _stream = None;

                match cpal_audio::start_audio_engine(
                    Arc::clone(&mixer),
                    Some(&host),
                    Some(&device),
                    buffer_size,
                ) {
                    Ok(s) => {
                        _stream = Some(s);
                        last_good = Some((Some(host), Some(device), buffer_size));
                    }
                    Err(e) => {
                        let _ = app.emit(
                            "audio-engine-error",
                            format!("Failed to start audio on {host} / {device}: {e}"),
                        );

                        // Fall back to the last-known-good configuration, then defaults.
                        let (h, d, b) = last_good.clone().unwrap_or((None, None, None));
                        match cpal_audio::start_audio_engine(
                            Arc::clone(&mixer),
                            h.as_deref(),
                            d.as_deref(),
                            b,
                        ) {
                            Ok(s) => _stream = Some(s),
                            Err(e2) => {
                                let _ = app.emit("audio-engine-error", format!("Failed to recover audio engine: {e2}"));
                                if let Ok(s) = cpal_audio::start_audio_engine(Arc::clone(&mixer), None, None, None) {
                                    _stream = Some(s);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mixer_state: SharedMixer = Arc::new(Mutex::new(MixerState::new()));

    let (tx, rx) = std::sync::mpsc::channel::<AudioMessage>();

    let engine_state = AudioEngineState::new(tx);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(mixer_state)
        .manage(engine_state)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let mixer_state_clone = Arc::clone(&app.state::<SharedMixer>());
            std::thread::spawn(move || {
                audio_engine_thread(mixer_state_clone, rx, app_handle);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_channel_params,
            set_master_volume,
            get_meter_levels,
            process_audio_tick,
            get_channel_defaults,
            sync_midi_data,
            set_transport,
            reset_transport,
            get_available_vst3_plugins,
            set_synth_type,
            export_project,
            set_master_limiter_params,
            set_tempo,
            set_synth_params,
            get_synth_presets,
            add_native_plugin,
            set_plugin_param,
            load_sample_to_memory,
            preview_sample,
            scan_sample_folder,
            set_sampler_sample,
            sync_audio_tracks,
            get_sample_waveform,
            trigger_note_on,
            trigger_note_off,
            remove_native_plugin,
            get_plugins,
            get_audio_hosts,
            get_audio_devices,
            set_audio_device,
        ])
        .run(tauri::generate_context!())
        .expect("error while running frost-studio-daw");
}
