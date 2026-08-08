# Tauri Command API Reference

Every Tauri command exposed by the Rust backend (`src-tauri/src/lib.rs`) to the
React frontend, with signatures, parameters, and return values.

> The frontend calls these via `invoke("command_name", { args })` from
> `@tauri-apps/api/core`. Types use Rust notation. All commands operate on the
> shared `SharedMixer = Arc<parking_lot::Mutex<MixerState>>` unless noted.

---

## Mixer

### `set_channel_params`

Apply full parameter state to one mixer channel.

```
fn set_channel_params(params: ChannelParams) -> Result<(), String>
```

`ChannelParams`:

| Field | Type | Notes |
|-------|------|-------|
| `channel_id` | `usize` | Must be < `NUM_CHANNELS` (4) |
| `volume` | `f32` | 0.0–1.0 |
| `pan` | `f32` | −1.0–1.0 |
| `muted` | `bool` | |
| `soloed` | `bool` | |
| `eq_low`/`eq_mid`/`eq_high` | `EqBandParams` | `{ freq_hz, gain_db, q }` |
| `send_1_amount` | `f32` | Reverb send 0–1 |
| `send_2_amount` | `f32` | Delay send 0–1 |
| `sidechain_source_id` | `Option<usize>` | Channel to duck against |
| `sidechain_ratio` | `f32` | 0.0–1.0 ducking depth |

### `set_master_volume`

```
fn set_master_volume(volume: f32) -> Result<(), String>
```

Volume clamped to 0.0–1.5.

### `set_master_limiter_params`

```
fn set_master_limiter_params(threshold: f32, ceiling: f32, attack_ms: f32, release_ms: f32) -> Result<(), String>
```

### `get_channel_defaults`

Returns default `ChannelParams` for all channels so the UI can initialize.

```
fn get_channel_defaults() -> Vec<ChannelParams>
```

---

## Transport

### `set_transport`

Start or stop playback.

```
async fn set_transport(playing: bool) -> Result<(), String>
```

Stopping resets the MIDI cursor, clears active notes, and resets all synths.

### `set_tempo`

```
fn set_tempo(tempo: f32) -> Result<(), String>
```

Sets BPM on the master clock.

### `get_meter_levels`

Current RMS/peak levels for all channels + master.

```
fn get_meter_levels() -> Vec<MeterLevel>
```

`MeterLevel`: `{ channel_id, rms_left, rms_right, peak_left, peak_right, playhead_beats }`.
Master channel id is `255`.

### `process_audio_tick`

Emit `meter-levels` event with current meters + playhead position.

```
async fn process_audio_tick() -> Result<(), String>
```

Frontend subscribes with `listen("meter-levels", ...)`.

---

## MIDI & Synths

### `trigger_note_on`

```
fn trigger_note_on(channel_id: usize, note: u8, velocity: u8) -> Result<(), String>
```

`velocity` is normalized to 0–1 (`velocity / 127.0`).

### `trigger_note_off`

```
fn trigger_note_off(channel_id: usize, note: u8) -> Result<(), String>
```

### `sync_midi_data`

Replace the playlist's notes with a new sequence.

```
async fn sync_midi_data(notes: Vec<NoteEvent>) -> Result<(), String>
```

`NoteEvent`: `{ channel_id, pitch, velocity, start_tick, duration_ticks }`.
Resets the playback cursor.

### `set_synth_type`

```
fn set_synth_type(channel_id: usize, synth_type: String) -> Result<(), String>
```

`synth_type` ∈ `"Summit" | "Eruption" | "Nebula" | "Sampler"`.

### `set_synth_params`

Apply ADSR params to **all** synths.

```
fn set_synth_params(params: SynthParams) -> Result<(), String>
```

`SynthParams`: `{ attack, decay, sustain, release }` (all `f32`).

### `set_sampler_sample`

```
fn set_sampler_sample(channel_id: usize, path: String) -> Result<(), String>
```

### `get_synth_presets`

```
fn get_synth_presets(synth_type: String) -> Result<Vec<SynthPreset>, String>
```

`synth_type` ∈ `"Summit" | "Eruption" | "Nebula"`. Returns the factory bank.
`SynthPreset`: `{ name, category, params: Vec<f32> }`.

---

## Plugins

### `get_available_vst3_plugins`

Scan the system VST3 directories and list discoverable plugins.

```
fn get_available_vst3_plugins() -> Vec<VstPluginInfo>
```

Windows-only (scans standard VST3 locations, parses `moduleinfo.json`).

### `add_native_plugin`

Append a native insert effect to a channel.

```
fn add_native_plugin(channel_id: usize, plugin_type: String) -> Result<Vec<String>, String>
```

`plugin_type` ∈ `"Compressor" | "EQ" | "Limiter" | "Bass" | "Delay" | "Reverb" | "Distortion" | "Saturator"`.
Returns the new list of insert plugin names.

### `remove_native_plugin`

```
fn remove_native_plugin(channel_id: usize, index: usize) -> Result<Vec<String>, String>
```

### `get_plugins`

```
fn get_plugins(channel_id: usize) -> Result<Vec<String>, String>
```

Names of the channel's inserts.

### `set_plugin_param`

```
fn set_plugin_param(channel_id: usize, plugin_index: usize, param_id: u32, value: f32) -> Result<(), String>
```

`param_id`/`value` mapping is documented per-effect in [dsp_reference.md](dsp_reference.md).

---

## Audio Device

### `get_audio_hosts`

```
fn get_audio_hosts() -> Vec<String>
```

Available CPAL host names.

### `get_audio_devices`

```
fn get_audio_devices(host: String) -> Result<Vec<String>, String>
```

### `set_audio_device`

Send a `SetDevice` message to the audio thread to switch host/device/buffer.

```
fn set_audio_device(host: String, device: String, buffer_size: Option<u32>) -> Result<(), String>
```

---

## Samples & Files

### `scan_sample_folder`

Recursively list audio files. Defaults to `<cwd>/FrostSamples` if no path given
(creates the folder if missing).

```
fn scan_sample_folder(path: Option<String>) -> Vec<SampleNode>
```

`SampleNode`: `{ name, path, isDir, children? }`. Only WAV/MP3/OGG/FLAC are listed.

### `load_sample_to_memory`

Decode an audio file into the global sample bank (CPU-intensive, runs native).

```
async fn load_sample_to_memory(path: String) -> Result<AudioSample, String>
```

### `get_sample_waveform`

Downsample a loaded sample to `buckets` peak amplitudes for waveform display.

```
fn get_sample_waveform(path: String, buckets: u32) -> Result<Vec<f32>, String>
```

### `preview_sample`

> Currently a no-op stub (returns `Ok(())`). Preview bus is a roadmap item.

```
fn preview_sample(_path: String) -> Result<(), String>
```

---

## Tracks & Export

### `sync_audio_tracks`

Replace the engine's audio track list (arrangement clips).

```
fn sync_audio_tracks(tracks: Vec<AudioTrack>) -> Result<(), String>
```

### `export_project`

Offline-render the project to a WAV file.

```
async fn export_project(path: String, duration_beats: f32) -> Result<(), String>
```

Uses `Exporter::export_to_wav` in `frost-core`.

---

## Events

| Event name | Payload | Direction |
|-----------|---------|-----------|
| `meter-levels` | `Vec<MeterLevel>` | Rust → Frontend (from `process_audio_tick`) |

## File-drop events

| Event name | Payload | Direction |
|-----------|---------|-----------|
| `tauri://file-drop` | `{ paths: string[] }` | Frontend listens to create audio tracks |
| `tauri://drag-drop` | `{ paths: string[] }` | Same handler (Tauri v2 variant) |

---

## Adding a New Command

1. Define an `async fn` (or sync `fn`) with `#[tauri::command]` in
   `src-tauri/src/lib.rs`.
2. Register it in the `invoke_handler(tauri::generate_handler![ ... ])` list.
3. If it needs shared state, add `state: State<'_, SharedMixer>` (or
   `AudioEngineState`) as a parameter.
4. Document it here with its signature, parameters, and return type.
