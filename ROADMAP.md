# Frost Studio DAW — Roadmap

This document tracks where Frost Studio is going. Milestones are ordered by
dependencies and rough priority, not by commitment to dates. Each release
targets a stable, usable increment.

Legend:
- ✅ **Done** — shipped in a release
- 🚧 **In progress** — actively worked on
- ⬜ **Planned** — scoped, not started
- 💡 **Backlog** — idea, needs specification

---

## v0.1.0 — Core DAW (Current)

> Released as the initial codebase. Establishes the engine, mixer, instruments,
> and plugin bridge.

| Area | Item | Status |
|------|------|--------|
| Engine | Real-time audio via CPAL on a dedicated thread | ✅ |
| Engine | `frost-core` shared DSP library (DAW + standalone VSTs) | ✅ |
| Mixer | 4-channel mixer: 3-band EQ, pan, volume, mute/solo | ✅ |
| Mixer | Sidechain ducking per channel | ✅ |
| Mixer | Send buses (Reverb + Delay) | ✅ |
| Mixer | Master bus with look-ahead brickwall limiter | ✅ |
| Instruments | Summit (Moog-style wavetable), Eruption (MS-20-style), Nebula (FM/wavetable hybrid), Sampler | ✅ |
| Effects | Compressor, Parametric EQ, Limiter, Bass enhancer, Delay, Reverb, Distortion | ✅ |
| MIDI | Piano roll with note sequencing | ✅ |
| Arrangement | Audio clip tracks with beat-synced playback | ✅ |
| Export | Offline WAV rendering | ✅ |
| Plugins | Native insert chain per channel | ✅ |
| Plugins | VST3 scanning + hosting (Windows) | ✅ |
| Standalone | 6 VST3/CLAP plugin binaries (compressor, EQ, limiter, bass, delay, reverb) | ✅ |
| UI | Mixer, Piano Roll, Arrangement, Instrument, Plugin Manager, Sample Browser, Visualizer | ✅ |
| State | Undo/redo via Zundo temporal middleware | ✅ |

---

## v0.2.0 — Editing & Workflow

> Target: make the DAW usable for real recording sessions.

| Item | Status | Notes |
|------|--------|-------|
| Project save/load (`.frost` files) | ⬜ | Serialize tracks/clips/notes/params to disk; persist across restarts |
| Audio track editing | 🚧 | Clipping, trimming, fades in the Arrangement view |
| Sampler multi-sample support | ⬜ | Velocity layers + round-robin |
| Sample preview bus | ⬜ | `preview_sample` is currently a no-op in `lib.rs` |
| Preset file loading at runtime | ⬜ | Load `src-tauri/presets/**` JSON instead of only in-memory bank |
| Tempo & time-signature automation | ⬜ | Beat-mapped transport changes |
| Sample-rate agnostic engine | ⬜ | Replace hardcoded 44100 Hz with the device's actual rate |
| MIDI recording from a keyboard | ⬜ | `isRecording` flag exists in state; wire capture path |
| Metronome | ⬜ | `metronomeEnabled` flag exists; needs click source |

---

## v0.3.0 — Automation & MIDI

| Item | Status | Notes |
|------|--------|-------|
| Parameter automation lanes | ⬜ | Draw curves for volume, pan, filter cutoff, effects |
| Clip-level MIDI editing | ⬜ | Move/resize/copy notes, snap to grid |
| Piano roll tools | ⬜ | Quantize, humanize, velocity editing, scale highlighting |
| MIDI CC mapping | ⬜ | External controller → synth parameters |
| Per-voice polyphony limit | ⬜ | 8-voice budget per synth, voice stealing |
| Performance metering | ⬜ | CPU/RAM usage, per-plugin meter |

---

## v0.5.0 — VST Ecosystem

| Item | Status | Notes |
|------|--------|-------|
| VST3 hosting on macOS/Linux | ⬜ | Currently Windows-only; use `vst3-sys`/`nih-plug` host path |
| Plugin sandboxing | 💡 | Isolate third-party DLLs in child processes |
| AU hosting (macOS) | 💡 | Optional, after VST3 parity |
| CLAP hosting | 💡 | Native CLAP host in addition to VST3 |
| External plugin GUI embedding | 💡 | Requires host-UI bridges (needs a more mature host crate) |

---

## v1.0.0 — Production Ready

> Target: a stable, distributable DAW suitable for general use.

| Item | Status | Notes |
|------|--------|-------|
| Project format stability | ⬜ | Backward-compatible `.frost` schema |
| Cross-platform parity | ⬜ | Feature parity across Windows/macOS/Linux |
| Undo/redo for all mutations | ⬜ | Verify Zundo coverage across every store action |
| Crash-safe audio thread | ⬜ | Panic containment, graceful stream restart |
| Code signing strategy | ⬜ | Document paid + free options for distribution |
| Docs & onboarding | ⬜ | First-run tour, keyboard shortcuts reference |
| Automated DSP tests | 🚧 | Expand `#[cfg(test)]` coverage in `frost-core` |

---

## Beyond v1.0 (Backlog)

- **AAX support** — Pro Tools format requires a C++/JUCE wrapper around `frost-core`
- **Track freeze/bounce** — offline render of tracks to audio
- **Plugin GUI framework** — replace per-plugin egui with a unified host UI
- **Cloud collaboration** — shared projects (requires backend infrastructure)
- **Score notation view** — MIDI as sheet music
- **VST instrument hosting** — load third-party synths, not just effects

---

## How Milestones Are Decided

1. **Dependencies first** — sample-rate plumbing (v0.2.0) unlocks automation
   accuracy (v0.3.0).
2. **User-visible wins** — project save/load is the #1 blocker for real use.
3. **Risk reduction** — cross-platform VST hosting is the biggest unknown, so
   it gets its own milestone with room to spike.
4. **Community input** — feature requests move items up from Backlog.

---

## Versioning

- Follows [Semantic Versioning](https://semver.org/).
- See [CHANGELOG.md](CHANGELOG.md) for release notes.
