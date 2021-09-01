use criterion::{criterion_group, criterion_main, Criterion};
use sycret::keygen;

pub fn batch_keygen(c: &mut Criterion) {
    let mut keys_a: Vec<u8> = Vec::with_capacity(16);
    let mut keys_b: Vec<u8> = Vec::with_capacity(16);
    let keys_a_pointer: *mut u8 = keys_a.as_mut_ptr();
    let keys_b_pointer: *mut u8 = keys_b.as_mut_ptr();
    let n_values: usize = 16;
    let n_threads: usize = 2;
    let op_id: usize = 0;
    unsafe {
        c.bench_function("Batch keygen", |b| {
            b.iter(|| keygen(keys_a_pointer, keys_b_pointer, n_values, n_threads, op_id))
        });
    }
}

criterion_group!(batch, batch_keygen);
criterion_main!(batch);
