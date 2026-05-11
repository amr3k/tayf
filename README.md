# AnimaView

<div align="center">

![Version](https://img.shields.io/badge/dynamic/json?url=https://raw.githubusercontent.com/amr3k/AnimaView/main/package.json&query=$.version&prefix=v&logo=github&label=Version)
![License](https://img.shields.io/badge/license-GPL--3.0--or--later-green)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)

**A fast, reliable, cross-platform Lottie animation viewer for desktop and mobile devices**

[Homepage](https://github.com/amr3k/AnimaView) • [Issues](https://github.com/amr3k/AnimaView/issues) • [Releases](https://github.com/amr3k/AnimaView/releases)

</div>

## About

AnimaView is a powerful desktop application designed for designers and developers who need to preview, inspect, and control Lottie animations. Built with Tauri and Svelte, it provides a native-like experience with modern web technologies.

## Features

### Core Functionality
- **Multi-format Support**: Open Lottie JSON (`.json`) and binary (`.lottie`) files
- **Multiple Input Methods**: File dialog, drag-and-drop, and OS-level file associations
- **Playback Controls**: Play/pause, loop toggle, speed adjustment (0.1x - 5.0x)
- **Frame Scrubbing**: Navigate through animation frames with a precise slider

### Visual Customization
- **Theme-aware Background Colors**: Customize canvas background separately for light and dark themes
- **Responsive Layout**: Desktop and mobile-friendly with adaptive UI
- **Clean Interface**: Modern design built on shadcn-svelte components

### Inspection Tools
- **Metadata Display**: View file information including:
  - Dimensions (original width/height)
  - Frame rate (FPS)
  - Duration and total frames
  - File size
- **Real-time Updates**: All playback changes reflected instantly

### Multi-window Architecture
- **Main Window**: Full-featured viewer with control panel
- **Preferences Window**: Configure theme and language settings
- **About Window**: App information and credits

### Internationalization
- **English & Arabic**: Full RTL support for Arabic users
- **Runtime Switching**: Change language without restarting

### Keyboard Shortcuts
| Shortcut | Action |
|----------|--------|
| `Space` | Toggle play/pause |
| `←` / `→` | Navigate frames backward/forward (1 frame) |
| `Ctrl+←` / `Ctrl+→` | Navigate frames backward/forward (10 frames) |
| `Esc` | Close window / Reset animation |
| `Ctrl+W` / `Cmd+W` | Close window or reset animation |
| `F1` | Open About window |
| `Ctrl+O` / `Cmd+O` | Open file dialog |
| `Ctrl+P` / `Cmd+P` | Open Preferences |
| `Ctrl+Q` / `Cmd+Q` | Quit application |

## Tech Stack

| Category | Technology |
|----------|-----------|
| **Application Shell** | Tauri 2.0 |
| **Frontend Framework** | Svelte 5 + SvelteKit |
| **Language** | TypeScript |
| **Lottie Playback** | @lottiefiles/dotlottie-svelte |
| **Styling** | Tailwind CSS v4 |
| **UI Components** | bits-ui / shadcn-svelte |
| **Theme System** | mode-watcher |
| **Notifications** | svelte-sonner |
| **i18n** | Wuchale |
| **Package Manager** | pnpm + cargo |

## Installation

### Prerequisites

- **Node.js** (v18 or higher)
- **pnpm** (v10 or higher)
- **Rust** and Cargo (for Tauri)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/amr3k/AnimaView.git
cd AnimaView

# Install dependencies
pnpm install

# Start development server
pnpm tauri dev
```

### Production Build

```bash
pnpm tauri build
```

The built application will be in `src-tauri/target/release/bundle/`.

### Linux AppImage release build

For a Linux-only AppImage release check:

```bash
pnpm check
pnpm build:appimage
pnpm inspect:appimage
```

The AppImage is written to `src-tauri/target/release/bundle/appimage/`.
`pnpm inspect:appimage` extracts the newest AppImage locally and prints the
desktop entry, icon files, and Lottie MIME/file-association references so the
package can be checked before publishing.

`pnpm build:appimage` runs Tauri with `NO_STRIP=1` because post-bundle
stripping can break bundled GTK/WebKit libraries. The follow-up finalization
script patches the desktop entry for Open With (`%F`), makes GTK prefer
native Wayland with X11/XWayland fallback, validates the metadata, and repacks
the AppImage. Set `APPIMAGE_HOST_STRIP=1` only for local size experiments.

### Installing from Release

Download the latest release for your platform from the [Releases page](https://github.com/amr3k/AnimaView/releases) and install according to your OS:

- **Windows**: Run the `.exe` installer
- **macOS**: Open the `.dmg` file and drag to Applications
- **Linux**: Install the `.AppImage` or `.deb` package

## Usage

### Opening Animations

1. **File Dialog**: Click "Choose file" button or press `Ctrl+O`
2. **Drag & Drop**: Drag a `.json` or `.lottie` file onto the main window
3. **File Association**: Double-click a Lottie file in your file manager

### Controlling Playback

- Use the **Control Panel** on the right side to adjust:
  - Play/Pause (large circular button)
  - Frame position (scrubber slider)
  - Playback speed (dropdown or custom slider)
  - Loop mode (toggle switch)
- Keyboard shortcuts for quick actions

### Customizing Appearance

1. Open **Preferences** via menu or `Ctrl+P`
2. Select **Theme** (System/Light/Dark)
3. Choose **Language** (English/Arabic)
4. Background colors are customizable from the Control Panel

## Configuration

Configuration is stored automatically in your OS's app config directory:

| Platform | Path |
|----------|------|
| Linux | `~/.config/me.a3k.animaview/configurations.json` |
| macOS | `~/Library/Application Support/me.a3k.animaview/configurations.json` |
| Windows | `%APPDATA%\me.a3k.animaview\configurations.json` |

### Default configurations:

```json
{
  "theme": "system",
  "lang": "en",
  "canvasBackgroundColor": "#FFFFFF",
  "canvasBackgroundColorDark": "#0F1115"
}
```

### Adding a New Language

1. Add language code to `languages.json`:
   ```json
   ["en", "ar", "es"]
   ```

2. Run `pnpm dev` to auto-generate types

3. Wuchale will automatically create a new translation file at: `src/locales/es.po`

4. Translate strings using Wuchale CLI

## Roadmap

- [x] Export animations as GIF/Video
- [x] Frame-by-frame navigation with arrow keys
- [ ] Animation timeline scrubbing
- [ ] Custom playback ranges
- [ ] Mobile app version
- [ ] More languages support

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- Use **English** for all code, comments, and documentation
- Follow **Svelte 5** conventions (runes for reactivity)
- Use **TypeScript** strictly (no `any` types)
- Follow existing **Tailwind** patterns

### Before Submitting

- Run `pnpm check` to ensure type safety
- Test on multiple platforms if possible
- Update documentation as needed

## License

This project is licensed under the **GPL-3.0-or-later** License - see the [LICENSE](LICENSE) file for details.

## Support

- [Bug Reports](https://github.com/amr3k/AnimaView/issues)
- [Feature Requests](https://github.com/amr3k/AnimaView/issues)
- [Discussions](https://github.com/amr3k/AnimaView/discussions)

---

<div align="center">

Made with ❤️ by [Amr](https://github.com/amr3k)

If you like AnimaView, please consider giving it a ⭐ on [GitHub](https://github.com/amr3k/AnimaView)!

</div>
