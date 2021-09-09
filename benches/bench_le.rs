use rand::Rng;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sycret::le::*;
use sycret::stream::{FSSKey, PRG};
use sycret::utils::MMO;
use sycret::{eval, keygen};

pub fn le_keygen(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let aes_keys: [u128; 4] = rng.gen();
    let mut prg = MMO::from_slice(&aes_keys);
    c.bench_function("Le keygen", |b| {
        b.iter(|| LeKey::generate_keypair(black_box(&mut prg)))
    });
}

/// op_id = 1 implies Le key generation
pub fn le_batch_keygen(c: &mut Criterion) {
    // 4600 comes from multiplying the Le key length (920)
    // by the number of values / n_values (5)
    let mut keys_a: Vec<u8> = Vec::with_capacity(4600);
    let mut keys_b: Vec<u8> = Vec::with_capacity(4600);
    let keys_a_pointer: *mut u8 = keys_a.as_mut_ptr();
    let keys_b_pointer: *mut u8 = keys_b.as_mut_ptr();
    let n_values: usize = 5;
    let n_threads: usize = 6;
    let op_id: usize = 1;
    unsafe {
        c.bench_function("Batch keygen", |b| {
            b.iter(|| keygen(keys_a_pointer, keys_b_pointer, n_values, n_threads, op_id))
        });
    }
}

pub fn le_eval(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let aes_keys: [u128; 4] = rng.gen();
    let mut prg = MMO::from_slice(&aes_keys);

    let (k_a, k_b) = LeKey::generate_keypair(&mut prg);
    let alpha = k_a.alpha_share.wrapping_add(k_b.alpha_share);

    c.bench_function("Le eval", |b| {
        b.iter(|| k_a.eval(black_box(&mut prg), 0, alpha))
    });
}

pub fn le_batch_eval(c: &mut Criterion) {
    let mut keys_a: Vec<u8> = Vec::with_capacity(4600);
    let mut keys_b: Vec<u8> = Vec::with_capacity(4600);
    let keys_a_pointer: *mut u8 = keys_a.as_mut_ptr();
    let keys_b_pointer: *mut u8 = keys_b.as_mut_ptr();
    let n_values: usize = 5;
    let n_threads: usize = 6;
    let op_id: usize = 1;
    unsafe {
        keygen(keys_a_pointer, keys_b_pointer, n_values, n_threads, op_id);
    }
    let party_id: usize = 0;
    let xs: Vec<u8> = Vec::with_capacity(4600);
    let xs_pointer: *const u8 = xs.as_ptr();
    let keys_pointer: *const u8 = keys_a_pointer as *const u8;
    let mut results: Vec<i64> = Vec::with_capacity(4600);
    let results_pointer: *mut i64 = results.as_mut_ptr();
    unsafe {
        c.bench_function("Le batch eval", |b| {
            b.iter(|| {
                eval(
                    party_id,
                    xs_pointer,
                    keys_pointer,
                    results_pointer,
                    n_values,
                    n_threads,
                    op_id,
                )
            })
        });
    }
}

criterion_group!(bench_le, le_keygen, le_eval, le_batch_keygen, le_batch_eval);
criterion_main!(bench_le);
