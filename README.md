# CHIP-8 Emulator

A standard CHIP-8 emulator written in Rust using [`minifb`](https://crates.io/crates/minifb) for rendering/input and `rand` for RNG.

## Features

- Full standard CHIP-8 instruction set (34 active opcodes)
- `64 × 32` monochrome display upscaled `16×` (`1024 × 512` window)
- `60 Hz` frame/timer loop (~`600 Hz` CPU clock at 10 cycles per frame)
- 16-key hex keypad mapped to a QWERTY keyboard

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (Edition 2024)
- A desktop display server (Windows/macOS natively, or X11/Wayland on Linux; if running over SSH, connect with `ssh -Y` or run locally)

## Usage

Pass the path to any standard `.ch8` ROM file:

```bash
cargo run --release -- path/to/rom.ch8
```

For example, if you place ROMs in a `roms/` directory (ignored by Git):

```bash
cargo run --release -- roms/PONG.ch8
```

Press **`Esc`** (or close the window) to exit.

## Controls

The original 4×4 CHIP-8 hex keypad is mapped to the left side of a QWERTY keyboard:

```text
CHIP-8 Keypad       Keyboard
1  2  3  C          1  2  3  4
4  5  6  D    <=>   Q  W  E  R
7  8  9  E          A  S  D  F
A  0  B  F          Z  X  C  V
```

## Project Structure

- `src/chip8.rs` — CHIP-8 virtual machine (`Chip8` state, fontset, fetch-decode-execute, sprite rendering, timers)
- `src/main.rs` — `minifb` window setup, keyboard polling, and 60 Hz main loop
