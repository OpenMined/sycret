//!
//! Comparison keys tailored for AriaNN
//!

use rand::Rng;
use std::convert::TryInto;
use std::fmt;

use std::slice;

use crate::fss::dif::{
    decompress_word, generate_cw_from_seeds, h, xor_2_words, CompressedCorrectionWord,
};
use crate::stream::{FSSKey, RawKey, PRG};
use crate::utils::{bit_decomposition_u32, compute_out};
use crate::{L, N};

pub struct LeKey {
    pub alpha_share: u32,
    pub s: u128,
    pub cw: [CompressedCorrectionWord; N * 8],
    pub cw_leaf: [u32; N * 8 + 1],
}

impl fmt::Debug for LeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = &self.cw_leaf[..].to_vec();
        f.debug_struct("LeKey")
            .field("alpha_share", &self.alpha_share)
            .field("s", &self.s)
            .field("cw", &self.cw)
            .field("cw_leaf", &v)
            .finish()
    }
}

const CW_LEN: usize = 24;

impl RawKey for LeKey {
    // 4 + 16 + 24 * (4 * 8) + 4 * (4 * 8 + 1)
    const key_len: usize = 920;

    unsafe fn to_raw_line(&self, key_pointer: *mut u8) {
        // Cast the output line to a raw pointer.
        let out_ptr: *mut [u8; Self::key_len] =
            slice::from_raw_parts_mut(key_pointer, Self::key_len).as_mut_ptr()
                as *mut [u8; LeKey::key_len];
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
}

impl FSSKey for LeKey {
    fn generate_keypair(prg: &mut impl PRG) -> (Self, Self) {
        // Thread randomness for parallelization.
        let mut rng = rand::thread_rng();
        // TODO: we can replace this randomness by AES-generated randomness and reduce the key size.
        // Random point on which we will check equality.
        let alpha: u32 = rng.gen();
        // Initialize seeds.
        let s_a: u128 = rng.gen();
        let s_b: u128 = rng.gen();
        let (cw, cw_leaf) = generate_cw_from_seeds(prg, alpha, s_a, s_b);
        // Secret-share alpha and split the keys between Alice and Bob.
        let mask: u32 = rng.gen();
        // Return a key pair.
        (
            LeKey {
                alpha_share: alpha.wrapping_sub(mask),
                s: s_a,
                cw,
                cw_leaf,
            },
            LeKey {
                alpha_share: mask,
                s: s_b,
                cw,
                cw_leaf,
            },
        )
    }

    fn eval(&self, prg: &mut impl PRG, party_id: u8, x: u32) -> u32 {
        assert!((party_id == 0u8) || (party_id == 1u8));
        let mut t_i: u8 = party_id;
        let mut s_i: u128 = self.s;
        let mut u_i = 0u8;
        let mut z_i = 0u32;
        let mut out = 0u32;
        let x_bits: Vec<u8> = bit_decomposition_u32(x);
        for i in 0..(N * 8) {
            let mut w = h(prg, s_i);
            if t_i == 1 {
                w = xor_2_words(&w, &decompress_word(&self.cw[i]))
            }
            if x_bits[i] == 0 {
                z_i = w.z_l;
                u_i = w.u_l;
                s_i = w.s_l;
                t_i = w.t_l;
            } else {
                z_i = w.z_r;
                u_i = w.u_r;
                s_i = w.s_r;
                t_i = w.t_r;
            }

            // Mask and sum in Z/2^32Z
            let out_i = compute_out(z_i, self.cw_leaf[i], u_i, party_id);
            out = out.wrapping_add(out_i);
        }
        let out_n = compute_out(s_i as u32, self.cw_leaf[N * 8], t_i, party_id);
        // The final sum is a share of (x <= alpha) in Z/2^32Z
        out.wrapping_add(out_n)
    }
}

///
/// Serialization functions
///

fn write_key_to_array(key: &LeKey, array: &mut [u8; LeKey::key_len]) {
    array[0..N].copy_from_slice(&key.alpha_share.to_le_bytes());
    array[N..(N + L)].copy_from_slice(&key.s.to_le_bytes());
    for i in 0..(N * 8) {
        // Start index for the control word
        let mut j = N + L + i * CW_LEN;
        let cw = key.cw[i];

        // Copy seeds first (u128)
        array[j..j + L].copy_from_slice(&cw.s.to_le_bytes());
        array[j + L..j + L + N].copy_from_slice(&cw.z.to_le_bytes());

        // Copy control bits at the end (u8)
        j = j + L + N;
        array[j] = cw.t_l;
        array[j + 1] = cw.t_r;
        array[j + 2] = cw.u_l;
        array[j + 3] = cw.u_r;
    }
    for i in 0..(N * 8 + 1) {
        let j = N + L + N * 8 * CW_LEN + i * N;
        array[j..j + N].copy_from_slice(&key.cw_leaf[i].to_le_bytes());
    }
}

fn read_key_from_array(array: &[u8; LeKey::key_len]) -> LeKey {
    let alpha_share = u32::from_le_bytes(array[0..N].try_into().unwrap());
    let s = u128::from_le_bytes(array[N..(N + L)].try_into().unwrap());

    let mut cw = [CompressedCorrectionWord {
        z: 0,
        s: 0,
        t_l: 0,
        t_r: 0,
        u_l: 0,
        u_r: 0,
    }; N * 8];
    let mut cw_leaf = [0u32; N * 8 + 1];

    for i in 0..(N * 8) {
        let mut j = N + L + i * CW_LEN;

        let s = u128::from_le_bytes(array[j..j + L].try_into().unwrap());
        let z = u32::from_le_bytes(array[j + L..j + L + N].try_into().unwrap());

        j = j + L + N;
        let t_l = array[j];
        let t_r = array[j + 1];
        let u_l = array[j + 2];
        let u_r = array[j + 3];

        cw[i] = CompressedCorrectionWord {
            s,
            z,
            t_l,
            u_l,
            t_r,
            u_r,
        }
    }
    for i in 0..(N * 8 + 1) {
        let j = N + L + N * 8 * CW_LEN + i * N;
        cw_leaf[i] = u32::from_le_bytes(array[j..j + N].try_into().unwrap());
    }

    LeKey {
        alpha_share,
        s,
        cw,
        cw_leaf,
    }
}
