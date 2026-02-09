# xkey ⌨️

**xkey** is a lightweight, high-performance Vietnamese Telex input method for Linux, running as an IBus engine. It is written in Rust for safety and efficiency.

## ✨ Features

- **Standard Telex**: Supports full Vietnamese Telex rules including vowel marks and tones.
- **Smart Tone Placement**: Automatically places tone marks on the correct vowels according to Vietnamese grammar.
- **IBus Integration**: Works seamlessly as a standard IBus input engine.
- **REPL Mode**: Includes a built-in terminal-based REPL for testing transformations without installing the full engine.
- **Lightweight**: Minimal resource usage and fast response times.
- **Safety**: Built with Rust to ensure memory safety and prevent common bugs.

## 🚀 Getting Started

### Prerequisites

- **Rust**: [Install Rust](https://www.rust-lang.org/tools/install)
- **IBus**: Ensure `ibus` is installed on your system.
- **Build Dependencies**:
  ```bash
  sudo apt-get update
  sudo apt-get install -y libdbus-1-dev pkg-config
  ```

### Installation

#### Option 1: Install via .deb (Recommended for Ubuntu/Debian)

1. Download the latest `.deb` package from the [Releases](https://github.com/manhpham90vn/xkey/releases) page.
2. Install the package:
   ```bash
   sudo apt install ./xkey_*.deb
   ```
3. Restart IBus:
   ```bash
   ibus restart
   ```

#### Option 2: Build from Source

1. Clone the repository:

   ```bash
   git clone https://github.com/manhpham90vn/xkey.git
   cd xkey
   ```

2. Run the installation script:

   ```bash
   ./install.sh
   ```

   This script will build the project in release mode and register the xkey engine with IBus.

3. Restart IBus:

   ```bash
   ibus restart
   ```

4. Add **xkey** to your IBus input methods via **IBus Preferences**.

### Usage in REPL Mode

You can test the Vietnamese transformation directly in your terminal:

```bash
cargo run -- --repl
```

In REPL mode, press **Space** or **Enter** to commit a word.

## 🏗️ Architecture

The project is structured into three main layers:

1.  **IBus Engine (`engine.rs`)**: Handles D-Bus communication with IBus, receiving key events and sending signals (UpdatePreedit, Commit).
2.  **Core Logic (`core.rs`)**: Manages the input buffer and decides when to transform or commit text.
3.  **Telex Processor (`telex.rs`)**: Implements the actual Vietnamese Telex transformation rules.

## 🛠️ Development

### Running Tests

We have a comprehensive unit test suite covering complex Vietnamese words and edge cases:

```bash
cargo test
```

### CI/CD

This project uses GitHub Actions for:

- **CI**: Automated building and testing on every push.
- **Release**: Automatically building and publishing binary releases when a tag (e.g., `v0.1.0`) is pushed.

## 📄 License

[MIT](LICENSE) (or your preferred license)

---

_Made with ❤️ for the Linux Vietnamese community._
