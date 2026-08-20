#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub const fn new() -> Self {
        Self([0u8; 6])
    }

    pub const fn from_parts(high: u32, low: u32) -> Self {
        Self([
            low as u8,
            (low >> 8) as u8,
            (low >> 16) as u8,
            (low >> 24) as u8,
            high as u8,
            (high >> 8) as u8,
        ])
    }
}
