#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Chip8Variant {
    Chip8,
    SuperChip,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    LoRes,
    HiRes,
}
