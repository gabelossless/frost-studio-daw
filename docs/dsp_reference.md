# DSP Reference — Synths & Effects

This document is the authoritative reference for the parameters of every
instrument and effect in Frost Studio's `frost-core` DSP engine.

> Source of truth: the Rust code in `frost-core/src/dsp/`. Parameter IDs are the
> `u32` index used by the `AudioPlugin::set_param`/`get_param` interface, which
> the frontend calls via the [Tauri command API](tauri_commands.md).

---

## Instruments

All synths share a common ADSR envelope plus a per-synth signal path. The active
synth on a channel is selected with `set_synth_type`. Polyphony is currently
unlimited (a per-voice budget is a v0.3.0 roadmap item).

### Shared: ADSR Envelope

| Param | Range | Notes |
|-------|-------|-------|
| Attack | seconds | Time to reach peak level |
| Decay | seconds | Time to fall to sustain level |
| Sustain | 0.0–1.0 | Level held while key is down |
| Release | seconds | Time to fall to zero after key-up |

### Summit — Moog-style Wavetable Synth

File: `frost-core/src/dsp/synths/summit.rs`

| Stage | Implementation | Details |
|-------|---------------|---------|
| Oscillators | 2× 2048-sample wavetable saws | Osc 2 detuned +0.2 semitone (~1.002×) for unison thickness |
| Filter | ZDF transistor-ladder (24 dB/oct) | Cutoff modulated by envelope + LFO; resonance fixed at 0.4 |
| LFO | Sine, 0.5 Hz | Modulates cutoff ±10% |
| Envelope | ADSR | Also scales output level |

Cutoff follows `(env * 5000 + 200) * (1 + lfo*0.1)`, clamped to 20 Hz–20 kHz.

### Eruption — Korg MS-20-style Synth

File: `frost-core/src/dsp/synths/eruption.rs`

| Stage | Implementation | Details |
|-------|---------------|---------|
| Oscillators | 2× wavetable saws | Osc 2 tuned a fifth (1.5×) above Osc 1 |
| Filter | Serial Sallen-Key HPF → LPF | HPF: `100 + env*1000` Hz, res 0.5; LPF: `1000 + env*8000` Hz, res 0.8 |
| Envelope | ADSR | Envelope drives filter sweeps + output |

Produces the aggressive, "screaming" high-resonance character of the MS-20.

### Nebula — Wavetable / FM Hybrid

File: `frost-core/src/dsp/synths/nebula.rs`

| Stage | Implementation | Details |
|-------|---------------|---------|
| Oscillators | 2 wavetable tables (sine, saw) | Crossfaded by `scan_pos` |
| FM Operator | Sine at 2nd harmonic (2× freq) | `fm_amount` controls modulation depth |
| Scan | `scan_pos` 0.0–1.0 | Blends between tables for evolving timbres |
| Envelope | ADSR | Output level |

> Note: FM is currently applied as an AM/crossfade blend in `next_sample()`.
> Full per-sample phase modulation is a planned refinement.

### Sampler — Sample Player

File: `frost-core/src/dsp/synths/sampler.rs`

| Feature | Details |
|---------|---------|
| Sample source | `GLOBAL_SAMPLE_BANK` (loaded via `load_sample_to_memory`) |
| Pitch | `2^((pitch-60)/12)` — middle C (60) plays at original speed |
| Interpolation | Linear between frames |
| Channels | Stereo averaged to mono; interleaved `[L,R,L,R...]` |
| Envelope | ADSR shapes the playback |
| Formats | WAV, MP3, OGG, FLAC (via `symphonia`) |

Set the active sample with `set_sampler_sample`.

---

## Effects (Insert Plugins)

Effects implement the `AudioPlugin` trait
(`frost-core/src/dsp/plugins/mod.rs`). Each has a `name()`, a per-sample
`process(l, r)`, and parameter access via integer IDs. They are added to a
channel's insert chain with `add_native_plugin`.

### Compressor

File: `frost-core/src/dsp/plugins/compressor.rs`

| ID | Param | Default | Range |
|----|-------|---------|-------|
| 0 | Threshold | 0.5 | ~0.0–1.0 (amplitude) |
| 1 | Ratio | 4.0 | ≥1.0 (higher = more reduction) |
| 2 | Attack (s) | 0.01 | 0.001–0.5 |
| 3 | Release (s) | 0.1 | 0.01–1.0 |
| 4 | Makeup gain | 1.0 | 0.0–4.0 |

VCA-style: envelope follows peak of `(|L|+|R|)/2`, gain reduction computed in dB.

### Parametric EQ

File: `frost-core/src/dsp/plugins/eq.rs`

5 bands: **Low Shelf → Peaking → Peaking → Peaking → High Shelf**.

| ID | Param | Range |
|----|-------|-------|
| `band*3 + 0` | Frequency (Hz) | 20–20000 |
| `band*3 + 1` | Gain (dB) | −24–+24 |
| `band*3 + 2` | Q | 0.1–10 |

Band index 0–4 maps to defaults 100 / 400 / 1000 / 4000 / 10000 Hz.

### Limiter

File: `frost-core/src/dsp/plugins/limiter.rs`

| ID | Param | Default | Range |
|----|-------|---------|-------|
| 0 | Threshold | 1.0 | 0.01–1.0 |
| 1 | Ceiling | 0.95 | 0.5–1.0 |
| 2 | Attack (s) | 0.001 | 0.0001–0.1 |
| 3 | Release (s) | 0.05 | 0.01–2.0 |

Brickwall: when the envelope exceeds threshold, gain = `threshold/envelope`;
output is clamped to ±ceiling.

### Bass Enhancer / Sub-Synth

File: `frost-core/src/dsp/plugins/bass.rs`

| ID | Param | Default | Range |
|----|-------|---------|-------|
| 0 | Pitch / Freq (Hz) | 440 | note frequency (sub osc at ×0.5) |
| 1 | Filter cutoff (Hz) | — | 20–5000 |
| 2 | Trigger (gate) | 0 | 0–1 (edge-triggers envelope) |

A saw + sub-sine wavetable pair through a ladder filter, mixed parallel with the
input. Behaves as a sub-harmonic enhancer/solo bass layer.

### Delay

File: `frost-core/src/dsp/plugins/delay.rs`

| ID | Param | Default | Range |
|----|-------|---------|-------|
| 0 | Time (s) | 0.5 | 0.01–2.0 |
| 1 | Feedback | 0.3 | 0.0–0.95 |
| 2 | Mix | 0.5 | 0.0–1.0 |

Circular-buffer echo with feedback and wet/dry mix.

### Reverb

File: `frost-core/src/dsp/plugins/reverb.rs`

Freeverb-style comb + all-pass network.

| ID | Param | Default | Range |
|----|-------|---------|-------|
| 0 | Room size | 0.7 | 0.0–0.98 |
| 1 | Damping | 0.2 | 0.0–1.0 |
| 2 | Mix | 0.3 | 0.0–1.0 |

### Distortion / Saturator

File: `frost-core/src/dsp/plugins/distortion.rs`

| ID | Param | Default | Range |
|----|-------|---------|-------|
| 0 | Drive | 1.0 | 1.0–12.0 (≈ +21 dB) |
| 1 | Mix | 1.0 | 0.0–1.0 |
| 2 | Mode | Soft | 0=Soft (atan), 1=Hard (clamp), 2=Tape (cubic) |

---

## Send Effects

The mixer has two global send buses fed post-fader from every channel
(`send_1_amount` → Reverb, `send_2_amount` → Delay). These are the global
versions in `frost-core/src/dsp/effects.rs`.

| Effect | Params |
|--------|--------|
| **Delay** | Time (1–2000 ms, default 300), Feedback (0–0.95, default 0.4), Mix (0–1, default 0.5) |
| **Reverb** | Mix (0–1, default 0.5); four parallel delays at 29/43/61/83 ms |

---

## Mixer

Per channel (`frost-core/src/dsp/mixer.rs`):

| Field | Range | Notes |
|-------|-------|-------|
| `volume` | 0.0–1.0 | Linear gain |
| `pan` | −1.0–1.0 | Constant-power law |
| `muted` / `soloed` | bool | — |
| `eq_low/mid/high` | freq 20–20000, gain −24–+24 dB, Q 0.1–10 | Low shelf 100 Hz, peaking 1 kHz, high shelf 8 kHz |
| `send_1_amount` / `send_2_amount` | 0.0–1.0 | Reverb / Delay sends |
| `sidechain_source_id` | channel id or null | Duck against this channel |
| `sidechain_ratio` | 0.0–1.0 | Ducking depth |

Master bus (`MasterBus`) applies a brickwall limiter then master volume; exposes
RMS + peak metering for all channels and the master.

---

## Clock & MIDI

`frost-core/src/dsp/midi.rs`

- `MasterClock`: tick-based, **960 PPQ** (pulses per quarter note), tempo in BPM.
- `NoteEvent`: `{ channel_id, pitch, velocity (0–1), start_tick, duration_ticks }`.
- `MidiPlaylist`: holds notes, sorts by `start_tick` for sequenced playback.
- Playhead position in beats = `current_tick / 960`.
