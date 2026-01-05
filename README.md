# Badge Designer

A native desktop editor for designing LED badge animations using egui. Create pixel art frames, configure animation speed, and export configurations to flash onto LED badges.

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

### Run locally

```bash
cargo run
```

### Build for release

```bash
cargo build --release
```
