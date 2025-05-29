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
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
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
    // 8-bit delay timer register
    delay_t: u8,
    // 8-bit sound timer register
    sound_t: u8,
}

// starting address
const START_ADDR: u16 = 0x200;

impl Cpu {
    pub fn new() -> Self {
        let mut new_cpu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_V_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            delay_t: 0,
            sound_t: 0,
        };

        new_cpu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_cpu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_V_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.delay_t = 0;
        self.sound_t = 0;
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
            if self.sound_t == 1 {
                // BEEP (TODO)
            }
            self.sound_t -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
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

        match (digit_1, digit_2, digit_3, digit_4) {
            // Nop
            (0, 0, 0, 0) => return,
            // Clear screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            // Return from subroutine
            (0, 0, 0xE, 0xE) => {
                let return_addr = self.pop();
                self.pc = return_addr;
            }
            // Jump
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }
            // Call subroutine
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            // Skip next if VX == NN
            (3, _, _, _) => {
                let x = digit_2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            // Skip next if VX != NN
            (4, _, _, _) => {
                let x = digit_2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }
            // Skip next if VX == VY
            (5, _, _, 0) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // VX = VN
            (6, _, _, _) => {
                let x = digit_2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            }
            // VX += VN
            (7, _, _, _) => {
                let x = digit_2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            // VX = VY
            (8, _, _, 0) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }
            // VX |= VY (OR)
            (8, _, _, 1) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;
                self.v_reg[x] |= self.v_reg[y];
                self.v_reg[0xF] = 0;
            }
            // VX &= VY (AND)
            (8, _, _, 2) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;
                self.v_reg[x] &= self.v_reg[y];
                self.v_reg[0xF] = 0;
            }
            // VX ^= VY (XOR)
            (8, _, _, 3) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
                self.v_reg[0xF] = 0;
            }
            // VX += VY
            (8, _, _, 4) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            // VX -= VY
            (8, _, _, 5) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            // VX >>= 1
            (8, _, _, 6) => {
                let x = digit_2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            }
            // VX = VY - VX
            (8, _, _, 7) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            // VX <<= 1
            (8, _, _, 0xE) => {
                let x = digit_2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            }
            // Skip if VX != VY
            (9, _, _, 0) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;
                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }
            // Jump to V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }
            // VX = rand() & NN
            (0xC, _, _, _) => {
                let x = digit_2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }
            // Draw Sprite
            (0xD, _, _, _) => {
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
                            // Apply modulo to wrap sprites around screen
                            let x = x_coord + x_line;

                            if x >= SCREEN_WIDTH as u16 {
                                continue;
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
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }
            // Skip if Key Pressed
            (0xE, _, 9, 0xE) => {
                let x = digit_2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            }
            // Skip if Key Not Pressed
            (0xE, _, 0xA, 1) => {
                let x = digit_2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            }
            // VX = DT
            (0xF, _, 0, 7) => {
                let x = digit_2 as usize;
                self.v_reg[x] = self.delay_t;
            }
            // Wait for Key Press
            (0xF, _, 0, 0xA) => {
                let x = digit_2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    // Redo opcode
                    self.pc -= 2;
                }
            }
            // DT = VX
            (0xF, _, 1, 5) => {
                let x = digit_2 as usize;
                self.delay_t = self.v_reg[x];
            }
            // ST = VX
            (0xF, _, 1, 8) => {
                let x = digit_2 as usize;
                self.sound_t = self.v_reg[x];
            }
            // I += VX
            (0xF, _, 1, 0xE) => {
                let x = digit_2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            // Set I to Font Address
            (0xF, _, 2, 9) => {
                let x = digit_2 as usize;
                let char = self.v_reg[x] as u16;
                self.i_reg = char * 5;
            }
            // I = BCD of VX
            (0xF, _, 3, 3) => {
                let x = digit_2 as usize;
                let vx = self.v_reg[x] as f32;

                // Get the hundreds digit of VX
                let hundreds = (vx / 100.0).floor() as u8;
                // Get the tens digit of VX
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                // Get the ones digit of VX
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            // Store V0 to VX into I
            (0xF, _, 5, 5) => {
                let x = digit_2 as usize;
                for idx in 0..=x {
                    self.ram[self.i_reg as usize] = self.v_reg[idx];
                    self.i_reg += 1;
                }
            }
            // Load I into V0 to VX
            (0xF, _, 6, 5) => {
                let x = digit_2 as usize;
                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[self.i_reg as usize];
                    self.i_reg += 1;
                }
            }
            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
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
