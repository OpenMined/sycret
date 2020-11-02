use rand::{thread_rng, Rng};
use std::convert::TryInto;
use std::fmt;
use std::num::Wrapping;
use std::slice;

use super::stream::{FSSKey, PRG};
use super::utils::{bit_decomposition_u32, MMO};
use super::{L, N};

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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CorrectionWord {
    pub z_l: u128,
    pub u_l: u8,
    pub s_l: u128,
    pub t_l: u8,
    pub z_r: u128,
    pub u_r: u8,
    pub s_r: u128,
    pub t_r: u8,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CompressedCorrectionWord {
    pub u_l: u8,
    pub t_l: u8,
    pub u_r: u8,
    pub t_r: u8,
    pub z: u128,
    pub s: u128,
}

const CW_LEN: usize = 36;

impl FSSKey for LeKey {
    // 4 + 16 + 36 * (4 * 8) + 1 * (4 * 8 + 1)
    const key_len: usize = 1205;

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
        let mut z_i = 0u128;
        let mut out = 0u8;
        let mut out_v: Vec<Wrapping<u32>> = vec![];
        let x_bits: Vec<u8> = bit_decomposition_u32(x);
        for i in 0..(N * 8) {
            let mut w = h(prg, s_i);
            // let mut w = h_127(s_i);
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
            let mut out_i = ((u_i as u32) * self.cw_leaf[i]).wrapping_add(z_i as u32);
            if party_id == 1 {
                out_i = 0u32.wrapping_sub(out_i);
            }
            out_v.push(Wrapping(out_i));
        }
        let mut out_n = ((t_i as u32) * self.cw_leaf[N * 8]).wrapping_add(s_i as u32);
        if party_id == 1 {
            out_n = 0u32.wrapping_sub(out_n);
        }
        out_v.push(Wrapping(out_n));

        // Sum modulo n ** 32 and unwrap
        let result = out_v.iter().sum::<Wrapping<u32>>().0;
        result
    }
}

pub fn h(prg: &mut impl PRG, seed: u128) -> CorrectionWord {
    // This is an awkward prototype

    // TODO: optimize format, byte operations, assembly call to AESNI (e.g. https://github.com/gendx/haraka-rs/blob/master/src/intrinsics.rs)
    assert_eq!(L, 128 / 8);

    let out = prg.expand(seed);

    // Get the randomness and chop the control bits.
    let mut s_l = out[0] >> 1 << 1;
    let mut z_l = out[1] >> 1 << 1;
    let mut s_r = out[2] >> 1 << 1;
    let mut z_r = out[3] >> 1 << 1;

    let t_l = (out[0] & 1u128) as u8;
    let u_l = (out[1] & 1u128) as u8;
    let t_r = (out[2] & 1u128) as u8;
    let u_r = (out[3] & 1u128) as u8;

    CorrectionWord {
        s_l,
        t_l,
        z_l,
        u_l,
        s_r,
        t_r,
        z_r,
        u_r,
    }
}

/// Internal deterministic function.
fn generate_cw_from_seeds(
    prg: &mut impl PRG,
    alpha: u32,
    s_a: u128,
    s_b: u128,
) -> ([CompressedCorrectionWord; N * 8], [u32; N * 8 + 1]) {
    // Initialize the output control words. Arrays instead of vectors for CFFI.
    let mut cw = [CompressedCorrectionWord {
        s: 0,
        z: 0,
        t_l: 0,
        t_r: 0,
        u_l: 0,
        u_r: 0,
    }; N * 8];
    let mut cw_leaf = [0u32; N * 8 + 1];

    // Initialize control bits.
    let mut t_a_i = 0u8;
    let mut t_b_i = 1u8;

    // Seeds at level i.
    let mut s_a_i: u128 = s_a;
    let mut s_b_i: u128 = s_b;

    let mut u_b_i;
    let mut z_a_i;
    let mut z_b_i;

    // Iterate over the bits of alpha
    let alpha_bits = super::utils::bit_decomposition_u32(alpha);

    for i in 0..(N * 8) {
        let w_a = h(prg, s_a_i);
        let w_b = h(prg, s_b_i);

        let mut cw_i = match alpha_bits[i] {
            1 => CorrectionWord {
                z_l: w_a.z_l ^ w_b.z_l,
                u_l: 1,
                s_l: 0,
                t_l: 0,
                z_r: 0,
                u_r: 0,
                s_r: w_a.s_l ^ w_b.s_l,
                t_r: 1,
            },
            _ => CorrectionWord {
                z_l: 0,
                u_l: 0,
                s_l: w_a.s_r ^ w_b.s_r,
                t_l: 1,
                z_r: w_a.z_r ^ w_b.z_r,
                u_r: 1,
                s_r: 0,
                t_r: 0,
            },
        };

        // TODO: improve efficiency (debug code)
        cw_i = xor_3_words(&cw_i, &w_a, &w_b);
        cw[i] = compress_word(&cw_i, alpha_bits[i]);
        cw_i = decompress_word(&cw[i]);

        // TODO: unroll and optimize.
        let w_a_next = match t_a_i {
            0 => w_a,
            _ => xor_2_words(&w_a, &cw_i),
        };
        let w_b_next = match t_b_i {
            0 => w_b,
            _ => xor_2_words(&w_b, &cw_i),
        };

        if alpha_bits[i] == 0 {
            s_a_i = w_a_next.s_l;
            t_a_i = w_a_next.t_l;
            z_a_i = w_a_next.z_r;
        } else {
            s_a_i = w_a_next.s_r;
            t_a_i = w_a_next.t_r;
            z_a_i = w_a_next.z_l;
        }

        if alpha_bits[i] == 0 {
            s_b_i = w_b_next.s_l;
            t_b_i = w_b_next.t_l;
            z_b_i = w_b_next.z_r;
            u_b_i = w_b_next.u_r;
        } else {
            s_b_i = w_b_next.s_r;
            t_b_i = w_b_next.t_r;
            z_b_i = w_b_next.z_l;
            u_b_i = w_b_next.u_l;
        }

        cw_leaf[i] = share_leaf(z_a_i, z_b_i, alpha_bits[i], u_b_i);
    }
    // NOTE: The flip bit (tau or t) is not used modulo 2.

    // Public beta = 1
    // cw_leaf[N * 8] = t_b_i;
    cw_leaf[N * 8] = share_leaf(s_a_i, s_b_i, 1, t_b_i);
    (cw, cw_leaf)
}

fn xor_2_words(u: &CorrectionWord, v: &CorrectionWord) -> CorrectionWord {
    CorrectionWord {
        s_l: u.s_l ^ v.s_l,
        t_l: u.t_l ^ v.t_l,
        s_r: u.s_r ^ v.s_r,
        t_r: u.t_r ^ v.t_r,
        z_l: u.z_l ^ v.z_l,
        u_l: u.u_l ^ v.u_l,
        z_r: u.z_r ^ v.z_r,
        u_r: u.u_r ^ v.u_r,
    }
}

fn xor_3_words(u: &CorrectionWord, v: &CorrectionWord, w: &CorrectionWord) -> CorrectionWord {
    CorrectionWord {
        s_l: u.s_l ^ v.s_l ^ w.s_l,
        t_l: u.t_l ^ v.t_l ^ w.t_l,
        s_r: u.s_r ^ v.s_r ^ w.s_r,
        t_r: u.t_r ^ v.t_r ^ w.t_r,
        z_l: u.z_l ^ v.z_l ^ w.z_l,
        u_l: u.u_l ^ v.u_l ^ w.u_l,
        z_r: u.z_r ^ v.z_r ^ w.z_r,
        u_r: u.u_r ^ v.u_r ^ w.u_r,
    }
}

fn compress_word(w: &CorrectionWord, alpha_i: u8) -> CompressedCorrectionWord {
    let (z, s) = match alpha_i {
        1 => (w.z_r, w.s_l),
        _ => (w.z_l, w.s_r),
    };
    CompressedCorrectionWord {
        z,
        s,
        t_l: w.t_l,
        u_l: w.u_l,
        t_r: w.t_r,
        u_r: w.u_r,
    }
}

fn decompress_word(w: &CompressedCorrectionWord) -> CorrectionWord {
    CorrectionWord {
        z_l: w.z,
        s_l: w.s,
        z_r: w.z,
        s_r: w.s,
        t_l: w.t_l,
        u_l: w.u_l,
        t_r: w.t_r,
        u_r: w.u_r,
    }
}

fn share_leaf(seed_a: u128, seed_b: u128, share_bit: u8, flip_bit: u8) -> u32 {
    let mut leaf = (seed_b as u32)
        .wrapping_sub(seed_a as u32)
        .wrapping_add(share_bit as u32);
    if flip_bit == 1 {
        leaf = 0u32.wrapping_sub(leaf);
    }
    leaf
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
        array[j + L..j + 2 * L].copy_from_slice(&cw.z.to_le_bytes());

        // Copy control bits at the end (u8)
        j = j + 2 * L;
        array[j] = cw.t_l;
        array[j + 1] = cw.t_r;
        array[j + 2] = cw.u_l;
        array[j + 3] = cw.u_r;
    }
    for i in 0..(N * 8 + 1) {
        // array[N + L + N * 8 * CW_LEN + i] = key.cw_leaf[i];
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
        let z = u128::from_le_bytes(array[j + L..j + 2 * L].try_into().unwrap());

        j = j + 2 * L;
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
        // cw_leaf[i] = array[N + L + N * 8 * CW_LEN + i];
    }

    LeKey {
        alpha_share,
        s,
        cw,
        cw_leaf,
    }
}
