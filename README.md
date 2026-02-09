# xkey

[![CI](https://github.com/manhpham90vn/xkey/actions/workflows/rust.yml/badge.svg)](https://github.com/manhpham90vn/xkey/actions/workflows/rust.yml)
[![Release](https://github.com/manhpham90vn/xkey/actions/workflows/release.yml/badge.svg)](https://github.com/manhpham90vn/xkey/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight, high-performance Vietnamese Telex input method for **Ubuntu**, running as an IBus engine. Written in Rust.

> [!IMPORTANT]
> **This project only supports Ubuntu.** Other Linux distributions are not officially supported and may not work correctly.

## Features

- **Standard Telex** - Full Vietnamese Telex rules support
- **Smart Tone Placement** - Automatically places tone marks correctly
- **IBus Integration** - Works as a standard IBus engine
- **REPL Mode** - Test transformations directly in terminal
- **Lightweight** - Minimal resources, fast response
- **Memory Safe** - Built with Rust

## Installation

### Prerequisites

> [!CAUTION]
> **Ubuntu only.** This project has been tested and developed exclusively for Ubuntu. It will not work on other Linux distributions.

- **Ubuntu** (required)
- [Rust](https://www.rust-lang.org/tools/install) (only for building from source)
- IBus daemon
- Build dependencies:
  ```bash
  sudo apt install libdbus-1-dev pkg-config ibus
  ```

### Option 1: Download .deb Package (Recommended)

1. Go to the [Releases](https://github.com/manhpham90vn/xkey/releases) page
2. Download the latest `.deb` file
3. Install:
   ```bash
   sudo apt install ./xkey_*.deb
   ibus restart
   ```
4. Add xkey via **Settings > Keyboard > Input Sources > Add Input Source... > ⋮ > Other > Vietnamese (XKey Vietnamese Telex) > Add**

### Option 2: Build from Source

```bash
git clone https://github.com/manhpham90vn/xkey.git
cd xkey
./install.sh
```

The install script will:

- Build in release mode (`cargo build --release`)
- Install binary to `/usr/libexec/ibus-engine-xkey`
- Install component XML to `/usr/share/ibus/component/xkey.xml`
- Restart IBus daemon
- Add xkey to GNOME input sources
- Activate xkey as default input method

### Uninstall

```bash
./clean.sh
```

This removes binary, component XML, and xkey from GNOME input sources.

## Usage

### As IBus Engine

Once installed, switch input methods using:

- **Super + Space** (GNOME default)
- IBus tray icon
- GNOME top bar input indicator

### Telex Typing

| Input        | Output | Description    |
| ------------ | ------ | -------------- |
| `vieetj`     | việt   | ê + tone       |
| `chaof`      | chào   | tone mark      |
| `ươ` → `uow` | ươ     | vowel shortcut |
| `dd`         | đ      | đ shortcut     |

Press **Space**, **Enter**, or punctuation to commit.

### REPL Mode

Test Telex transformations directly:

```bash
cargo run -- --repl
```

### Troubleshooting

**xkey not appearing in input list:**

```bash
ibus list-engine | grep xkey
```

**Check IBus daemon:**

```bash
ibus restart
# or
systemctl --user restart org.freedesktop.IBus.session.GNOME.service
```

**View logs:**

```bash
journalctl --user -b | grep -i ibus | tail -50
```

## Architecture

```
src/
├── main.rs     # Entry point, CLI
├── engine.rs   # IBus engine, D-Bus communication
├── core.rs     # Buffer management, input logic
├── telex.rs    # Telex transformation rules
├── repl.rs     # REPL mode
└── utils.rs    # Utilities
```

| Module      | Description                                                          |
| ----------- | -------------------------------------------------------------------- |
| `engine.rs` | D-Bus communication with IBus, receives key events and sends signals |
| `core.rs`   | Manages input buffer, decides when to transform/commit text          |
| `telex.rs`  | Implements Vietnamese Telex transformation rules                     |

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

- **CI**: Automated build and test on every push
- **Release**: Build and publish binaries when pushing tags (`v*`)

## License

[MIT](LICENSE)
