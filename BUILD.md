# Building Frost Studio DAW from Source

Building from source on your own machine avoids code signing costs and SmartScreen warnings. The installer will be trusted because you built it yourself.

## Prerequisites

### All Platforms

- **Node.js** 18+ (recommended: 20 LTS): [nodejs.org](https://nodejs.org)
- **Rust** nightly toolchain: [rustup.rs](https://rustup.rs)

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

## Development Mode

```bash
npm run dev
```

This starts the Vite dev server on port 3001 and a Tauri dev window with hot-reload.

## CI/CD Builds

Pre-built installers are available as GitHub Actions artifacts for each release. See the [Actions tab](https://github.com/gabelossless/frost-studio-daw/actions) to download the latest build for your platform.

Builds are still **unsigned** — they come from a public CI runner, not a trusted certificate authority. Your OS may still warn you. Building from source is the only way to get a fully trusted binary without paying for code signing.
