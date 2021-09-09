use rand::Rng;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sycret::eq::*;
use sycret::stream::{FSSKey, PRG};
use sycret::utils::MMO;
use sycret::{eval, keygen};

pub fn eq_keygen(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let aes_keys: [u128; 4] = rng.gen();
    let mut prg = MMO::from_slice(&aes_keys);
    c.bench_function("Eq keygen", |b| {
        b.iter(|| EqKey::generate_keypair(black_box(&mut prg)))
    });
}

/// op_id = 0 implies Eq key generation
pub fn eq_batch_keygen(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch Eq keygen");
    for &n_values in [10, 100, 1000].iter() {
        let key_length: usize = 621;
        let mut keys_a: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let mut keys_b: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let keys_a_pointer: *mut u8 = keys_a.as_mut_ptr();
        let keys_b_pointer: *mut u8 = keys_b.as_mut_ptr();
        let n_threads: usize = 6;
        let op_id: usize = 0;
        unsafe {
            group.bench_function(format!("n_values: {}", n_values), |b| {
                b.iter(|| keygen(keys_a_pointer, keys_b_pointer, n_values, n_threads, op_id))
            });
        }
    }
    for &n_threads in [1, 4, 6].iter() {
        let key_length: usize = 621;
        let n_values: usize = 100;
        let op_id: usize = 0;
        let mut keys_a: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let mut keys_b: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let keys_a_pointer: *mut u8 = keys_a.as_mut_ptr();
        let keys_b_pointer: *mut u8 = keys_b.as_mut_ptr();
        unsafe {
            group.bench_function(format!("n_threads: {}", n_threads), |b| {
                b.iter(|| keygen(keys_a_pointer, keys_b_pointer, n_values, n_threads, op_id))
            });
        }
    }
}

pub fn eq_eval(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let aes_keys: [u128; 4] = rng.gen();
    let mut prg = MMO::from_slice(&aes_keys);

    let (k_a, k_b) = EqKey::generate_keypair(&mut prg);
    let alpha = k_a.alpha_share.wrapping_add(k_b.alpha_share);

    c.bench_function("Eq eval", |b| {
        b.iter(|| k_a.eval(black_box(&mut prg), 0, alpha))
    });
}

pub fn eq_batch_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch Eq eval");
    for &n_values in [10, 100, 1000].iter() {
        let key_length: usize = 621;
        let n_threads: usize = 6;
        let op_id: usize = 0;
        let mut keys_a: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let mut keys_b: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let keys_a_pointer: *mut u8 = keys_a.as_mut_ptr();
        let keys_b_pointer: *mut u8 = keys_b.as_mut_ptr();
        unsafe {
            keygen(keys_a_pointer, keys_b_pointer, n_values, n_threads, op_id);
        }
        let party_id: usize = 0;
        let xs: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let xs_pointer: *const u8 = xs.as_ptr();
        let keys_pointer: *const u8 = keys_a_pointer as *const u8;
        let mut results: Vec<i64> = Vec::with_capacity(key_length * n_values);
        let results_pointer: *mut i64 = results.as_mut_ptr();
        unsafe {
            group.bench_function(format!("n_values: {}", n_values), |b| {
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
    for &n_threads in [1, 4, 6].iter() {
        let key_length: usize = 621;
        let n_values: usize = 100;
        let op_id: usize = 0;
        let mut keys_a: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let mut keys_b: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let keys_a_pointer: *mut u8 = keys_a.as_mut_ptr();
        let keys_b_pointer: *mut u8 = keys_b.as_mut_ptr();
        unsafe {
            keygen(keys_a_pointer, keys_b_pointer, n_values, n_threads, op_id);
        }
        let party_id: usize = 0;
        let xs: Vec<u8> = Vec::with_capacity(key_length * n_values);
        let xs_pointer: *const u8 = xs.as_ptr();
        let keys_pointer: *const u8 = keys_a_pointer as *const u8;
        let mut results: Vec<i64> = Vec::with_capacity(key_length * n_values);
        let results_pointer: *mut i64 = results.as_mut_ptr();
        unsafe {
            group.bench_function(format!("n_threads: {}", n_threads), |b| {
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
}

criterion_group!(bench_eq, eq_keygen, eq_eval, eq_batch_keygen, eq_batch_eval);
criterion_main!(bench_eq);
