// Enumerable containing Chip 8 Variants
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Chip8Variant {
    Chip8,
    SuperChip,
}

// Quirks struct that contains all the differences in instructions
// between Chip 8 Variants
pub struct Quirks {
    // 8XY1, 8XY2, 8XY3 Quirk
    pub vf_reset: bool,
    // FX55, FX65 Quirk
    pub memory: bool,
    // 8XY6, 8XYE Quirk
    pub shifting: bool,
    // BNNN Quirk
    pub jumping: bool,
}

impl Quirks {
    pub fn new_variant(variant: Chip8Variant) -> Self {
        match variant {
            Chip8Variant::Chip8 => Self {
                vf_reset: true,
                memory: true,
                shifting: false,
                jumping: false,
            },
            Chip8Variant::SuperChip => Self {
                vf_reset: false,
                memory: false,
                shifting: true,
                jumping: true,
            },
        }
    }
}

// Enumerable containing resolution modes
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    LoRes,
    HiRes,
}

// Set the ticks per frame based on Chip 8 Variant
pub fn ticks_per_frame(variant: Chip8Variant) -> usize {
    match variant {
        Chip8Variant::SuperChip => 16,
        _ => 8,
    }
}
