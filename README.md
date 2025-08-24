# terminal-fractal

Animated Julia fractal renderer for the terminal. Minimal, fast, and deliberately simple: a single `main.rs` plus a small color helper. It animates the complex constant and paints smooth ANSI 256-color output each frame. Press `q` or `Ctrl+C` to quit.

## Features

- Animated Julia set (parameter follows a smooth pseudo-random drift, not a fixed circle)
- 256-color ANSI gradient + perceptual character ramp
- Adaptive terminal size usage every frame (resizing just works)
- Clean exit (raw mode + alternate screen restored)
- Zero configuration runtime (edit a few constants to tweak)
- Pure Rust with only `crossterm`, `ctrlc`, and `num-complex`

## Quick Start

```bash
# Run
cargo run --release
```

## Controls

| Key    | Action          |
|--------|-----------------|
| q      | Quit            |
| Ctrl+C | Quit (graceful) |

## Character Ramp

The ramp currently used:

```text
' . : - = + * o O # █
```

## Color Support

Requires a terminal with 256-color (most modern terminals). If colors look flat, ensure:

- `TERM` advertises 256 colors (e.g. `xterm-256color`)
- Windows Terminal or recent Windows console (which supports ANSI sequences) is in use

## Demo

<video sr

https://github.com/user-attachments/assets/449aa87d-98fd-4176-8088-3bcbb0bb5524

c="assets/demo.mp4" autoplay loop muted playsinline width="640"></video>

## License

MIT.

## Acknowledgements

- Built with Rust + crossterm.
- Inspired by classic ASCII fractal demos.

---
