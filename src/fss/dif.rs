//!
//! DIF implementation
//!

use crate::stream::PRG;
use crate::utils::{bit_decomposition_u32, compute_out, share_leaf};
use crate::{L, N};
use rand::Rng;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CorrectionWord {
    pub z_l: u32,
    pub u_l: u8,
    pub s_l: u128,
    pub t_l: u8,
    pub z_r: u32,
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
    pub z: u32,
    pub s: u128,
}

/// DIF Key for alpha in u32 given at Keygen time and beta = 1
#[derive(Debug)]
pub struct DIFKeyAlpha1 {
    pub s: u128,
    pub cw: [CompressedCorrectionWord; N * 8],
    pub cw_leaf: [u32; N * 8 + 1],
}

pub trait DIFKey1: Sized {
    fn eval(&self, prg: &mut impl PRG, party_id: u8, x: u32) -> u32;

    fn generate_keypair(prg: &mut impl PRG, alpha: u32) -> (Self, Self);
}

impl DIFKey1 for DIFKeyAlpha1 {
    fn generate_keypair(prg: &mut impl PRG, alpha: u32) -> (Self, Self) {
        // Thread randomness for parallelization.
        let mut rng = rand::thread_rng();

        // Initialize seeds.
        let s_a: u128 = rng.gen();
        let s_b: u128 = rng.gen();

        let (cw, cw_leaf) = generate_cw_from_seeds(prg, alpha, s_a, s_b);

        // Return a key pair.
        (
            DIFKeyAlpha1 {
                s: s_a,
                cw,
                cw_leaf,
            },
            DIFKeyAlpha1 {
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
        let mut u_i;
        let mut z_i;
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

pub fn h(prg: &mut impl PRG, seed: u128) -> CorrectionWord {
    assert_eq!(L, 128 / 8);

    let out = prg.expand(seed);

    // Get the randomness and chop the control bits.
    let s_l = out[0];
    let s_r = out[1];
    let z_l = out[2] as u32;
    let z_r = (out[2] >> 32) as u32;

    // TODO: A 3x expansion PRG is slightly overkill
    let t_l = (out[2] >> 64) as u8 & 1u8;
    let u_l = (out[2] >> 65) as u8 & 1u8;
    let t_r = (out[2] >> 66) as u8 & 1u8;
    let u_r = (out[2] >> 67) as u8 & 1u8;

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
pub fn generate_cw_from_seeds(
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
    let alpha_bits = bit_decomposition_u32(alpha);

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
    cw_leaf[N * 8] = share_leaf(s_a_i as u32, s_b_i as u32, 1, t_b_i);
    (cw, cw_leaf)
}

///
/// Correction words logic
///

pub fn xor_2_words(u: &CorrectionWord, v: &CorrectionWord) -> CorrectionWord {
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

pub fn decompress_word(w: &CompressedCorrectionWord) -> CorrectionWord {
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
