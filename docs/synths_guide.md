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

## 2. Preset Management
Each synth comes with **50 factory presets** found in `src-tauri/presets/`.

### Adding Your Own Presets
1. Navigate to `src-tauri/presets/[synth_name]/`.
2. Create a new `.json` file.
3. Use the following format:
```json
{
  "name": "My Lead",
  "category": "Lead",
  "cutoff": 0.5,
  "resonance": 0.3,
  "attack": 0.05,
  "decay": 0.2,
  "sustain": 0.7,
  "release": 0.4
}
```
The engine will automatically scan and load these files on startup.

## 3. UI Interaction
The **Synth LCD** at the top of the mixer provides real-time visual feedback:
- **Waveform Analyzer**: Displays the output frequency spectrum.
- **LCD Display**: Shows voice counts, sample rate, and active patch information.
- **Touch Controls**: Tap the synth name tabs to swap models instantly.
