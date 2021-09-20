//!
//! Equality keys tailored for AriaNN
//!

use rand::Rng;
use std::convert::TryInto;

use std::slice;

use crate::fss::dpf::{g, generate_cw_from_seeds};
use crate::stream::{FSSKey, Prg, RawKey};
use crate::utils::{bit_decomposition_u32, compute_out};
use crate::{L, N};

#[derive(Debug)]
pub struct EqKey {
    pub alpha_share: u32,
    pub s: u128,
    pub cw: [u128; N * 8],
    pub t_l: [u8; N * 8],
    pub t_r: [u8; N * 8],
    pub cw_leaf: u32,
}

impl RawKey for EqKey {
    const KEY_LEN: usize = 621;

    unsafe fn to_raw_line(&self, key_pointer: *mut u8) {
        // Cast the output line to a raw pointer.
        let out_ptr: *mut [u8; Self::KEY_LEN] =
            slice::from_raw_parts_mut(key_pointer, Self::KEY_LEN).as_mut_ptr()
                as *mut [u8; Self::KEY_LEN];
        // Get a mutable reference.
        let out_ref: &mut [u8; Self::KEY_LEN] = &mut *out_ptr;
        // Write the key.
        write_key_to_array(&self, out_ref);
    }

    unsafe fn from_raw_line(key_pointer: *const u8) -> Self {
        let key_ptr = slice::from_raw_parts(key_pointer, Self::KEY_LEN).as_ptr()
            as *const [u8; Self::KEY_LEN];
        read_key_from_array(&*key_ptr)
    }
}

impl FSSKey for EqKey {
    fn generate_keypair(prg: &mut impl Prg) -> (Self, Self) {
        // Thread randomness for parallelization.
        let mut rng = rand::thread_rng();

        // Random point on which we will check equality.
        let alpha: u32 = rng.gen();

        // Initialize seeds.
        let s_a: u128 = rng.gen();
        let s_b: u128 = rng.gen();

        // Memory allocation. We could write inplace instead.
        let mut cw = [0u128; N * 8];
        let mut t_l = [0u8; N * 8];
        let mut t_r = [0u8; N * 8];

        let cw_leaf = generate_cw_from_seeds(prg, alpha, s_a, s_b, &mut cw, &mut t_l, &mut t_r);

        // Secret-share alpha and split the keys between Alice and Bob.
        let mask: u32 = rng.gen();

        // Return a key pair.
        (
            EqKey {
                alpha_share: alpha.wrapping_sub(mask),
                s: s_a,
                cw,
                t_l,
                t_r,
                cw_leaf,
            },
            EqKey {
                alpha_share: mask,
                s: s_b,
                cw,
                t_l,
                t_r,
                cw_leaf,
            },
        )
    }

    fn eval(&self, prg: &mut impl Prg, party_id: u8, x: u32) -> u32 {
        // Initialize the control bit and the seed.
        assert!((party_id == 0u8) || (party_id == 1u8));
        let mut t_i: u8 = party_id;
        let mut s_i: u128 = self.s;

        // Compare the bit decomposition of x with the special path.
        let x_bits: Vec<u8> = bit_decomposition_u32(x);
        for i in 0..(N * 8) {
            let (s_l, t_l, s_r, t_r) = g(prg, s_i);
            let s_cw = self.cw[i];
            let t_cw_l = self.t_l[i];
            let t_cw_r = self.t_r[i];

            // We don't compute the XOR on the side that we don't keep.
            if x_bits[i] == 0u8 {
                // If x[i] = 0, keep left.
                if t_i == 0u8 {
                    s_i = s_l;
                    t_i = t_l;
                } else {
                    s_i = s_l ^ s_cw;
                    t_i = t_l ^ t_cw_l;
                }
            } else {
                // If x[i] = 1, keep right.
                if t_i == 0u8 {
                    s_i = s_r;
                    t_i = t_r;
                } else {
                    s_i = s_r ^ s_cw;
                    t_i = t_r ^ t_cw_r;
                }
            }
        }
        compute_out(s_i as u32, self.cw_leaf, t_i, party_id)
    }
}

///
/// Serialization
///

fn write_key_to_array(key: &EqKey, array: &mut [u8; EqKey::KEY_LEN]) {
    array[0..N].copy_from_slice(&key.alpha_share.to_le_bytes());
    array[N..(N + L)].copy_from_slice(&key.s.to_le_bytes());
    for i in 0..(N * 8) {
        // This key structure is retrocompatible with byte arrays.
        let cw_start = N + L + i * (L + 2);
        let cw_end = N + L + (i + 1) * (L + 2);
        array[cw_start..cw_end - 2].copy_from_slice(&key.cw[i].to_le_bytes());
        array[cw_end - 2] = key.t_l[i];
        array[cw_end - 1] = key.t_r[i];
    }
    // array[EqKey::KEY_LEN - 1] = key.t_leaf;
    let j = EqKey::KEY_LEN - N;
    array[j..j + N].copy_from_slice(&key.cw_leaf.to_le_bytes());
}

fn read_key_from_array(array: &[u8; EqKey::KEY_LEN]) -> EqKey {
    let alpha_share = u32::from_le_bytes(array[0..N].try_into().unwrap());
    let s = u128::from_le_bytes(array[N..(N + L)].try_into().unwrap());

    let mut cw = [0u128; N * 8];
    let mut t_l = [0u8; N * 8];
    let mut t_r = [0u8; N * 8];

    for i in 0..(N * 8) {
        let cw_start = N + L + i * (L + 2);
        let cw_end = N + L + (i + 1) * (L + 2);
        cw[i] = u128::from_le_bytes(array[cw_start..cw_end - 2].try_into().unwrap());
        t_l[i] = array[cw_end - 2];
        t_r[i] = array[cw_end - 1];
    }

    // let t_leaf = array[EqKey::KEY_LEN - 1];
    let j = EqKey::KEY_LEN - N;
    let cw_leaf = u32::from_le_bytes(array[j..j + N].try_into().unwrap());

    EqKey {
        alpha_share,
        s,
        cw,
        t_l,
        t_r,
        cw_leaf,
    }
}
