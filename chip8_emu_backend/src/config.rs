#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Chip8Variant {
    Chip8,
    SuperChip,
}

pub struct Quirks {
    // 8XY1, 8XY2, 8XY3 Quirk
    pub vf_reset: bool,
    // FX55, FX65 Quirk
    pub save_load_increment_i: bool,
}

impl Quirks {
    pub fn new_variant(variant: Chip8Variant) -> Self {
        match variant {
            Chip8Variant::Chip8 => Self {
                vf_reset: true,
                save_load_increment_i: true,
            },
            Chip8Variant::SuperChip => Self {
                vf_reset: false,
                save_load_increment_i: false,
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    LoRes,
    HiRes,
}
