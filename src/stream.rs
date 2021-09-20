//!
//! Utilities to iterate over Numpy arrays
//!

use std::slice;

use crate::eq::EqKey;
use crate::le::LeKey;
use crate::utils::Mmo;
use crate::N;

pub trait FSSKey: Sized {
    fn eval(&self, prg: &mut impl Prg, party_id: u8, x: u32) -> u32;

    fn generate_keypair(prg: &mut impl Prg) -> (Self, Self);
}

pub trait RawKey: Sized {
    const KEY_LEN: usize;

    unsafe fn from_raw_line(raw_line_pointer: *const u8) -> Self;

    unsafe fn to_raw_line(&self, raw_line_pointer: *mut u8);
}

// Keyed Prg
pub trait Prg {
    fn from_slice(key: &[u128]) -> Self;

    fn from_vec(key: &Vec<u128>) -> Self;

    // NOTE: Rust Stable does not have const generics
    // const expansion_factor: usize;
    // fn expand(&mut self, seed: u128) -> [u128; Self::expansion_factor];
    fn expand(&mut self, seed: u128) -> Vec<u128>;

    // TODO: key type, read/write state to line
}

pub fn generate_key_stream(
    aes_keys: &Vec<u128>,
    _stream_id: usize,
    stream_length: usize,
    key_a_pointer: usize,
    key_b_pointer: usize,
    op_id: usize,
) {
    // Generate keys in sequence
    let key_a_p = key_a_pointer as *mut u8;
    let key_b_p = key_b_pointer as *mut u8;

    // TODO: def. Impl Prg.
    let mut prg = Mmo::from_vec(aes_keys);

    for line_counter in 0..stream_length {
        if op_id == 0 {
            let (key_a, key_b) = EqKey::generate_keypair(&mut prg);
            let key_len = EqKey::KEY_LEN;
            unsafe {
                &key_a.to_raw_line(key_a_p.add(key_len * line_counter));
                &key_b.to_raw_line(key_b_p.add(key_len * line_counter));
            }
        } else {
            let (key_a, key_b) = LeKey::generate_keypair(&mut prg);
            let key_len = LeKey::KEY_LEN;
            unsafe {
                &key_a.to_raw_line(key_a_p.add(key_len * line_counter));
                &key_b.to_raw_line(key_b_p.add(key_len * line_counter));
            }
        }
    }
}

pub fn eval_key_stream(
    party_id: u8,
    aes_keys: &Vec<u128>,
    _stream_id: usize,
    stream_length: usize,
    x_pointer: usize,
    key_pointer: usize,
    result_pointer: usize,
    op_id: usize,
) {
    assert!((party_id == 0u8) || (party_id == 1u8));

    let mut prg = Mmo::from_vec(aes_keys);

    // Read, eval, write line by line

    let x_pointer_p = x_pointer as *const u8;
    let key_pointer_p = key_pointer as *const u8;
    let result_ptr_p = result_pointer as *mut i64;

    for line_counter in 0..stream_length {
        // Read key and value to evaluate
        unsafe {
            let x_ptr: *const [u8; N] = slice::from_raw_parts(x_pointer_p.add(N * line_counter), N)
                .as_ptr() as *const [u8; N];
            let x: u32 = u32::from_le_bytes(*x_ptr);

            if op_id == 0 {
                let key = EqKey::from_raw_line(key_pointer_p.add(EqKey::KEY_LEN * line_counter));
                let result: u32 = key.eval(&mut prg, party_id, x);
                *(result_ptr_p.add(line_counter)) = result as i64;
            } else {
                let key = LeKey::from_raw_line(key_pointer_p.add(LeKey::KEY_LEN * line_counter));
                let result: u32 = key.eval(&mut prg, party_id, x);
                *(result_ptr_p.add(line_counter)) = result as i64;
            }
        }
    }
}
