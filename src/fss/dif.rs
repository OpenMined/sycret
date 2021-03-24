//!
//! Generic DIF implementation
//!

use super::super::le::{
    decompress_word, generate_cw_from_seeds, h, xor_2_words, CompressedCorrectionWord,
};
use super::super::stream::PRG;
use super::super::utils::{bit_decomposition_u32, compute_out};
use super::super::N;
use rand::Rng;

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
