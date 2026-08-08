# Building Frost Studio DAW from Source

Building from source on your own machine avoids code signing costs and SmartScreen warnings. The installer will be trusted because you built it yourself.

## Prerequisites

### All Platforms

- **Node.js** 18+ (recommended: 20 LTS): [nodejs.org](https://nodejs.org)
- **Rust** stable toolchain: [rustup.rs](https://rustup.rs)

```bash
rustup default stable
```

### Windows

- **Visual Studio Build Tools** or **Visual Studio 2022** with the "Desktop development with C++" workload
  - [Download VS Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
  - Or in VS Installer, check: "Desktop development with C++" → includes MSVC toolchain + Windows SDK
- **WebView2** — pre-installed on Windows 10/11

### macOS

- **Xcode Command Line Tools:**
  ```bash
  xcode-select --install
  ```
- Requires macOS 10.15+ (Catalina)
- **Note:** VST3 hosting currently only works on Windows, but all built-in synths/effects work on macOS.

### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  wget \
  file \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libxdo-dev
```

For other distros, see [Tauri v2 Linux dependencies](https://v2.tauri.app/start/prerequisites/#linux).

## Build Steps

### 1. Clone or copy the source

```bash
git clone https://github.com/gabelossless/frost-studio-daw
cd frost-studio-daw
```

### 2. Install Node.js dependencies

```bash
npm install
```

### 3. TypeScript type-check (optional)

```bash
npx tsc --noEmit
```

### 4. Build for production

```bash
npm run tauri build
```

This command:
1. Builds the React frontend (`npm run build`)
2. Compiles all Rust crates (DAW backend, frost-core, VST plugins)
3. Packages everything into platform-specific installers

### 5. Find the installer

| Platform | Location |
|----------|----------|
| Windows  | `src-tauri/target/release/bundle/msi/*.msi` |
| Windows  | `src-tauri/target/release/bundle/nsis/*.exe` |
| macOS    | `src-tauri/target/release/bundle/dmg/*.dmg` |
| Linux    | `src-tauri/target/release/bundle/deb/*.deb` |

### 6. Install

- **Windows:** Double-click the `.msi` or `.exe` — no SmartScreen warning since you built it.
- **macOS:** Drag the `.app` from the `.dmg` to Applications. You may need to right-click → Open on first launch.
- **Linux:** `sudo dpkg -i path/to/*.deb`

## Building the Standalone VST Plugins

The `vst/` workspace compiles the Frost effects into VST3 and CLAP plugin
binaries using `nih-plug`:

```bash
cargo build --release -p frost-vst-compressor -p frost-vst-eq -p frost-vst-limiter \
             -p frost-vst-bass -p frost-vst-delay -p frost-vst-reverb
```

Output lands in:

```
target/release/frost_compressor.vst3/
target/release/frost_eq.vst3/
target/release/frost_limiter.vst3/
target/release/frost_bass.vst3/
target/release/frost_delay.vst3/
target/release/frost_reverb.vst3/
# plus .clap variants
```

To install into a DAW, copy the `.vst3/` folders to your standard VST3 directory:

| Platform | Default VST3 location |
|----------|----------------------|
| Windows  | `C:\Program Files\Common Files\VST3` |
| macOS    | `~/Library/Audio/Plug-Ins/VST3` |
| Linux    | `~/.vst3` |

## Distribution

Installers are **unsigned**. There are three ways to distribute, all free:

1. **Build from source** — recommended. The installer is trusted because you
   built it. Follow the steps above.
2. **GitHub Actions artifacts** — the CI builds all three platforms on every
   push/PR. Download from the [Actions tab](https://github.com/gabelossless/frost-studio-daw/actions).
   Still unsigned, so users may see SmartScreen/Gatekeeper warnings.
3. **GitHub Releases** — attach the built installers to a release page for a
   clean download URL (still unsigned).

Without a code-signing certificate, users will see:

- **Windows**: SmartScreen "Windows protected your PC" → *More info → Run anyway*.
- **macOS**: Gatekeeper "cannot be opened because it is from an unidentified
  developer" → right-click → Open, or `xattr -cr /Applications/Frost Studio DAW.app`.
- **Linux**: no warnings.

Paid code-signing (Windows EV cert ~$300–500/yr, Apple Developer $99/yr) removes
these warnings but is optional for development and open-source distribution.

## Troubleshooting

### Windows

- **`link.exe` not found / MSVC build tools missing**
  Install Visual Studio Build Tools with the "Desktop development with C++"
  workload, then reopen the terminal so the environment is refreshed.

- **SmartScreen warning on your *own* build**
  This only appears for binaries downloaded from the internet or built by CI.
  A locally built installer is signed by your own machine context. If it still
  warns, right-click the file → Properties → check "Unblock".

- **Audio device is a "Default" placeholder with no options**
  CPAL enumerates available hosts; on Windows the WASAPI host is used. Ensure no
  other app is exclusively locking the device (Settings → System → Sound →
  check the device is not in exclusive mode).

### macOS

- **"App is damaged" / cannot be opened**
  `xattr -cr "/Applications/Frost Studio DAW.app"` removes the quarantine flag.

- **No audio output**
  Grant microphone/audio permissions in System Settings → Privacy & Security,
  and make sure the app is not sandboxed by Gatekeeper flags (same `xattr -cr` fix).

### Linux (Debian/Ubuntu)

- **`libwebkit2gtk-4.1-dev` not found**
  On older releases the package may be named `libwebkit2gtk-4.0-dev` or you need
  to enable the appropriate apt repos. See the
  [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux).

- **Missing runtime libraries at launch**
  Install the same packages from the Prerequisites section that are used at
  build time (they are also runtime dependencies).

### Rust / Cargo (any platform)

- **Wrong toolchain**
  Run `rustup default stable` and `rustup show` to verify. The workspace uses
  stable Rust (CI uses `dtolnay/rust-toolchain@stable`).

- **Workspace member errors / missing crates**
  The workspace members are declared in the root `Cargo.toml`. After pulling
  new changes, run `cargo check --workspace` once to refresh the lockfile.

### Audio (all platforms)

- **No sound / underruns (clicks and pops)**
  1. Check the audio device selection in the Audio Settings modal.
  2. Try a larger buffer size.
  3. Confirm the sample rate of the device matches expectations — the engine
     currently assumes 44100 Hz (a v0.2.0 roadmap item is sample-rate
     independence).
  4. Close CPU-heavy applications.

## Development Mode

```bash
npm run dev
```

This starts the Vite dev server on port 3001 and a Tauri dev window with hot-reload.

## CI/CD Builds

Pre-built installers are available as GitHub Actions artifacts for each release. See the [Actions tab](https://github.com/gabelossless/frost-studio-daw/actions) to download the latest build for your platform.

Builds are still **unsigned** — they come from a public CI runner, not a trusted certificate authority. Your OS may still warn you. Building from source is the only way to get a fully trusted binary without paying for code signing.
