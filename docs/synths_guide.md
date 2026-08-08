# Frost Studio: Custom Synthesizer User Guide

## Introduction
Frost Studio now includes three unique, high-fidelity synthesizers inspired by legendary hardware. Each synth has a distinct sound profile and dedicated internal DSP.

## 1. The Synthesizers

### Summit (High-Peaked Ladder)
- **Inspiration**: Moog Transistor Ladder.
- **Sound**: Rich, warm, smooth. Ideal for fat bass and classic leads.
- **Controls**: Focused on the 4-pole ZDF Ladder filter.

### Eruption (Raw Dual-Peak)
- **Inspiration**: Korg MS-20.
- **Sound**: Aggressive, screaming, and punchy. Ideal for industrial textures and acid bass.
- **Controls**: High-resonance series HPF and LPF.

### Nebula (Crystalline Hybrid)
- **Inspiration**: Arturia/Modern Wavetable.
- **Sound**: Evolving, digital, and expansive. Ideal for pads and metallic bells.
- **Controls**: Wavetable scanning and FM modulation.

### Sampler (Sample Player)
- **Inspiration**: Modern beat samplers.
- **Sound**: Plays audio files back from the sample bank.
- **Controls**: Load a sample via the Sample Browser, then trigger it from the
  piano roll. Pitched playback follows the note you play; one-shot behavior for
  percussive material.
- **Supported formats**: WAV, MP3, OGG, FLAC (decoded by `symphonia`).

## 1.5 Built-in Effects

Frost Studio ships with seven insert effects that can be chained on any mixer
channel. See [docs/dsp_reference.md](dsp_reference.md) for the full parameter
list and value ranges.

| Effect | What it does |
|--------|-------------|
| **Compressor** | VCA-style gain reduction; controls threshold, ratio, attack, release, makeup |
| **Parametric EQ** | 5-band EQ (low shelf, 3 peaking, high shelf) with freq/gain/Q per band |
| **Limiter** | Peak ceiling with soft-knee style limiting |
| **Bass Enhancer** | Sub-bass synth/enhancer for low-end weight |
| **Delay** | Tempo-friendly echo with feedback |
| **Reverb** | Stereo reverb for space and depth |
| **Distortion / Saturator** | Harmonic saturation and drive |

Add effects from the **Plugin Insert** panel on any channel. Effects process in
order top-to-bottom.

## 2. Preset Management
Each synth comes with **50 factory presets** defined in `frost-core/src/dsp/presets.rs`. Corresponding JSON presets are also generated into `src-tauri/presets/` by the build script.

### Adding Your Own Presets
1. Edit `frost-core/src/dsp/presets.rs` to add new entries to `init_summit()`, `init_eruption()`, or `init_nebula()`.
2. Alternatively, add a new `.json` file in `src-tauri/presets/[synth_name]/` following the format below, then run `node scripts/generate_presets.cjs` to regenerate.
3. Use the following parameter structure:
```json
{
  "name": "My Lead",
  "category": "Lead",
  "params": [0.05, 0.2, 0.7, 0.4]
}
```

## 3. UI Interaction
The **Synth LCD** at the top of the mixer provides real-time visual feedback:
- **Waveform Analyzer**: Displays the output frequency spectrum.
- **LCD Display**: Shows voice counts, sample rate, and active patch information.
- **Touch Controls**: Tap the synth name tabs to swap models instantly.
