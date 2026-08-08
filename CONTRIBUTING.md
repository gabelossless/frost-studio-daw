# Contributing to Frost Studio DAW

Thanks for your interest in contributing! This guide covers the workflow,
standards, and expectations for the Frost Studio DAW repository.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Ways to Contribute](#ways-to-contribute)
- [Development Setup](#development-setup)
- [Project Layout](#project-layout)
- [Branch Naming](#branch-naming)
- [Commit Conventions](#commit-conventions)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

Be respectful and constructive. This is a small, community-driven audio project —
harassment of any kind is not tolerated. If you have concerns, open an issue.

## Ways to Contribute

- **Bug reports** — open an issue with steps to reproduce, expected vs. actual
  behavior, and your OS/audio device.
- **Feature requests** — open an issue describing the use case; it will be
  triaged onto the [roadmap](ROADMAP.md).
- **Code** — bug fixes, DSP improvements, new effects/instruments, UI polish.
- **Documentation** — this repo lives and dies by its docs; fixes are welcome.
- **Presets & sound design** — see [docs/synths_guide.md](docs/synths_guide.md).

## Development Setup

Prerequisites: Node.js 18+ (20 LTS recommended), Rust **stable** toolchain.

```bash
git clone https://github.com/gabelossless/frost-studio-daw
cd frost-studio-daw
npm install
npm run dev        # Vite dev server (port 3001) + Tauri window with HMR
```

For full platform prerequisites, see [BUILD.md](BUILD.md).

## Project Layout

| Path | Purpose |
|------|---------|
| `src/` | React frontend (components, Zustand store) |
| `src-tauri/` | Tauri shell + Rust backend (CPAL audio engine, Tauri commands) |
| `frost-core/` | Shared DSP crate (synths, effects, mixer, MIDI, export) |
| `vst/` | Standalone VST3/CLAP plugin crates (`nih-plug`) |
| `scripts/` | Build utilities (preset generation) |
| `docs/` | User + reference documentation |

## Branch Naming

Prefix branches with a type so PRs are self-describing:

```
feat/<description>      New feature
fix/<description>       Bug fix
docs/<description>      Documentation
refactor/<description>  Code change with no behavior change
test/<description>      Adding or fixing tests
perf/<description>      Performance improvement
ci/<description>        CI/CD changes
```

Example: `feat/project-save-load`.

## Commit Conventions

Use concise, imperative-mood commit messages. Follow the
[Conventional Commits](https://www.conventionalcommits.org/) format when it adds
clarity:

```
feat: add project save/load (.frost files)
fix(mixer): correct sidechain RMS gain path
docs(README): document distribution options
test(dsp): cover limiter edge cases
```

Keep commits focused on a single logical change. Rebase rather than merge when
integrating upstream changes.

## Pull Request Process

1. **Check the roadmap** — if the change is significant, confirm it isn't
   already planned or claimed in an issue.
2. **Branch from `main`** using the naming rules above.
3. **Write or update tests** for the change (see [Testing](#testing)).
4. **Run the checks** locally:
   ```bash
   npx tsc --noEmit            # TypeScript type check
   cargo check --workspace      # Rust compilation
   cargo test -p frost-core     # DSP unit tests
   ```
5. **Update documentation** — if behavior, commands, or directory structure
   change, update the relevant `.md` files in the same PR.
6. **Open the PR** — describe what changed and why. Reference related issues.
7. **Address review feedback** — small review iterations are expected.

## Code Style

- **Rust**: follow `rustfmt` and `cargo clippy`. Prefer `edition = 2021`.
  - Audio hot paths must be **zero-allocation** and **lock-free** — no
    `Vec::new()`, `Box::new()`, `String::from()`, or `std::sync::Mutex` inside
    `process()` / `generate_frame()`. See [skills.md](skills.md).
  - Keep DSP state explicit; prefer simple structs over clever abstractions.
- **TypeScript/React**: strict mode is on. Use granular Zustand selectors
  (`useDawStore(state => state.meters)`) rather than destructuring the whole
  store. Prefer canvas/rAF for high-frequency visuals.
- **Formatting**: run the formatters for the files you touch.

## Testing

- Rust DSP unit tests live in `#[cfg(test)]` modules next to the code
  (`cargo test -p frost-core`).
- The CI pipeline runs `npx tsc --noEmit`, `cargo check --workspace`, and builds
  installers for Windows, macOS, and Linux. See `.github/workflows/build.yml`.

## Documentation

Docs live in the repo root and `docs/`. When you change code, keep these in sync:

- `README.md` — features, quick start, structure
- `BUILD.md` — build steps and troubleshooting
- `devguide.md` — architecture for developers
- `docs/dsp_reference.md` — synth/effect parameters
- `docs/tauri_commands.md` — the Rust ↔ frontend command API

Add a `CHANGELOG.md` entry under **Unreleased** for user-visible changes.

---

*Questions? Open an issue or start a discussion.*
