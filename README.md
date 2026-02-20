# xkey

[![CI](https://github.com/manhpham90vn/xkey/actions/workflows/rust.yml/badge.svg)](https://github.com/manhpham90vn/xkey/actions/workflows/rust.yml)
[![Release](https://github.com/manhpham90vn/xkey/actions/workflows/release.yml/badge.svg)](https://github.com/manhpham90vn/xkey/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight, high-performance Vietnamese Telex input method for **Linux** (IBus), **macOS** (InputMethodKit), and **Windows** (keyboard hook). Written in Rust.

## Features

- **Standard Telex** - Full Vietnamese Telex rules support
- **Smart Tone Placement** - Automatically places tone marks correctly
- **Cross-Platform** - Linux (IBus engine), macOS (InputMethodKit), and Windows (keyboard hook)
- **REPL Mode** - Test transformations directly in terminal
- **Lightweight** - Minimal resources, fast response
- **Memory Safe** - Built with Rust

## Installation

### Linux (Ubuntu)

#### Download .deb Package (Recommended)

1. Go to the [Releases](https://github.com/manhpham90vn/xkey/releases) page
2. Download the latest `.deb` file
3. Install:
   ```bash
   sudo apt install ./xkey_*.deb
   ibus restart
   ```
4. Add xkey via **Settings > Keyboard > Input Sources > Add Input Source... > ⋮ > Other > Vietnamese (XKey Vietnamese Telex) > Add**

#### Build from Source

Install prerequisites first:
```bash
sudo apt install libdbus-1-dev pkg-config ibus
```

You also need [Rust](https://www.rust-lang.org/tools/install). Then:
```bash
git clone https://github.com/manhpham90vn/xkey.git
cd xkey
./linux/install.sh
```

### macOS

<<<<<<< HEAD
#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

#### Option 1: Download App Bundle (Recommended)
=======
#### Download App Bundle (Recommended)
>>>>>>> master

1. Go to the [Releases](https://github.com/manhpham90vn/xkey/releases) page
2. Download `xkey-macos.zip`
3. Unzip and move `XKey.app` to `~/Library/Input Methods/`
4. Log out and log back in
5. Add xkey via **System Settings > Keyboard > Input Sources > + > Vietnamese > XKey Vietnamese Telex**

#### Build from Source

Install [Rust](https://www.rust-lang.org/tools/install) first, then:
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

### Windows

#### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

#### Option 1: Download Executable (Recommended)

1. Go to the [Releases](https://github.com/manhpham90vn/xkey/releases) page
2. Download `xkey-windows.zip`
3. Extract `xkey.exe` to a folder (e.g., `C:\Program Files\XKey\`)
4. Run `xkey.exe` — it runs as a background process
5. (Optional) Add to Windows startup by placing a shortcut in `shell:startup`

#### Option 2: Build from Source

```powershell
git clone https://github.com/manhpham90vn/xkey.git
cd xkey
.\windows\install.ps1
```

The install script will:
- Build in release mode (`cargo build --release`)
- Copy `xkey.exe` to `%LOCALAPPDATA%\XKey\`
- Add to Windows startup (Registry Run key)

> [!NOTE]
> XKey runs as a background process using a keyboard hook. It intercepts keystrokes and injects Vietnamese characters via `SendInput`. Press **Ctrl+C** in the terminal to stop, or end the xkey process from Task Manager.

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

**Windows:**
```powershell
.\windows\clean.ps1
```

## Usage

### As Input Method

Once installed, switch input methods using:

- **Linux**: Super + Space (GNOME default) or IBus tray icon
- **macOS**: Ctrl + Space or globe key (⌘)
- **Windows**: Run `xkey.exe` to activate (runs in background)

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

**Windows - xkey not working:**

1. Make sure `xkey.exe` is running (check Task Manager)
2. Some elevated (Administrator) applications may not receive injected input due to UIPI
3. If keystrokes are not intercepted, try running `xkey.exe` as Administrator

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

linux/
├── install.sh            # Build & install script
├── clean.sh              # Uninstall script
└── xkey.xml              # IBus engine descriptor

macos/
├── main.swift            # macOS app entry point (IMKServer)
├── Engine.swift          # InputMethodKit engine (Swift)
├── xkey-Bridging-Header.h # C-to-Swift bridging header
├── Info.plist            # App bundle configuration
├── xkey.icns             # App icon
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

- **CI**: Automated build and test on every push (Linux + macOS + Windows)
- **Release**: Build and publish binaries for all three platforms when pushing tags (`v*`)

## License

[MIT](LICENSE)
