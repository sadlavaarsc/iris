# Iris

A terminal image viewer written in Rust, with support for Kitty graphics protocol and automatic fallback to Unicode halfblocks.

## Features

- **Kitty Graphics Protocol** - Native support for high-quality terminal image rendering
- **Auto Fallback** - Automatically falls back to iTerm2, Sixel, or Unicode halfblocks depending on terminal capabilities
- **Interactive Mode** - Full TUI with zoom, pan, and mouse support
- **Static Mode** - Quick one-shot image display without entering interactive mode
- **Keyboard Controls** - Vim-style (`hjkl`) and arrow key navigation
- **Mouse Support** - Scroll to zoom in/out

## Installation

```bash
git clone https://github.com/sadlavaarsc/iris.git
cd iris
cargo build --release
```

The binary will be at `target/release/iris`.

## Usage

### Interactive Mode (default)

```bash
iris path/to/image.png
```

### Static Mode

```bash
iris path/to/image.png --no-interactive
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `+` / `=` | Zoom in |
| `-` / `_` | Zoom out |
| `←` `↑` `↓` `→` | Pan |
| `h` `j` `k` `l` | Pan (vim-style) |
| `w` `a` `s` `d` | Pan |
| `r` | Reset view |
| `q` / `Esc` | Quit |

### Mouse Controls

| Action | Effect |
|--------|--------|
| Scroll up | Zoom in |
| Scroll down | Zoom out |

## Terminal Compatibility

Iris uses [ratatui-image](https://github.com/benjajaja/ratatui-image) for protocol detection. Supported terminals:

| Protocol | Terminals |
|----------|-----------|
| Kitty | Kitty, Ghostty |
| iTerm2 | iTerm2, WezTerm |
| Sixel | XTerm, MLTerm, mintty |
| Halfblocks | Any truecolor terminal (fallback) |

## Tech Stack

- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal I/O
- [ratatui-image](https://github.com/benjajaja/ratatui-image) - Terminal image rendering
- [image](https://github.com/image-rs/image) - Image decoding
- [clap](https://github.com/clap-rs/clap) - CLI parsing

## License

MIT
