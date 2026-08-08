# Frost Studio DAW

[![Build](https://github.com/gabelossless/frost-studio-daw/actions/workflows/build.yml/badge.svg)](https://github.com/gabelossless/frost-studio-daw/actions/workflows/build.yml)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](CHANGELOG.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)

A high-performance digital audio workstation built with **Tauri v2**, **Rust**, and **React**.

Frost Studio pairs a low-latency Rust audio engine with a modern React UI. It
ships with four built-in synthesizers, seven effects, a full mixer, MIDI piano
roll, arrangement view, WAV export, and VST3 plugin hosting on Windows. The same
DSP core is packaged as standalone VST3/CLAP plugins.

**Status:** Early development (v0.1.0) — core DAW features functional.
See the [Roadmap](ROADMAP.md) for what's next.

## Features

### Core DAW
- ✅ Multi-channel mixer (4 channels) with 3-band EQ, pan, volume, mute/solo
- ✅ Sidechain ducking per channel
- ✅ Send buses (Reverb + Delay)
- ✅ Master bus with look-ahead brickwall limiter
- ✅ MIDI piano roll with note sequencing
- ✅ Arrangement view with audio clip tracks
- ✅ Offline WAV export
- ✅ Sample browser
- ✅ Runtime preset loading (JSON files, in-memory fallback)
- ✅ Sample-rate adaptive engine (works at 44.1/48/96 kHz)
- ✅ Real-time audio via CPAL (dedicated thread)
- ⬜ Project save/load (planned for v0.2.0)
- ⬜ MIDI recording & metronome (planned for v0.2.0)

### Instruments
- ✅ **Summit** — Moog-style wavetable synth (ZDF transistor-ladder filter)
- ✅ **Eruption** — Korg MS-20-style subtractive synth (Sallen-Key filters)
- ✅ **Nebula** — wavetable / FM hybrid synth
- ✅ **Sampler** — one-shot and pitched sample playback

### Built-in Effects
- ✅ Compressor, Parametric EQ, Limiter, Bass Enhancer, Delay, Reverb, Distortion

### Plugin Support
- ✅ VST3 plugin **hosting** (Windows)
- ✅ Standalone VST3/CLAP plugins: Compressor, EQ, Limiter, Bass, Delay, Reverb

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Shell | [Tauri v2](https://v2.tauri.app) |
| Frontend | React 19, TypeScript, Vite 6, Tailwind CSS 4 |
| State | Zustand 5 + Zundo (undo/redo) |
| Audio Engine | Rust (CPAL, ringbuf) |
| DSP | Custom (`frost-core` crate) |
| VST3 Hosting | Rust (`vst3` crate, `windows-rs`) |
| Standalone VSTs | Rust (`nih-plug`) |

## Quick Start

```bash
# Install prerequisites: Rust (stable), Node.js 18+
git clone https://github.com/gabelossless/frost-studio-daw
cd frost-studio-daw
npm install
npm run tauri build
```

The unsigned installer will be in `src-tauri/target/release/bundle/`.

See [BUILD.md](BUILD.md) for detailed platform-specific build instructions,
prerequisites, and troubleshooting.

## Distribution

Installers are **unsigned** (code signing costs ~$300–500/yr). To avoid OS
warnings, build from source on your own machine:

```bash
npm install
npm run tauri build
```

For sharing binaries with others, see [BUILD.md → Distribution](BUILD.md#distribution).

## Documentation

- **[User Guide](docs/synths_guide.md)** — the synthesizers, presets, and UI
- **[DSP Reference](docs/dsp_reference.md)** — parameters for every synth and effect
- **[Tauri Command API](docs/tauri_commands.md)** — the Rust ↔ frontend bridge
- **[Developer Guide](devguide.md)** — architecture and systems
- **[Build Guide](BUILD.md)** — build from source on any platform
- **[Roadmap](ROADMAP.md)** — where the project is headed
- **[Changelog](CHANGELOG.md)** — release history

## Project Structure

```
frost-studio-daw/
├── src/                  # React frontend
├── src-tauri/            # Tauri shell + Rust backend
│   ├── src/              #   Rust source (lib.rs, cpal_audio.rs)
│   ├── presets/          #   Generated preset JSON (summit/, eruption/, nebula/)
│   └── tauri.conf.json   #   Tauri configuration
├── frost-core/           # Shared DSP engine crate (synths, effects, mixer)
├── vst/                  # Standalone VST3/CLAP plugins (nih-plug)
├── scripts/              # Build/utility scripts (preset generator)
├── docs/                 # User & reference documentation
└── .github/workflows/    # CI/CD (Windows, macOS, Linux)
```

## Roadmap

v0.2.0 targets project save/load and audio track editing. Runtime preset
loading and sample-rate independence shipped in the Unreleased milestone.
Full milestone details in [ROADMAP.md](ROADMAP.md).

## Contributing

PRs are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for the process, branch
naming, and code style expectations.

## License

MIT — see [LICENSE](LICENSE).
