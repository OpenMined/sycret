use rand::Rng;

extern crate sycret;
use sycret::eq::*;
use sycret::stream::{FSSKey, Prg};
use sycret::utils::Mmo;

#[test]
fn generate_and_evaluate_alpha() {
    // alpha is randomized, test on different inputs to make sure we are not just lucky.
    for _ in 0..16 {
        let mut rng = rand::thread_rng();
        let aes_keys: [u128; 4] = rng.gen();
        let mut prg = Mmo::from_slice(&aes_keys);
        let (k_a, k_b) = EqKey::generate_keypair(&mut prg);

        // Recover alpha from the shares.
        let alpha = k_a.alpha_share.wrapping_add(k_b.alpha_share);
        println!("alpha: {}", alpha);

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
    for _ in 0..16 {
        let mut rng = rand::thread_rng();
        let aes_keys: [u128; 4] = rng.gen();
        let mut prg = Mmo::from_slice(&aes_keys);
        let (k_a, k_b) = EqKey::generate_keypair(&mut prg);

        // Recover alpha from the shares.
        let mut alpha = k_a.alpha_share.wrapping_add(k_b.alpha_share);
        // xor_two_inplace(&k_a.alpha_share, &k_b.alpha_share, &mut alpha);
        println!("alpha: {:?}, masked: {:?}", alpha, k_a.alpha_share);

        // Modify alpha: we should leave the special path.
        alpha += 1;
        println!("alpha flipped: {}", alpha);

        // Evaluate separately on the same input.
        let t_a_output = k_a.eval(&mut prg, 0, alpha);
        let t_b_output = k_b.eval(&mut prg, 1, alpha);

        // The output bit is additively secret-shared in Z/32Z
        assert_eq!(t_a_output.wrapping_add(t_b_output), 0u32);
    }
}
