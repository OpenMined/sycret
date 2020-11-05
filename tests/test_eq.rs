use rand::Rng;

extern crate sycret;
pub use sycret::eq::*;
pub use sycret::stream::{FSSKey, PRG};
pub use sycret::utils::MMO;

#[test]
fn generate_and_evaluate_alpha() {
    // alpha is randomized, test on different inputs to make sure we are not just lucky.
    for _ in 0..16 {
        let mut rng = rand::thread_rng();
        let aes_keys: [u128; 4] = rng.gen();
        let mut prg = MMO::from_slice(&aes_keys);
        let (k_a, k_b) = EqKey::generate_keypair(&mut prg);
        let t_leaf = k_a.t_leaf;

        // Recover alpha from the shares.
        let mut alpha = k_a.alpha_share.wrapping_add(k_b.alpha_share);
        println!("alpha: {}", alpha);

        // Evaluate separately on the same input.
        let t_a_output = k_a.eval(&mut prg, 0, alpha);
        let t_b_output = k_b.eval(&mut prg, 1, alpha);
        println!(
            "t_a, t_b, t_leaf: {}, {}, {}",
            t_a_output, t_b_output, t_leaf
        );

        // The output bit is additively secret-shared for public beta.
        assert_eq!(t_a_output + t_b_output, 1i8);
    }
}

#[test]
fn generate_and_evaluate_not_alpha() {
    // alpha is randomized, test on different inputs to make sure we are not just lucky.
    for _ in 0..16 {
        let mut rng = rand::thread_rng();
        let aes_keys: [u128; 4] = rng.gen();
        let mut prg = MMO::from_slice(&aes_keys);
        let (k_a, k_b) = EqKey::generate_keypair(&mut prg);
        let t_leaf = k_a.t_leaf;

        // Recover alpha from the shares.
        let mut alpha = k_a.alpha_share.wrapping_add(k_b.alpha_share);
        // xor_two_inplace(&k_a.alpha_share, &k_b.alpha_share, &mut alpha);
        println!("alpha: {:?}, masked: {:?}", alpha, k_a.alpha_share);

        // Modify alpha: we should leave the special path.
        alpha = alpha + 1;
        println!("alpha flipped: {}", alpha);

        // Evaluate separately on the same input.
        let t_a_output = k_a.eval(&mut prg, 0, alpha);
        let t_b_output = k_b.eval(&mut prg, 1, alpha);
        println!(
            "t_a, t_b, t_leaf: {}, {}, {}",
            t_a_output, t_b_output, t_leaf
        );

        // The output bit is additively secret-shared for public beta.
        assert_eq!(t_a_output + t_b_output, 0i8);
    }
}
