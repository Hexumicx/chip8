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

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    memory: [u8; Chip8::MEMORY_SIZE],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    display: [bool; Chip8::SCREEN_WIDTH * Chip8::SCREEN_HEIGHT],
    keypad: [bool; 16],
    delay_timer: u8,
    sound_timer: u8,
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {
    pub const SCREEN_WIDTH: usize = 64;
    pub const SCREEN_HEIGHT: usize = 32;
    const MEMORY_SIZE: usize = 4096;
    const START_ADDR: u16 = 0x200;
    const FONTSET_ADDR: usize = 0x050;

    pub fn new() -> Self {
        let mut memory = [0; Self::MEMORY_SIZE];
        memory[Self::FONTSET_ADDR..Self::FONTSET_ADDR + FONTSET.len()].copy_from_slice(&FONTSET);

        Self {
            memory,
            v: [0; 16],
            i: 0,
            pc: Self::START_ADDR,
            stack: [0; 16],
            sp: 0,
            display: [false; Self::SCREEN_WIDTH * Self::SCREEN_HEIGHT],
            keypad: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn load_rom(&mut self, bytes: &[u8]) {
        let start = Self::START_ADDR as usize;
        assert!(
            start + bytes.len() <= Self::MEMORY_SIZE,
            "ROM too large to fit in memory"
        );
        self.memory[start..start + bytes.len()].copy_from_slice(bytes);
    }

    pub fn cycle(&mut self) {
        let hi = self.memory[self.pc as usize];
        let lo = self.memory[(self.pc + 1) as usize];
        let opcode = (hi as u16) << 8 | (lo as u16);
        self.pc += 2;

        let n1 = hi >> 4;
        let n2 = hi & 0x0F;
        let n3 = lo >> 4;
        let n4 = lo & 0x0F;

        let x = n2 as usize;
        let y = n3 as usize;
        let nn = lo;
        let nnn = opcode & 0x0FFF;

        match (n1, n2, n3, n4) {
            (0x0, 0x0, 0xE, 0x0) => self.display.fill(false),
            (0x0, 0x0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            }
            (0x1, _, _, _) => self.pc = nnn,
            (0x2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            (0x3, _, _, _) => self.skip_if(self.v[x] == nn),
            (0x4, _, _, _) => self.skip_if(self.v[x] != nn),
            (0x5, _, _, 0) => self.skip_if(self.v[x] == self.v[y]),
            (0x6, _, _, _) => self.v[x] = nn,
            (0x7, _, _, _) => self.v[x] = self.v[x].wrapping_add(nn),
            (0x8, _, _, _) => self.exec_8xy(opcode, x, y, n4),
            (0x9, _, _, 0) => self.skip_if(self.v[x] != self.v[y]),
            (0xA, _, _, _) => self.i = nnn,
            (0xB, _, _, _) => {
                self.pc = nnn + (self.v[0] as u16);
            }
            (0xC, _, _, _) => self.v[x] = rand::random::<u8>() & nn,
            (0xD, _, _, _) => self.draw_sprite(x, y, n4),
            (0xE, _, _, _) => self.exec_ex(opcode, x, y, n4),
            (0xF, _, _, _) => self.exec_fx(opcode, x, y, n4),
            _ => panic!("unknown opcode: {opcode:#06X}"),
        }
    }

    fn exec_8xy(&mut self, opcode: u16, x: usize, y: usize, n4: u8) {
        match n4 {
            0 => self.v[x] = self.v[y],
            1 => self.v[x] |= self.v[y],
            2 => self.v[x] &= self.v[y],
            3 => self.v[x] ^= self.v[y],
            4 => {
                let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = res;
                self.v[0xF] = overflow as u8;
            }
            5 => {
                let (res, borrow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = res;
                self.v[0xF] = (!borrow) as u8;
            }
            6 => {
                let lsb = self.v[x] & 1;
                self.v[x] >>= 1;
                self.v[0xF] = lsb;
            }
            7 => {
                let (res, borrow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = res;
                self.v[0xF] = (!borrow) as u8;
            }
            0xE => {
                let msb = (self.v[x] >> 7) & 1;
                self.v[x] <<= 1;
                self.v[0xF] = msb;
            }
            _ => panic!("unknown 8xy opcode: {opcode:#06X}"),
        }
    }

    fn exec_ex(&mut self, opcode: u16, x: usize, y: usize, n4: u8) {
        let key = (self.v[x] & 0x0F) as usize;
        match (y, n4) {
            (0x9, 0xE) => {
                self.skip_if(self.keypad[key]);
            }
            (0xA, 0x1) => {
                self.skip_if(!self.keypad[key]);
            }
            _ => panic!("unknown Ex opcode: {opcode:#06X}"),
        }
    }

    fn exec_fx(&mut self, opcode: u16, x: usize, y: usize, n4: u8) {
        match (y, n4) {
            (0x0, 0x7) => self.v[x] = self.delay_timer,
            (0x0, 0xA) => {
                for pressed in 0..self.keypad.len() {
                    if self.keypad[pressed] {
                        self.v[x] = pressed as u8;
                        return;
                    }
                }
                self.pc -= 2;
            }
            (0x1, 0x5) => self.delay_timer = self.v[x],
            (0x1, 0x8) => self.sound_timer = self.v[x],
            (0x1, 0xE) => self.i = self.i.wrapping_add(self.v[x] as u16),
            (0x2, 0x9) => self.i = (Self::FONTSET_ADDR as u16) + ((self.v[x] & 0x0F) as u16) * 5,
            (0x3, 0x3) => {
                let i = self.i as usize;
                self.memory[i] = self.v[x] / 100;
                self.memory[i + 1] = self.v[x] / 10 % 10;
                self.memory[i + 2] = self.v[x] % 10;
            }
            (0x5, 0x5) => {
                for offset in 0..=x {
                    self.memory[self.i as usize + offset] = self.v[offset];
                }
            }
            (0x6, 0x5) => {
                for offset in 0..=x {
                    self.v[offset] = self.memory[self.i as usize + offset];
                }
            }
            _ => panic!("unknown Fx opcode: {opcode:#06X}"),
        }
    }

    fn draw_sprite(&mut self, x: usize, y: usize, n4: u8) {
        let x_start = (self.v[x] as usize) % Self::SCREEN_WIDTH;
        let y_start = (self.v[y] as usize) % Self::SCREEN_HEIGHT;
        self.v[0xF] = 0;

        for row in 0..(n4 as usize) {
            let py = y_start + row;
            if py >= Self::SCREEN_HEIGHT {
                break;
            }
            let sprite_byte = self.memory[self.i as usize + row];

            for col in 0..8 {
                let px = x_start + col;
                if px >= Self::SCREEN_WIDTH {
                    break;
                }
                if sprite_byte & (0x80 >> col) != 0 {
                    let idx = py * Self::SCREEN_WIDTH + px;
                    if self.display[idx] {
                        self.v[0xF] = 1;
                    }
                    self.display[idx] ^= true;
                }
            }
        }
    }

    fn skip_if(&mut self, cond: bool) {
        if cond {
            self.pc += 2;
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn display(&self) -> &[bool] {
        &self.display
    }

    pub fn set_key(&mut self, key: usize, pressed: bool){
        self.keypad[key] = pressed;
    }
}

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

    let mut buffer = [0u32; Chip8::SCREEN_WIDTH*Chip8::SCREEN_HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape){
        for (chip8_key, &host_key) in KEYMAP.iter().enumerate() {
            emulator.set_key(chip8_key, window.is_key_down(host_key));
        }
        for _ in 0..10{
            emulator.cycle();
        }

        emulator.tick_timers();

        for (dst, &on) in buffer.iter_mut().zip(emulator.display()){
            *dst = if on {0x00FF_FFFF} else {0x0000_0000};
        }

        window.update_with_buffer(&buffer, Chip8::SCREEN_WIDTH, Chip8::SCREEN_HEIGHT).expect("failed to update window");
    }
}
