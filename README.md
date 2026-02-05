# Badge Designer

A desktop and web editor for designing LED badge animations using Slint UI framework with Rust backend. Create pixel art frames, configure animation speed, and export configurations to flash onto LED badges.

## Features

- **11x44 pixel grid** for LED badge design
- **Click and drag** to draw pixels
- **Multiple frames** for animations
- **Frame operations**: Invert, Clear, Clone, Delete
- **Make Cycle**: Create smooth back-and-forth animations
- **Configurable speed** (1-7) and padding (0-20)
- **Export/Import** .toml files compatible with [badgemagic-rs](https://github.com/fossasia/badgemagic-rs)
- **Auto-save**: State persists across sessions (localStorage for web, config file for native)

## Usage

1. Click or drag on the grid to toggle pixels on/off
2. Use the controls to invert, clear, clone, or remove frames
3. Add frames with "Add Frame" or duplicate with "Clone"
4. Use "Make Cycle" to create a smooth back-and-forth animation
5. Adjust padding between frames and animation speed
6. Export your design as a `.toml` file
7. Flash to your badge using [badgemagic-rs](https://github.com/fossasia/badgemagic-rs)

## Development

### Prerequisites

- [Rust](https://rustup.rs/)
- [Trunk](https://trunkrs.dev/) (for web builds)

```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
```

### Run locally (native)

```bash
cargo run
```

### Run locally (web)

```bash
trunk serve
```

Then open http://localhost:8080 in your browser.

### Build for release (native)

```bash
cargo build --release
```

The binary will be at `target/release/badge_designer`

### Build for release (web)

```bash
trunk build --release
```

The web app will be in `dist/` directory. Deploy these files to any static web server.

## Architecture

- **UI**: Slint (declarative UI framework)
- **Backend**: Rust
- **Persistence**: 
  - Native: JSON file in OS-specific config directory (via `directories` crate)
  - Web: Browser localStorage
- **File dialogs**: rfd (cross-platform, works in browser)

## Conversion from eframe/egui

This app was converted from an egui-based implementation to Slint. Key changes:
- UI defined in `.slint` files instead of immediate-mode Rust code
- Custom persistence layer (localStorage + file system) instead of eframe's built-in persistence
- Same TOML export format for compatibility

## License

See LICENSE file (inherited from original project).
