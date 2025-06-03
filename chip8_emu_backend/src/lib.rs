pub mod audio;
pub mod config;

pub use audio::AudioManager;
pub use config::{Chip8Variant, DisplayMode, Quirks};
use rand::random;

// 16 sprites for each hexadecimal digit of size 5 bytes each
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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

// 64x32 display
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

// 4 KB of RAM
const RAM_SIZE: usize = 4096;
// 16 V Registers
const NUM_V_REGS: usize = 16;
const STACK_SIZE: usize = 16;
// 16 key input
const NUM_KEYS: usize = 16;

pub struct Cpu {
    // 16-bit program counter
    pc: u16,
    // RAM is in bytes (8 bits) of RAM_SIZE size
    ram: [u8; RAM_SIZE],
    // monochrome display only requires 1 bit values (boolean)
    // for colors (i.e., black is false, white is true)
    screen: Vec<bool>,
    screen_width: usize,
    screen_height: usize,
    // 8-bit registers
    v_reg: [u8; NUM_V_REGS],
    // 16-bit indexing register
    i_reg: u16,
    // 16-bit stack pointer
    sp: u16,
    // 16-bit stack
    stack: [u16; STACK_SIZE],
    // keys are either pressed (true) or not pressed (false)
    keys: [bool; NUM_KEYS],
    // previous keys array for use with FX0A instruction
    prev_keys: [bool; NUM_KEYS],
    // 8-bit delay timer register
    delay_t: u8,
    // 8-bit sound timer register
    sound_t: u8,
    audio: AudioManager,
    variant: Chip8Variant,
    display_mode: DisplayMode,
    quirks: Quirks,
}

// starting address
const START_ADDR: u16 = 0x200;

impl Cpu {
    pub fn new(audio: AudioManager, variant: Chip8Variant) -> Self {
        let quirks = Quirks::new_variant(variant);

        let mut new_cpu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: vec![false; SCREEN_WIDTH * SCREEN_HEIGHT],
            screen_width: SCREEN_WIDTH,
            screen_height: SCREEN_HEIGHT,
            v_reg: [0; NUM_V_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            prev_keys: [false; NUM_KEYS],
            delay_t: 0,
            sound_t: 0,
            audio,
            variant,
            display_mode: DisplayMode::LoRes,
            quirks,
        };

        new_cpu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_cpu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = vec![false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.screen_width = SCREEN_WIDTH;
        self.screen_height = SCREEN_HEIGHT;
        self.v_reg = [0; NUM_V_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.prev_keys = [false; NUM_KEYS];
        self.delay_t = 0;
        self.sound_t = 0;
        self.audio.stop_beep();
        self.display_mode = DisplayMode::LoRes;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode and execute
        self.execute(op);
    }

    pub fn tick_timers(&mut self) {
        if self.delay_t > 0 {
            self.delay_t -= 1;
        }

        if self.sound_t > 0 {
            self.audio.start_beep();
            self.sound_t -= 1;
        } else {
            self.audio.stop_beep();
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.prev_keys[idx] = self.keys[idx];
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn fetch(&mut self) -> u16 {
        let first_byte = self.ram[self.pc as usize] as u16;
        let second_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (first_byte << 8) | second_byte;
        self.pc += 2;
        op
    }

    fn execute(&mut self, op: u16) {
        let digit_1 = (op & 0xF000) >> 12;
        let digit_2 = (op & 0x0F00) >> 8;
        let digit_3 = (op & 0x00F0) >> 4;
        let digit_4 = op & 0x000F;
        let x = digit_2 as usize;
        let y = digit_3 as usize;
        let nn = (op & 0xFF) as u8;
        let nnn = op & 0x0FFF;

        match digit_1 {
            0x0 => match (digit_2, digit_3, digit_4) {
                // 0000 - NOOP
                (0x0, 0x0, 0x0) => return,
                (0x0, 0xC, _) => println!("{:#x}", op),
                // 00E0 - Clear screen
                (0x0, 0xE, 0x0) => {
                    self.screen = vec![false; SCREEN_WIDTH * SCREEN_HEIGHT];
                }
                // 00EE - Return from subroutine
                (0x0, 0xE, 0xE) => {
                    self.pc = self.pop();
                }
                (0x0, 0xF, 0xB) => println!("{:#x}", op),
                (0x0, 0xF, 0xC) => println!("{:#x}", op),
                // 00FD - Exit interpreter
                (0x0, 0xF, 0xD) => std::process::exit(0),
                (0x0, 0xF, 0xE) => println!("{:#x}", op),
                (0x0, 0xF, 0xF) => println!("{:#x}", op),
                _ => panic!("invalid opcode"),
            },
            0x1 => {
                // 1NNN - Jump
                self.pc = nnn;
            }
            0x2 => {
                // 2NNN - Call subroutine
                self.push(self.pc);
                self.pc = nnn;
            }
            0x3 => {
                // 3XNN - Skip next if VX == NN
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            0x4 => {
                // 4XNN - Skip next if VX != NN
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }
            0x5 => match digit_4 {
                // 5XY0 - Skip next if VX == VY
                0x0 => {
                    if self.v_reg[x] == self.v_reg[y] {
                        self.pc += 2;
                    }
                }
                _ => panic!("invalid opcode"),
            },
            0x6 => {
                // 6XNN - VX = NN
                self.v_reg[x] = nn;
            }
            0x7 => {
                // 7XNN - VX += NN
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            0x8 => match digit_4 {
                // 8XY0 - VX = VY
                0x0 => {
                    self.v_reg[x] = self.v_reg[y];
                }
                // 8XY1 - VX |= VY (OR)
                0x1 => {
                    self.v_reg[x] |= self.v_reg[y];
                    if self.quirks.vf_reset {
                        self.v_reg[0xF] = 0;
                    }
                }
                // 8XY2 - VX &= VY (AND)
                0x2 => {
                    self.v_reg[x] &= self.v_reg[y];
                    if self.quirks.vf_reset {
                        self.v_reg[0xF] = 0;
                    }
                }
                // 8XY3 - VX ^= VY (XOR)
                0x3 => {
                    self.v_reg[x] ^= self.v_reg[y];
                    if self.quirks.vf_reset {
                        self.v_reg[0xF] = 0;
                    }
                }
                // 8XY4 - VX += VY
                0x4 => {
                    let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                    let new_vf = if carry { 1 } else { 0 };

                    self.v_reg[x] = new_vx;
                    self.v_reg[0xF] = new_vf;
                }
                // 8XY5 - VX -= VY
                0x5 => {
                    let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                    let new_vf = if borrow { 0 } else { 1 };

                    self.v_reg[x] = new_vx;
                    self.v_reg[0xF] = new_vf;
                }
                // 8XY6 - VX >>= 1
                0x6 => {
                    self.v_reg[x] = self.v_reg[y];
                    let lsb = self.v_reg[x] & 1;
                    self.v_reg[x] >>= 1;
                    self.v_reg[0xF] = lsb;
                }
                // 8XY7 - VX = VY - VX
                0x7 => {
                    let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                    let new_vf = if borrow { 0 } else { 1 };

                    self.v_reg[x] = new_vx;
                    self.v_reg[0xF] = new_vf;
                }
                // 8XYE - VX <<= 1
                0xE => {
                    self.v_reg[x] = self.v_reg[y];
                    let msb = (self.v_reg[x] >> 7) & 1;
                    self.v_reg[x] <<= 1;
                    self.v_reg[0xF] = msb;
                }
                _ => panic!("invalid opcode"),
            },
            0x9 => match digit_4 {
                // 9XY0 - Skip next if VX != VY
                0x0 => {
                    if self.v_reg[x] != self.v_reg[y] {
                        self.pc += 2;
                    }
                }
                _ => panic!("invalid opcode"),
            },
            0xA => {
                // ANNN - I = NNN
                self.i_reg = nnn;
            }
            0xB => {
                // BNNN - Jump to V0 + NNN
                self.pc = (self.v_reg[0] as u16) + nnn;
            }
            0xC => {
                // CXNN - rand() & NN
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }
            0xD => {
                // DXYN - Draw Sprite

                // Get (x, y) coords for sprite, wrap before drawing.
                let x_coord = self.v_reg[digit_2 as usize] as u16 % SCREEN_WIDTH as u16;
                let y_coord = self.v_reg[digit_3 as usize] as u16 % SCREEN_HEIGHT as u16;
                // Last digit determines height of sprite
                let num_rows = digit_4;
                // Keep track if any pixels were flipped
                let mut flipped = false;
                // Iterate over each row of the sprite
                for y_line in 0..num_rows {
                    let y = y_coord + y_line;
                    if y >= SCREEN_HEIGHT as u16 {
                        continue; // Clip bottom
                    }

                    // Determine where row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // Iterate over column in current row
                    for x_line in 0..8 {
                        // Use mask to fetch current pixel's bit. Flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = x_coord + x_line;
                            if x >= SCREEN_WIDTH as u16 {
                                continue; // Clip right
                            }

                            // Get pixel's index for the 1D screen array
                            let idx = x as usize + SCREEN_WIDTH * y as usize;
                            // Check if pixel will be flipped and set
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                // Populate VF register
                self.v_reg[0xF] = flipped as u8;
            }
            0xE => match (digit_3, digit_4) {
                // EX9E - Skip if Key Pressed
                (0x9, 0xE) => {
                    let vx = self.v_reg[x];
                    let key = self.keys[vx as usize];
                    if key {
                        self.pc += 2;
                    }
                }
                // EXA1 - Skip if Key Not Pressed
                (0xA, 0x1) => {
                    let vx = self.v_reg[x];
                    let key = self.keys[vx as usize];
                    if !key {
                        self.pc += 2;
                    }
                }
                _ => panic!("invalid opcode"),
            },
            0xF => match (digit_3, digit_4) {
                // FXO7 - VX = DT
                (0x0, 0x7) => {
                    self.v_reg[x] = self.delay_t;
                }
                // FX0A - Wait for Key Press (Release)
                (0x0, 0xA) => {
                    let mut released = false;
                    for i in 0..self.keys.len() {
                        if !self.keys[i] && self.prev_keys[i] {
                            self.v_reg[x] = i as u8;
                            released = true;
                            break;
                        }
                    }

                    if !released {
                        // Redo opcode
                        self.pc -= 2;
                    }
                }
                // FX15 - DT = VX
                (0x1, 0x5) => {
                    self.delay_t = self.v_reg[x];
                }
                // FX18 - ST = VX
                (0x1, 0x8) => {
                    self.sound_t = self.v_reg[x];
                }
                // FX1E - I += VX
                (0x1, 0xE) => {
                    let vx = self.v_reg[x] as u16;
                    self.i_reg = self.i_reg.wrapping_add(vx);
                }
                // FX29 - Set I to Sprite for Digit VX
                (0x2, 0x9) => {
                    let char = self.v_reg[x] as u16;
                    self.i_reg = char * 5;
                }
                (0x3, 0x0) => println!("{:#x}", op),
                // FX33 - I = BCD of VX
                (0x3, 0x3) => {
                    let vx = self.v_reg[x];

                    // Get the hundreds digit of VX
                    let hundreds = vx / 100;
                    // Get the tens digit of VX
                    let tens = (vx / 10) % 10;
                    // Get the ones digit of VX
                    let ones = vx % 10;

                    self.ram[self.i_reg as usize] = hundreds;
                    self.ram[(self.i_reg + 1) as usize] = tens;
                    self.ram[(self.i_reg + 2) as usize] = ones;
                }
                // FX55 - Store V0 to VX into I
                (0x5, 0x5) => {
                    if self.quirks.save_load_increment_i {
                        for idx in 0..=x {
                            self.ram[self.i_reg as usize] = self.v_reg[idx];
                            self.i_reg += 1;
                        }
                    } else {
                        for idx in 0..=x {
                            self.ram[self.i_reg as usize + idx] = self.v_reg[idx];
                        }
                    }
                }
                // FX65 - Load I into V0 to VX
                (0x6, 0x5) => {
                    if self.quirks.save_load_increment_i {
                        for idx in 0..=x {
                            self.v_reg[idx] = self.ram[self.i_reg as usize];
                            self.i_reg += 1;
                        }
                    } else {
                        for idx in 0..=x {
                            self.v_reg[idx] = self.ram[self.i_reg as usize + idx];
                        }
                    }
                }
                (0x7, 0x5) => println!("{:#x}", op),
                (0x8, 0x5) => println!("{:#x}", op),
                _ => panic!("invalid opcode"),
            },
            _ => panic!("invalid hexadecimal digit"),
        }
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
