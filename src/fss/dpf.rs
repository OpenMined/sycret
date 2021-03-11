//!
//! Generic DPF implementation
//!  

use super::super::eq::{g, generate_cw_from_seeds};
use super::super::stream::PRG;
use super::super::utils::{bit_decomposition_u32, compute_out};
use super::super::{N};
use rand::{Rng};

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
