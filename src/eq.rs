use aesni::cipher::generic_array::GenericArray;
use aesni::cipher::{NewStreamCipher, StreamCipher, SyncStreamCipher};
use aesni::{Aes128, Aes128Ctr};

use rand::{thread_rng, Rng};
use std::convert::TryInto;
use std::fmt;
use std::num::Wrapping;
use std::slice;

use super::stream::{FSSKey, PRG};
use super::utils::{bit_decomposition_u32, compute_out, share_leaf, MMO};
use super::{L, N};

#[derive(Debug)]
pub struct EqKey {
    pub alpha_share: u32,
    pub s: u128,
    pub cw: [u128; N * 8],
    pub t_l: [u8; N * 8],
    pub t_r: [u8; N * 8],
    pub cw_leaf: u32,
}

impl FSSKey for EqKey {
    const key_len: usize = 597;

    unsafe fn to_raw_line(&self, key_pointer: *mut u8) {
        // Cast the output line to a raw pointer.
        let out_ptr: *mut [u8; Self::key_len] =
            slice::from_raw_parts_mut(key_pointer, Self::key_len).as_mut_ptr()
                as *mut [u8; Self::key_len];
        // Get a mutable reference.
        let out_ref: &mut [u8; Self::key_len] = &mut *out_ptr;
        // Write the key.
        write_key_to_array(&self, out_ref);
    }

    unsafe fn from_raw_line(key_pointer: *const u8) -> Self {
        let key_ptr = slice::from_raw_parts(key_pointer, Self::key_len).as_ptr()
            as *const [u8; Self::key_len];
        read_key_from_array(&*key_ptr)
    }

    fn generate_keypair(prg: &mut impl PRG) -> (Self, Self) {
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

    fn eval(&self, prg: &mut impl PRG, party_id: u8, x: u32) -> u32 {
        // Initialize the control bit and the seed.
        assert!((party_id == 0u8) || (party_id == 1u8));
        let mut t_i: u8 = party_id;
        let mut s_i: u128 = self.s;

        // Compare the bit decomposition of x with the special path.
        let x_bits: Vec<u8> = bit_decomposition_u32(x);
        for i in 0..(N * 8) {
            let (s_l, t_l, s_r, t_r) = g_127_tuple_aes_u128(s_i);
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
/// Internal deterministic function
///
// #[flame]
fn generate_cw_from_seeds(
    prg: &mut impl PRG,
    alpha: u32,
    s_a: u128,
    s_b: u128,
    cw: &mut [u128; N * 8],
    t_l: &mut [u8; N * 8],
    t_r: &mut [u8; N * 8],
) -> u32 {
    // Initialize control bits.
    let mut t_a_i = 0u8;
    let mut t_b_i = 1u8;

    // Seeds at level i.
    let mut s_a_i: u128 = s_a;
    let mut s_b_i: u128 = s_b;

    // Iterate over the bits of alpha
    let alpha_bits = bit_decomposition_u32(alpha);

    for i in 0..(N * 8) {
        // Keep only 1 bit instead of a byte for t_l and t_r (not optimal)
        let (s_a_l, t_a_l, s_a_r, t_a_r) = g_127_tuple_aes_u128(s_a_i);
        let (s_b_l, t_b_l, s_b_r, t_b_r) = g_127_tuple_aes_u128(s_b_i);

        // Keep left if a_i = 0, keep right if a_i = 1.
        let (s_a_keep, s_a_loose, t_a_keep) = match alpha_bits[i] {
            0u8 => (s_a_l, s_a_r, t_a_l),
            _ => (s_a_r, s_a_l, t_a_r),
        };
        let (s_b_keep, s_b_loose, t_b_keep) = match alpha_bits[i] {
            0u8 => (s_b_l, s_b_r, t_b_l),
            _ => (s_b_r, s_b_l, t_b_r),
        };

        let t_cw_l = t_a_l ^ t_b_l ^ alpha_bits[i] ^ 1u8;
        let t_cw_r = t_a_r ^ t_b_r ^ alpha_bits[i];
        let t_cw_keep = match alpha_bits[i] {
            0u8 => t_cw_l,
            _ => t_cw_r,
        };

        // Optimized DPF: re-use the randomness we didn't keep to seed next round.
        cw[i] = s_a_loose ^ s_b_loose;
        t_l[i] = t_cw_l;
        t_r[i] = t_cw_r;

        // For Alice: update the seed and value bit for next bit.
        if t_a_i == 0 {
            // Xoring with t_a_i * cw[i][0..L] does nothing.
            s_a_i = s_a_keep;
            t_a_i = t_a_keep;
        } else {
            s_a_i = s_a_keep ^ cw[i];
            t_a_i = t_a_keep ^ t_cw_keep;
        }

        // Same update for Bob.
        if t_b_i == 0 {
            s_b_i = s_b_keep;
            t_b_i = t_b_keep;
        } else {
            s_b_i = s_b_keep ^ cw[i];
            t_b_i = t_b_keep ^ t_cw_keep;
        }
    }
    // We only need 32 bits to make a sharing of 1
    share_leaf(s_a_i as u32, s_b_i as u32, 1, t_b_i)
}

///
/// A "slow" PRG before we try MMO here too
///
pub fn g_127_tuple_aes_u128(seed: u128) -> (u128, u8, u128, u8) {
    assert_eq!(L, 128 / 8);

    // Use the seed as the symmetric key.
    let seed_slice = seed.to_le_bytes();
    let key = GenericArray::from_slice(&seed_slice);

    // The key is not reused for other data so we don't need a unique counter.
    let nonce = GenericArray::from_slice(&[0u8; 16]);
    let mut cipher = aesni::Aes128Ctr::new(&key, &nonce);

    // Fixed plaintext.
    let mut output = [0u8; 32];

    // Encrypt the plaintext inplace.
    cipher.apply_keystream(&mut output);
    let left_bytes = &output[0..L];
    let right_bytes = &output[L..(2 * L)];

    // Chop the most significant bit for the control bit.
    let mut s_l = u128::from_le_bytes(left_bytes.try_into().unwrap());
    let t_l = output[0] & 1u8;
    s_l = s_l >> 1 << 1;
    let mut s_r = u128::from_le_bytes(right_bytes.try_into().unwrap());
    let t_r = output[L] & 1u8;
    s_r = s_r >> 1 << 1;

    (s_l, t_l, s_r, t_r)
}

///
/// Serialization
///

fn write_key_to_array(key: &EqKey, array: &mut [u8; EqKey::key_len]) {
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
    // array[EqKey::key_len - 1] = key.t_leaf;
    let j = EqKey::key_len - N;
    array[j..j + N].copy_from_slice(&key.cw_leaf.to_le_bytes());
}

fn read_key_from_array(array: &[u8; EqKey::key_len]) -> EqKey {
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

    // let t_leaf = array[EqKey::key_len - 1];
    let j = EqKey::key_len - N;
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
