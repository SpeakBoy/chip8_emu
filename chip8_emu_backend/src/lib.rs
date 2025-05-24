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

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
