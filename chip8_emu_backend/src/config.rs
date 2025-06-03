#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Chip8Variant {
    Chip8,
    SuperChip,
}

pub struct Quirks {
    // 8XY1, 8XY2, 8XY3 Quirk
    pub vf_reset: bool,
    // FX55, FX65 Quirk
    pub memory: bool,
    // 8XY6, 8XYE Quirk
    pub shifting: bool,
}

impl Quirks {
    pub fn new_variant(variant: Chip8Variant) -> Self {
        match variant {
            Chip8Variant::Chip8 => Self {
                vf_reset: true,
                memory: true,
                shifting: false,
            },
            Chip8Variant::SuperChip => Self {
                vf_reset: false,
                memory: false,
                shifting: true,
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    LoRes,
    HiRes,
}
