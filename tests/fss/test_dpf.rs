use rand::Rng;

extern crate sycret;
use sycret::fss::dpf::*;
use sycret::stream::{FSSKey, Prg};
use sycret::utils::Mmo;

#[test]
fn generate_and_evaluate_alpha() {
    // alpha is randomized, test on different inputs to make sure we are not just lucky.
    let mut rng = rand::thread_rng();
    for _ in 0..16 {
        let alpha: u32 = rng.gen();
        let aes_keys: [u128; 4] = rng.gen();
        let mut prg = Mmo::from_slice(&aes_keys);
        let (k_a, k_b) = DPFKeyAlpha1::generate_keypair(&mut prg, alpha);

        // Evaluate separately on the same input.
        let t_a_output = k_a.eval(&mut prg, 0, alpha);
        let t_b_output = k_b.eval(&mut prg, 1, alpha);

        // The output bit is additively secret-shared in Z/32Z
        assert_eq!(t_a_output.wrapping_add(t_b_output), 1u32);
    }
}

#[test]
fn generate_and_evaluate_not_alpha() {
    // alpha is randomized, test on different inputs to make sure we are not just lucky.
    let mut rng = rand::thread_rng();
    for _ in 0..16 {
        let alpha: u32 = rng.gen();
        let aes_keys: [u128; 4] = rng.gen();
        let mut prg = Mmo::from_slice(&aes_keys);
        let (k_a, k_b) = DPFKeyAlpha1::generate_keypair(&mut prg, alpha);

        let not_alpha: u32 = alpha.wrapping_add(rng.gen::<u32>());
        if not_alpha == alpha {
            let not_alpha = alpha.wrapping_add(1);
        }
        // Evaluate separately on the same input
        let t_a_output = k_a.eval(&mut prg, 0, not_alpha);
        let t_b_output = k_b.eval(&mut prg, 1, not_alpha);

        // The output bit is additively secret-shared in Z/32Z
        assert_eq!(t_a_output.wrapping_add(t_b_output), 0u32);
    }
}
