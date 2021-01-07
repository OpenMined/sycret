use rand::Rng;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sycret::eq::*;
use sycret::stream::{FSSKey, PRG};
use sycret::utils::MMO;

// fn gen() {

// }

pub fn eq_keygen(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let aes_keys: [u128; 4] = rng.gen();
    let mut prg = MMO::from_slice(&aes_keys);
    c.bench_function("Eq keygen", |b| {
        b.iter(|| EqKey::generate_keypair(black_box(&mut prg)))
    });
}

criterion_group!(benches, eq_keygen);
criterion_main!(benches);
