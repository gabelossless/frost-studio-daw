# Frost Studio DAW

A high-performance digital audio workstation built with Tauri v2, Rust, and React.

**Status:** Early development (v0.1.0) — core DAW features functional.

## Features

- Multi-channel mixer with 3-band EQ, pan, volume, sidechain
- Built-in synths: Summit, Eruption, Nebula, Sampler
- MIDI piano roll with note sequencing
- Arrangement view with audio clip tracks
- Built-in effects: Compressor, EQ, Limiter, Bass, Delay, Reverb, Distortion
- VST3 plugin hosting (Windows)
- Standalone VST plugins (compressor, EQ, limiter, bass, delay, reverb)
- WAV export
- Sample browser
- Real-time audio via CPAL

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
# Install prerequisites: Rust, Node.js
git clone https://github.com/YOUR_USERNAME/frost-studio-daw
cd frost-studio-daw
npm install
npm run tauri build
```

See [BUILD.md](BUILD.md) for detailed build instructions for Windows, macOS, and Linux.

## Build from Source

The app installers are unsigned (code signing costs ~$300-500/yr). To avoid SmartScreen warnings, **build from source** on your own machine:

```bash
npm install
npm run tauri build
```

The unsigned installer will be at `src-tauri/target/release/bundle/`.

## Project Structure

```
frost-studio-daw/
├── src/                  # React frontend
├── src-tauri/            # Tauri shell + Rust backend
│   ├── src/              #   Rust source (lib.rs, cpal_audio.rs)
│   └── tauri.conf.json   #   Tauri configuration
├── frost-core/           # Shared DSP engine crate
├── vst/                  # Standalone VST plugins (nih-plug)
├── scripts/              # Build/utility scripts
├── docs/                 # Additional documentation
└── Installers/           # Pre-built installers (unsigned)
```

## License

MIT
