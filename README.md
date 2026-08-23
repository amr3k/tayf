# AnimaView

<div align="center">

![License](https://img.shields.io/badge/license-GPL--3.0--or--later-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
![Rust](https://img.shields.io/badge/rust-100%25-orange?logo=rust)
![UI](https://img.shields.io/badge/GUI-GPUI-blue)
![Engine](https://img.shields.io/badge/Lottie-ThorVG-purple)

**A blazing-fast, lightweight, 100% native Rust Lottie animation viewer powered by GPUI and ThorVG.**

[Homepage](https://github.com/amr3k/AnimaView) • [Issues](https://github.com/amr3k/AnimaView/issues) • [Releases](https://github.com/amr3k/AnimaView/releases)

</div>

## About

AnimaView is a high-performance native desktop application designed for designers and developers who need to preview, inspect, control, and export Lottie animations. Built entirely in native Rust using **GPUI** (GPU-accelerated UI engine from Zed) and **ThorVG** (high-performance vector graphics engine), AnimaView starts instantly, consumes minimal memory, and renders animations with silky-smooth precision.

## Features

### Core Functionality
- **Multi-format Support**: Open standard Lottie JSON (`.json`) and binary DotLottie package (`.lottie`) files
- **Multiple Input Methods**: File dialog, command-line arguments (`animaview <path>`), and drag-and-drop
- **Playback Controls**: Play/pause, step frame by frame, loop toggle, and speed adjustment (`0.5x`, `1.0x`, `1.5x`, `2.0x`)
- **Frame Scrubbing**: Interactive timeline scrubber displaying frame count and timestamps

### Export Engine
- **GIF Export**: Fast, high-quality animated GIF export with custom resolution, frame rate, transparency, and looping
- **MP4 Video Export**: H.264 video rendering with adjustable CRF quality and frame rates

### Visual Customization
- **Theme-aware Background Colors**: Customize canvas background separately for light and dark themes with color preset swatches
- **System Theme Integration**: Seamless switching between System, Light, and Dark modes

### Inspection Tools
- **Metadata Display**: Comprehensive file details:
  - Dimensions (original width/height)
  - Frame rate (FPS)
  - Duration and total frames
  - File size and format

### Internationalization
- **English & Arabic**: Native RTL layout and full Arabic localization powered by `rust-i18n`
- **Runtime Switching**: Instant language toggle from Preferences

### Keyboard Shortcuts
| Shortcut | Action |
|----------|--------|
| `Space` | Toggle play/pause |
| `←` / `→` | Step 1 frame backward / forward |
| `Shift+←` / `Shift+→` | Step 10 frames backward / forward |
| `Esc` | Close modal or reset animation |
| `Ctrl+O` / `Cmd+O` | Open file dialog |
| `Ctrl+P` / `Cmd+P` | Open Preferences |
| `Ctrl+E` / `Cmd+E` | Open Export dialog |
| `Ctrl+B` / `Cmd+B` | Toggle Sidebar |
| `F1` | Open About dialog |

## Tech Stack

| Category | Technology |
|----------|-----------|
| **GUI Framework** | [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) (GPU-accelerated 2D UI) |
| **Vector / Lottie Engine** | [ThorVG](https://www.thorvg.org/) (High-performance vector rasterization) |
| **Language** | 100% Rust |
| **Export Formats** | Animated GIF (`gif` crate) & MP4 (`ffmpeg`) |
| **Localization** | `rust-i18n` (English & Arabic with RTL) |
| **Package Manager** | `cargo` |

## Installation & Development

### Prerequisites

- **Rust toolchain** (1.80+ recommended)
- **C++ compiler** (for building ThorVG C++ backend)

### Build & Run

```bash
# Clone repository
git clone https://github.com/amr3k/AnimaView.git
cd AnimaView

# Run in development mode
cargo run

# Open a specific file
cargo run -- /path/to/animation.json

# Run unit tests
cargo test

# Build optimized release binary
cargo build --release
```

The resulting standalone binary is located at `target/release/animaview`.

## Configuration

Configuration is automatically stored in the OS app configuration directory:

| Platform | Path |
|----------|------|
| Linux | `~/.config/com.Amr.AnimaView/configurations.json` |
| macOS | `~/Library/Application Support/com.Amr.AnimaView/configurations.json` |
| Windows | `%APPDATA%\Amr\AnimaView\configurations.json` |

## License

This project is licensed under the **GPL-3.0-or-later** License - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

Made with ❤️ by [Amr](https://github.com/amr3k)

</div>
