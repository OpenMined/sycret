use rand::Rng;

extern crate sycret;
use sycret::le::*;
use sycret::stream::{FSSKey, Prg};
use sycret::utils::Mmo;

fn eval_on_alpha_with_offset(offset: i32) -> (u32, u32) {
    let mut rng = rand::thread_rng();
    let aes_keys: [u128; 4] = rng.gen();
    let mut prg = Mmo::from_slice(&aes_keys);

    let (k_a, k_b) = LeKey::generate_keypair(&mut prg);

    // Recover alpha from the shares
    let mut alpha: u32 = k_a.alpha_share.wrapping_add(k_b.alpha_share);

    // Add some offset
    if offset > 0 {
        alpha = alpha + offset as u32;
    } else {
        alpha = alpha - (-offset as u32);
    }

    // Clean ciphers
    let mut prg = Mmo::from_slice(&aes_keys);

    // Evaluate separately on alpha
    let a_output = k_a.eval(&mut prg, 0, alpha);

    let mut prg = Mmo::from_slice(&aes_keys);
    let b_output = k_b.eval(&mut prg, 1, alpha);

    (a_output, b_output)
}

#[test]
fn generate_and_evaluate_alpha() {
    // alpha is randomized, test on different inputs to make sure we are not just lucky.
    for _ in 0..16 {
        let (a_output, b_output) = eval_on_alpha_with_offset(0);
        assert_eq!(a_output.wrapping_add(b_output), 1u32);
    }
}

#[test]
fn generate_and_evaluate_le_alpha() {
    for _ in 0..16 {
        let (a_output, b_output) = eval_on_alpha_with_offset(-1);
        assert_eq!(a_output.wrapping_add(b_output), 1u32);
    }
}

#[test]
fn generate_and_evaluate_strictly_greater_than_alpha() {
    for _ in 0..16 {
        let (a_output, b_output) = eval_on_alpha_with_offset(1);
        assert_eq!(a_output.wrapping_add(b_output), 0u32);
    }
}
