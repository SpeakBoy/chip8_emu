#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Chip8Variant {
    Chip8,
    SuperChip,
}

pub struct Quirks {
    pub vf_reset: bool,
}

impl Quirks {
    pub fn new_variant(variant: Chip8Variant) -> Self {
        match variant {
            Chip8Variant::Chip8 => Self { vf_reset: true },
            Chip8Variant::SuperChip => Self { vf_reset: false },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    LoRes,
    HiRes,
}
