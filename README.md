# xkey

[![CI](https://github.com/manhpham90vn/xkey/actions/workflows/rust.yml/badge.svg)](https://github.com/manhpham90vn/xkey/actions/workflows/rust.yml)
[![Release](https://github.com/manhpham90vn/xkey/actions/workflows/release.yml/badge.svg)](https://github.com/manhpham90vn/xkey/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight, high-performance Vietnamese Telex input method for **Linux** (IBus) and **macOS** (InputMethodKit). Written in Rust.

## Features

- **Standard Telex** - Full Vietnamese Telex rules support
- **Smart Tone Placement** - Automatically places tone marks correctly
- **Cross-Platform** - Linux (IBus engine) and macOS (InputMethodKit)
- **REPL Mode** - Test transformations directly in terminal
- **Lightweight** - Minimal resources, fast response
- **Memory Safe** - Built with Rust

## Installation

### Linux (Ubuntu)

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (only for building from source)
- IBus daemon
- Build dependencies:
  ```bash
  sudo apt install libdbus-1-dev pkg-config ibus
  ```

#### Option 1: Download .deb Package (Recommended)

1. Go to the [Releases](https://github.com/manhpham90vn/xkey/releases) page
2. Download the latest `.deb` file
3. Install:
   ```bash
   sudo apt install ./xkey_*.deb
   ibus restart
   ```
4. Add xkey via **Settings > Keyboard > Input Sources > Add Input Source... > ⋮ > Other > Vietnamese (XKey Vietnamese Telex) > Add**

#### Option 2: Build from Source

```bash
git clone https://github.com/manhpham90vn/xkey.git
cd xkey
./linux/install.sh
```

### macOS

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (only for building from source)

#### Option 1: Download App Bundle (Recommended)

1. Go to the [Releases](https://github.com/manhpham90vn/xkey/releases) page
2. Download `xkey-macos.zip`
3. Unzip and move `XKey.app` to `~/Library/Input Methods/`
4. **Bypass Gatekeeper** (the app is not signed with an Apple Developer Certificate):
   - Open Terminal and run:
     ```bash
     xattr -cr ~/Library/Input\ Methods/XKey.app
     ```
   - Or if you see **"XKey.app can't be opened because Apple cannot check it for malicious software"**:
     1. Open **System Settings > Privacy & Security**
     2. Scroll down to the **Security** section, find the message about `XKey.app`
     3. Click **Open Anyway**
5. Log out and log back in
6. Add xkey via **System Settings > Keyboard > Input Sources > + > Vietnamese > XKey Vietnamese Telex**

> [!WARNING]
> The app is not signed with an Apple Developer Certificate, so macOS will block it on first launch. You must complete step 4 to allow it to run.

#### Option 2: Build from Source

```bash
git clone https://github.com/manhpham90vn/xkey.git
cd xkey
./macos/install.sh
```

The install script will:
- Build in release mode (`cargo build --release`)
- Create the `.app` bundle with proper `Info.plist`
- Install to `~/Library/Input Methods/`

> [!NOTE]
> After installation, you must **log out and log back in** for macOS to detect the new input method.

### Uninstall

**Linux:**
```bash
./linux/clean.sh
```

**macOS:**
```bash
./macos/clean.sh
```
Then log out and log back in.

## Usage

### As Input Method

Once installed, switch input methods using:

- **Linux**: Super + Space (GNOME default) or IBus tray icon
- **macOS**: Ctrl + Space or globe key (⌘)

### Telex Typing

| Input        | Output | Description    |
| ------------ | ------ | -------------- |
| `vieetj`     | việt   | ê + tone       |
| `chaof`      | chào   | tone mark      |
| `ươ` → `uow` | ươ     | vowel shortcut |
| `dd`         | đ      | đ shortcut     |

Press **Space**, **Enter**, or punctuation to commit.

### REPL Mode

Test Telex transformations directly (works on all platforms):

```bash
cargo run -- --repl
```

### Troubleshooting

**Linux - xkey not appearing in input list:**

```bash
ibus list-engine | grep xkey
```

**Linux - Check IBus daemon:**

```bash
ibus restart
```

**macOS - xkey not appearing:**

1. Make sure `XKey.app` is in `~/Library/Input Methods/`
2. Log out and log back in
3. Check Console.app for any InputMethodKit errors

## Architecture

```
src/
├── main.rs              # Entry point, platform dispatch
├── lib.rs               # Library exports (for FFI + doctests)
├── core.rs              # Buffer management, input logic
├── telex.rs             # Telex transformation rules
├── repl.rs              # REPL mode
├── utils.rs             # Utilities
└── platform/
    ├── mod.rs            # Platform cfg gates
    ├── linux/
    │   ├── mod.rs        # IBus startup logic
    │   └── engine.rs     # IBus D-Bus engine
    └── macos/
        └── mod.rs        # C FFI exports for Swift bridge

macos/
├── main.swift            # macOS app entry point (IMKServer)
├── Engine.swift          # InputMethodKit engine (Swift)
├── xkey-Bridging-Header.h # C-to-Swift bridging header
├── Info.plist            # App bundle configuration
├── install.sh            # Build & install script
└── clean.sh              # Uninstall script
```

| Module                  | Description                                             |
| ----------------------- | ------------------------------------------------------- |
| `core.rs`               | Manages input buffer, decides when to transform/commit  |
| `telex.rs`              | Implements Vietnamese Telex transformation rules        |
| `platform/linux/`       | IBus engine via D-Bus (zbus) — Linux only               |
| `platform/macos/mod.rs` | C FFI exports for the Swift bridge — macOS only         |
| `macos/Engine.swift`    | InputMethodKit controller (Swift) — macOS only          |

## Development

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## CI/CD

- **CI**: Automated build and test on every push (Linux + macOS)
- **Release**: Build and publish binaries for both platforms when pushing tags (`v*`)

## License

[MIT](LICENSE)
