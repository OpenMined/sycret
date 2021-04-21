//!
//! DPF implementations
//!

use crate::stream::PRG;
use crate::utils::{bit_decomposition_u32, compute_out, share_leaf};
use crate::{L, N};
use rand::Rng;

///
/// Deterministic function
///
pub fn generate_cw_from_seeds(
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
        let (s_a_l, t_a_l, s_a_r, t_a_r) = g(prg, s_a_i);
        let (s_b_l, t_b_l, s_b_r, t_b_r) = g(prg, s_b_i);

        // Keep left if a_i = 0, keep right if a_i = 1.
        let (s_a_keep, s_a_lose, t_a_keep) = match alpha_bits[i] {
            0u8 => (s_a_l, s_a_r, t_a_l),
            _ => (s_a_r, s_a_l, t_a_r),
        };
        let (s_b_keep, s_b_lose, t_b_keep) = match alpha_bits[i] {
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
        cw[i] = s_a_lose ^ s_b_lose;
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
/// Wrapper around a PRG with expansion factor 2
///
pub fn g(prg: &mut impl PRG, seed: u128) -> (u128, u8, u128, u8) {
    assert_eq!(L, 128 / 8);

    let out = prg.expand(seed);

    let t_l = out[0] as u8 & 1u8;
    let t_r = out[1] as u8 & 1u8;
    let s_l = out[0] >> 1 << 1;
    let s_r = out[1] >> 1 << 1;

    (s_l, t_l, s_r, t_r)
}

/// DPF Key for alpha in u32 given at Keygen time and beta = 1
#[derive(Debug)]
pub struct DPFKeyAlpha1 {
    pub s: u128,
    pub cw: [u128; N * 8],
    pub t_l: [u8; N * 8],
    pub t_r: [u8; N * 8],
    pub cw_leaf: u32,
}

pub trait DPFKey1: Sized {
    fn eval(&self, prg: &mut impl PRG, party_id: u8, x: u32) -> u32;

    fn generate_keypair(prg: &mut impl PRG, alpha: u32) -> (Self, Self);
}

impl DPFKey1 for DPFKeyAlpha1 {
    fn generate_keypair(prg: &mut impl PRG, alpha: u32) -> (Self, Self) {
        // Thread randomness for parallelization.
        let mut rng = rand::thread_rng();

        // Initialize seeds.
        let s_a: u128 = rng.gen();
        let s_b: u128 = rng.gen();

        // Memory allocation for the correction words
        let mut cw = [0u128; N * 8];
        let mut t_l = [0u8; N * 8];
        let mut t_r = [0u8; N * 8];

        let cw_leaf = generate_cw_from_seeds(prg, alpha, s_a, s_b, &mut cw, &mut t_l, &mut t_r);

        // Return a key pair.
        (
            DPFKeyAlpha1 {
                s: s_a,
                cw,
                t_l,
                t_r,
                cw_leaf,
            },
            DPFKeyAlpha1 {
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
