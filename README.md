# Badge Designer

A web-based editor for designing LED badge animations. Create pixel art frames, configure animation speed, and export configurations to flash onto LED badges.

## 🚀 [Try it online](https://mnlphlp.github.io/badge_designer)

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
- [Dioxus CLI](https://dioxuslabs.com/learn/0.7/getting_started)

```bash
curl -sSL http://dioxus.dev/install.sh | sh
```

### Run locally

```bash
dx serve
```

### Build for production

```bash
dx build --release
```
