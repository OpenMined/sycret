pub trait FSSKey {
    const key_len: usize;

    unsafe fn from_raw_line(raw_line_pointer: *const u8) -> Self;

    unsafe fn to_raw_line(&self, raw_line_pointer: *mut u8);

    fn eval(&self, prg: &impl PRG, party_id: u8, x: u32) -> u8;

    fn generate_keypair(prg: &impl PRG) -> Self, Self
};

// Keyed PRG
pub trait PRG {

    const expansion_factor: usize;

    fn from_slice(&[u128]) -> Self;

    fn expand(&mut self, seed: u128) -> [u128, usize];

    // TODO: key type, read/write state to line
};