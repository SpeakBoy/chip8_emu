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
                // BEEP
            }
            self.sound_t -= 1;
        }
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
            }
            // VX &= VY (AND)
            (8, _, _, 2) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }
            // VX ^= VY (XOR)
            (8, _, _, 3) => {
                let x = digit_2 as usize;
                let y = digit_3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
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
