# Research & Plan: Custom Synthesizer Implementation

## Overview
Based on the goal of creating three unique synthesizers inspired by classic analog and modern digital hardware (Moog, Korg, Arturia), this document outlines the architecture, sound design philosophy, and implementation strategy.

## 1. Synth Concepts

### A. "Summit" (Inspired by Moog)
- **Sound Profile**: Warm, thick, and harmonically rich.
- **Key Features**:
    - **Dual Oscillators**: Aliasing-free Saw and Square waves with "Unison" detune.
    - **Transistor Ladder Filter**: 24dB/octave low-pass filter with non-linear saturation.
    - **Envelopes**: Snappy, classic ADSR with exponential curves.
- **Unique Difference**: Emphasis on low-end "weight" and smooth resonance.

### B. "Eruption" (Inspired by Korg MS-20)
- **Sound Profile**: Raw, aggressive, and "screaming."
- **Key Features**:
    - **Dual Peak Filter**: Serial High-Pass and Low-Pass filters with high resonance.
    - **Cross-Modulation**: Frequency modulation between oscillators.
    - **External Signal Path**: A "faux" preamp stage that adds grit/distortion.
- **Unique Difference**: Unpredictable resonance and "acidic" textures.

### C. "Nebula" (Inspired by Arturia/Modern Hybrid)
- **Sound Profile**: Expansive, evolving, and crystalline.
- **Key Features**:
    - **Wavetable Engine**: 2D scanning between complex waveforms.
    - **FM Operator**: A secondary sine operator for FM bells and metallic hits.
    - **Multi-mode Filter**: Selectable between Ladder, SEM, and Steiner-Parker styles.
- **Unique Difference**: Built-in effects (Shimmer, Chorus) and automated position scanning.

## 2. Technical Research

### DSP Engine (Rust)
- **Wavetable Implementation**: Use a pre-generated period of 2048 samples. Use linear or cubic interpolation for pitch shifting.
- **Filter Modeling**: Implement a ZDF (Zero-Delay Feedback) ladder filter for the Moog style to avoid the one-sample delay in the feedback loop.
- **Preset Management**: Presets will be stored as JSON files in `src-tauri/presets/`. Each synth will have a dedicated folder with 50-100 `.json` patches.

### UI Design (React)
- **Aesthetic**: Logic Pro X "Industrial Glass" style. Dark translucent panels, high-contrast text.
- **LCD Display**: A central canvas or SVG-based display showing:
    - Real-time waveform view.
    - Filter curve visualization.
    - Active preset name and sound metadata.
- **Touch Interaction**: Large hit areas for knobs and faders. Double-tap to reset.

## 3. Implementation Plan

1.  **Phase 1: Research & Setup** (Current)
    - Define JSON schema for presets.
    - Research specific coefficient formulas for MS-20 and Ladder filters.
2.  **Phase 2: Backend DSP Core**
    - Refactor `dsp/synth.rs` to support multiple synth types.
    - Implement `WavetableOscillator` and `ZDFLadderFilter`.
3.  **Phase 3: Preset Generation**
    - Write a script to generate 50 seed presets for each synth by varying parameters.
4.  **Phase 4: Frontend UI**
    - Build the "LCD" display component.
    - Create a "Synth Selector" in the UI.
5.  **Phase 5: Testing & Review**
    - Verify CPU usage (8-voice polyphony limit).
    - Refactor for performance (SIMD for filters if needed).

## 4. Preset & Sound Design Guide
To add more presets:
1. Create a new JSON file in `src-tauri/presets/[synth_name]/`.
2. Follow the required structure (Osc, Filter, Env, FX params).
3. The engine will automatically load any file ending in `.json`.
