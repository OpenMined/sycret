use aes::cipher::generic_array::GenericArray;
use aes::Aes128;
use aes::{BlockEncrypt, NewBlockCipher};
use std::slice;

use super::stream::Prg;
use super::L;

pub fn share_leaf(mask_a: u32, mask_b: u32, share_bit: u8, flip_bit: u8) -> u32 {
    let mut leaf = mask_b.wrapping_sub(mask_a).wrapping_add(share_bit as u32);
    if flip_bit == 1 {
        leaf = 0u32.wrapping_sub(leaf);
    }
    leaf
}

pub fn compute_out(mask: u32, leaf: u32, tau: u8, flip_bit: u8) -> u32 {
    let mut out: u32 = match tau {
        1 => leaf.wrapping_add(mask),
        _ => mask,
    };
    if flip_bit == 1 {
        out = 0u32.wrapping_sub(out);
    }
    out
}

pub fn bit_decomposition_u32(alpha: u32) -> Vec<u8> {
    let mut alpha_bits: Vec<u8> = Vec::new();
    // Most significant bits first
    for j in (0u8..32).rev() {
        alpha_bits.push((alpha >> j) as u8 & 1);
    }
    alpha_bits
}

/// # Safety
/// De-referencing raw pointer
pub unsafe fn write_aes_key_to_raw_line(aes_key: u128, key_line_pointer: *mut u8) {
    // Cast the output line to a raw pointer.
    let out_ptr: *mut [u8; 16] =
        slice::from_raw_parts_mut(key_line_pointer, 16).as_mut_ptr() as *mut [u8; 16];
    // Get a mutable reference.
    let out_ref: &mut [u8; 16] = &mut *out_ptr;
    // Write the key.
    out_ref.copy_from_slice(&aes_key.to_le_bytes());
}

/// # Safety
/// De-referencing raw pointer
pub unsafe fn read_aes_key_from_raw_line(key_line_pointer: *const u8) -> u128 {
    let key_ptr: *const [u8; 16] =
        slice::from_raw_parts(key_line_pointer, 16).as_ptr() as *const [u8; 16];
    let key: u128 = u128::from_le_bytes(*key_ptr);
    key
}

pub struct Mmo {
    // pub expansion_factor: usize,
    pub ciphers: Vec<Aes128>,
}

// TODO: hardcode the default keys
impl Prg for Mmo {
    fn from_slice(aes_keys: &[u128]) -> Mmo {
        let mut ciphers = vec![];
        for key in aes_keys {
            ciphers.push(aes::Aes128::new(GenericArray::from_slice(
                &key.to_le_bytes(),
            )));
        }
        Mmo {
            // expansion_factor: ciphers.len(),
            ciphers,
        }
    }

    fn from_vec(aes_keys: &[u128]) -> Mmo {
        let mut ciphers = vec![];
        for key in aes_keys {
            ciphers.push(aes::Aes128::new(GenericArray::from_slice(
                &key.to_le_bytes(),
            )));
        }
        Mmo {
            // expansion_factor: ciphers.len(),
            ciphers,
        }
    }

    fn expand(&mut self, seed: u128) -> Vec<u128> {
        // NOTE: to improve performance, try:
        // - const generic instead of Vec?
        // - inplace as much as possible, u8 rather than u128?
        let mut output = Vec::new();
        let mut output_array = [0u8; L];
        let seed_slice = seed.to_le_bytes();
        // Matyas-Meyer-Oseas with AES (ECB)
        for cipher in &self.ciphers {
            let mut block = GenericArray::clone_from_slice(&seed_slice);
            cipher.encrypt_block(&mut block);
            // XOR byte by byte
            for k in 0..L {
                output_array[k] = block[k] ^ seed_slice[k];
            }
            output.push(u128::from_le_bytes(output_array));
        }
        output
    }
}
