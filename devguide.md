# Frost Studio DAW - Developer Guide

## Introduction
**Frost Studio DAW** is a professional-grade, hybrid Digital Audio Workstation built using **Tauri**, **Rust**, and **React**. It is designed for high-performance audio production, featuring a custom native DSP engine, a suite of built-in instruments and effects, and a modern, high-fidelity user interface.

## Core Architecture
The project follows a hybrid architecture to balance UI flexibility with low-latency audio performance:

- **Frontend (UI Library)**: Built with **React** and **Vite**. It handles the arrangement view, mixer UI, piano roll, and preset browsing. State is managed via **Zustand** (with **Zundo** for undo/redo).
- **Backend (Audio Engine)**: Driven by **Rust** and **Tauri**. It utilizes **cpal** for cross-platform audio I/O, managed in a dedicated background thread in `src-tauri/src/lib.rs` to maintain continuous stream streaming bypassing Send/Sync bottlenecks.
- **Shared DSP Core**: The `frost-core` library contains the mathematical definitions for oscillators, filters, and effects, ensuring consistency between the DAW and standalone VST plugins.
- **VST3 Host Engine**: Powered by `libloading`, bridging native VST3 DLLs into the AudioPlugin trait bridge dynamically on startup.

## Directory Map
- `src/`: React frontend source code.
    - `components/`: UI components (Arrangement, Mixer, Piano Roll, etc.).
    - `store/`: Zustand state management (`useDawStore.ts`).
    - `presets/`: JSON preset banks for internal synths.
- `src-tauri/`: Rust backend source code.
    - `src/lib.rs`: Tauri command definitions and event emitting.
    - `src/cpal_audio.rs`: Native audio driver implementation.
- `frost-core/`: The "Brain" of the project. Contains shared DSP logic used by both the DAW and the VST3 export suite.
- `src-vst/`: Workspace for compiling standalone Frost effects into VST3/CLAP formats using `nih_plug`.
- `vst/`: Target directory for compiled plugin binaries.

## Key Systems

### 1. Audio Engine & Transport
The audio engine runs on a dedicated high-priority thread. The playhead position is tracked in beats and synchronized with the UI via the `meter-levels` event. A browser-fallback mode exists for web-only development, simulating playhead movement via `setInterval`.

### 2. Mixer & Plugin Chain
Each channel in the mixer has a serialized `MixerChannel` state. This includes volume, pan, and an internal "Insert Chain" where native plugins (Compressor, EQ, Limiter, etc.) are processed sequentially.

### 3. Instruments
Frost Studio features several core synthesis engines:
- **Summit**: Wavetable synthesizer.
- **Eruption**: Analog-modeled subtractive synth.
- **Nebula**: Phase modulation / FM hybrid.
- **Sampler**: High-performance one-shot and pitched sampler.
- **PRO-TRAP NEO**: Specialized trap workstation with premium preset banks.

### 4. Drag-and-Drop Workflow
The DAW supports native file dropping (`tauri://drop`). Dropping `.wav` or `.mp3` files into the window automatically creates new audio tracks and arrangement clips, which are then synced to the Rust backend for playback via `symphonia` decoding.

### 5. Native VST3 Hosting & Scanning
Frost Studio hosts external VST3 plugins:
- **Scanner**: Reads bundle `.vst3` structures and parses `moduleinfo.json` to extract full ClassIDs (CIDs) and parameters.
- **Host Engine**: Loads binary binaries via `libloading` into a wrapper that translates frame buffers directly into the native mixer workflow seamlessly.
- **Sandboxing Pipeline**: (Planned) Isolate third-party DLL execution into child processes to safeguard DAW core stability.

## Development Workflow

### Requirements
- **Node.js**: v18+ 
- **Rust**: Latest stable (cargo/rustc)
- **Tauri CLI**: `cargo install tauri-cli`

### Running the App
```bash
# Start Vite and Tauri in dev mode (port 3001)
npm run dev
```

### Compiling VST3 Plugins
```bash
# Compile standalone VST3 versions of the Frost Suite
cargo build --release -p frost-compressor # (and so on)
```

### Production Bundling (Installers)
```bash
# Generate .msi and .exe installers Node flawless
npm run tauri build
```
Binaries will be placed inside `src-tauri/target/release/bundle/msi/` or `nsis/`.
The **`.msi`** file is the recommended release distribution for Windows users.

## Maintenance Note: Git & Submodules
> [!IMPORTANT]
> The `frost-core` directory is treated as a submodule/separate crate. If you encounter issues during `git add`, ensure that `frost-core` has a valid commit checked out or is properly ignored if you are initializing a new parent repository from scratch.

---
*Created by Frost Studio Senior Dev Agents - 2026*
