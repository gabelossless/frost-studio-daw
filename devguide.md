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
- `vst/`: Workspace for compiling standalone Frost effects into VST3/CLAP formats using `nih-plug`.

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

### 4. Drag-and-Drop Workflow
The DAW supports native file dropping (`tauri://drop`). Dropping `.wav` or `.mp3` files into the window automatically creates new audio tracks and arrangement clips, which are then synced to the Rust backend for playback via `symphonia` decoding.

### 5. Native VST3 Hosting & Scanning
Frost Studio hosts external VST3 plugins:
- **Scanner**: Reads bundle `.vst3` structures and parses `moduleinfo.json` to extract full ClassIDs (CIDs) and parameters.
- **Host Engine**: Loads binary binaries via `libloading` into a wrapper that translates frame buffers directly into the native mixer workflow seamlessly.
- **Sandboxing Pipeline**: (Planned) Isolate third-party DLL execution into child processes to safeguard DAW core stability.

## Data Flow

The system is split into three layers that communicate through narrow, well-defined boundaries:

```
┌─────────────────────────┐        ┌──────────────────────────┐
│  React Frontend (src/)  │        │  Rust Backend (src-tauri)│
│                         │ invoke │                          │
│  Zustand store  ────────┼───────▶│  Tauri command handlers  │
│  (useDawStore.ts)       │        │  (src/lib.rs)            │
└─────────────────────────┘        └────────────┬─────────────┘
                                                │ parking_lot::Mutex
                                                ▼
                                     ┌────────────────────────┐
                                     │  MixerState             │
                                     │  (frost-core/src/engine.rs)│
                                     │  channels, synths,      │
                                     │  clock, playlist, fx    │
                                     └────────────┬───────────┘
                                                  │ Arc + Mutex (shared)
                                     ┌────────────▼───────────┐
                                     │  Audio thread (CPAL)    │
                                     │  cpal_audio.rs callback │
                                     │  calls generate_frame() │
                                     └────────────┬───────────┘
                                                  │
                                     ┌────────────▼───────────┐
                                     │  speakers / audio HW   │
                                     └────────────────────────┘
```

### Control path (UI → engine)

1. A React component calls a Zustand store action (e.g. `setTempo`).
2. The action calls `invoke("set_tempo", { tempo })` (the Tauri IPC bridge).
3. The Rust command handler locks `SharedMixer` (an `Arc<parking_lot::Mutex<MixerState>>`)
   and mutates state (e.g. `mixer.clock.set_bpm(tempo)`).
4. The audio thread reads the same shared state on every audio callback, so the
   change takes effect on the next processed frame.

### Audio path (engine → speakers)

1. CPAL opens a stream in a dedicated background thread (`src-tauri/src/lib.rs::run`).
2. The stream callback calls `MixerState::generate_frame()` for each buffer.
3. `generate_frame()` advances the master clock, schedules note-on/off from the
   MIDI playlist, sums synths + audio tracks per channel, runs channel EQ/pan/
   volume/sidechain, applies send effects (Reverb/Delay), then the master limiter.
4. The stereo result is written to the audio device's output buffer.

### Metering path (engine → UI)

1. The audio thread continuously updates per-channel RMS/peak levels inside
   `MixerChannel`.
2. A periodic UI-side `process_audio_tick` command reads meters and emits the
   `meter-levels` event to the frontend.
3. The frontend listens via `listen("meter-levels", ...)` and writes into the
   Zustand `meters` slice, which drives the peak meters and playhead.

## Tauri Commands at a Glance

The full API (signatures, parameters, returns) is documented in
[docs/tauri_commands.md](docs/tauri_commands.md). Quick index:

| Category | Commands |
|----------|----------|
| Mixer | `set_channel_params`, `set_master_volume`, `set_master_limiter_params`, `get_channel_defaults` |
| Transport | `set_transport`, `set_tempo`, `process_audio_tick`, `get_meter_levels` |
| MIDI/Synth | `trigger_note_on`, `trigger_note_off`, `sync_midi_data`, `set_synth_type`, `set_synth_params`, `set_sampler_sample`, `get_synth_presets` |
| Plugins | `add_native_plugin`, `remove_native_plugin`, `get_plugins`, `set_plugin_param`, `get_available_vst3_plugins` |
| Audio device | `get_audio_hosts`, `get_audio_devices`, `set_audio_device` |
| Samples | `scan_sample_folder`, `load_sample_to_memory`, `get_sample_waveform`, `preview_sample` |
| Tracks/Export | `sync_audio_tracks`, `export_project` |

## Testing

```bash
# Rust DSP unit tests (mixer, filters, etc.)
cargo test -p frost-core

# All Rust tests in the workspace
cargo test --workspace

# TypeScript type check
npx tsc --noEmit
```

The CI pipeline (`.github/workflows/build.yml`) runs the TypeScript check, a
`cargo check --workspace`, and produces installers for all three platforms on
every push and pull request.

## Profiling & Performance

The audio hot paths live in `frost-core` (`generate_frame()`, plugin
`process()`, synth `next_sample()`). They must stay **zero-allocation** and
**lock-free**. See [skills.md](skills.md) for the coding rules.

```bash
cargo build --release -p frost-core
```

- **Windows**: profile with Visual Studio Performance Profiler, Windows
  Performance Recorder (WPR), or `perf` via WSL2.
- **macOS**: use Instruments (Time Profiler / Allocations).
- **Linux**: `perf record`/`perf report`, or `cargo flamegraph`.

Common suspects when audio glitches (underruns/clicks):
1. Heap allocations in the audio callback (the #1 cause of jitter).
2. Blocking locks (`std::sync::Mutex`) inside `process()` — use atomics,
   `parking_lot` outside the hot path, or lock-free queues.
3. High-order filters computing coefficients every sample instead of caching
   them on parameter change.
4. Syscalls (I/O, logging) inside the callback.

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
# Compile standalone VST3/CLAP versions of the Frost Suite
cargo build --release -p frost-vst-compressor
cargo build --release -p frost-vst-eq
cargo build --release -p frost-vst-limiter
cargo build --release -p frost-vst-bass
cargo build --release -p frost-vst-delay
cargo build --release -p frost-vst-reverb
# or build them all at once
cargo build --release -p frost-vst-compressor -p frost-vst-eq -p frost-vst-limiter -p frost-vst-bass -p frost-vst-delay -p frost-vst-reverb
```
Output lands in `target/release/*.vst3/` and `target/release/*.clap/`.

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
