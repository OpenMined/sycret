use std::slice;

pub fn bit_decomposition_u32(alpha: u32) -> Vec<u8> {
    let mut alpha_bits: Vec<u8> = Vec::new();
    // Most significant bits first
    for j in (0u8..32).rev() {
        alpha_bits.push((alpha >> j) as u8 & 1);
    }
    alpha_bits
}

pub unsafe fn write_aes_key_to_raw_line(aes_key: u128, key_line_pointer: *mut u8) {
    // Cast the output line to a raw pointer.
    let out_ptr: *mut [u8; 16] =
        slice::from_raw_parts_mut(key_line_pointer, 16).as_mut_ptr() as *mut [u8; 16];
    // Get a mutable reference.
    let out_ref: &mut [u8; 16] = &mut *out_ptr;
    // Write the key.
    out_ref.copy_from_slice(&aes_key.to_le_bytes());
}

pub unsafe fn read_aes_key_from_raw_line(key_line_pointer: *const u8) -> u128 {
    let key_ptr: *const [u8; 16] =
        slice::from_raw_parts(key_line_pointer, 16).as_ptr() as *const [u8; 16];
    let key: u128 = u128::from_le_bytes(*key_ptr);
    key
}
