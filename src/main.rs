mod chip8;
use chip8::Chip8;

use minifb::{Key, Scale, Window, WindowOptions};

const KEYMAP: [Key; 16] = [
    Key::X,
    Key::Key1,
    Key::Key2,
    Key::Key3,
    Key::Q,
    Key::W,
    Key::E,
    Key::A,
    Key::S,
    Key::D,
    Key::Z,
    Key::C,
    Key::Key4,
    Key::R,
    Key::F,
    Key::V,
];
fn main() {
    let rom_path = std::env::args()
        .nth(1)
        .expect("usage: cargo run -- <path-to-rom>");
    let bytes = std::fs::read(&rom_path).expect("failed to read ROM");

    let mut emulator = Chip8::new();
    emulator.load_rom(&bytes);

    let mut window = Window::new(
        "CHIP-8",
        Chip8::SCREEN_WIDTH,
        Chip8::SCREEN_HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )
    .expect("failed to create window");

    window.set_target_fps(60);

    let mut buffer = [0u32; Chip8::SCREEN_WIDTH * Chip8::SCREEN_HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for (chip8_key, &host_key) in KEYMAP.iter().enumerate() {
            emulator.set_key(chip8_key, window.is_key_down(host_key));
        }
        for _ in 0..10 {
            emulator.cycle();
        }

        emulator.tick_timers();

        for (dst, &on) in buffer.iter_mut().zip(emulator.display()) {
            *dst = if on { 0x00FF_FFFF } else { 0x0000_0000 };
        }

        window
            .update_with_buffer(&buffer, Chip8::SCREEN_WIDTH, Chip8::SCREEN_HEIGHT)
            .expect("failed to update window");
    }
}
