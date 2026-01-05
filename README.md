# Badge Designer

A desktop and web editor for designing LED badge animations using egui. Create pixel art frames, configure animation speed, and export configurations to flash onto LED badges.

## Usage

1. Click or drag on the grid to toggle pixels on/off
2. Use the controls to invert, clear, clone, or remove frames
3. Add frames with "Add Frame" or duplicate with "Clone"
4. Use "Make Cycle" to create a smooth back-and-forth animation
5. Export your design as a `.toml` file
6. Flash to your badge using [badgemagic-rs](https://github.com/fossasia/badgemagic-rs)

## Development

### Prerequisites

- [Rust](https://rustup.rs/)
- [Trunk](https://trunkrs.dev/) (for web builds)

```bash
cargo install trunk
```

### Run locally (native)

```bash
cargo run
```

### Run locally (web)

```bash
trunk serve
```

### Build for release (native)

```bash
cargo build --release
```

### Build for release (web)

```bash
trunk build --release
```
