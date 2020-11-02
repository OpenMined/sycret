pub fn generate_key_stream(
    &aes_keys &[u128],
    stream_id: usize,
    stream_length: usize,
    key_a_pointer: usize,
    key_b_pointer: usize,
    op_id: usize,
) {
    // Generate keys in sequence
    let key_a_p = key_a_pointer as *mut u8;
    let key_b_p = key_b_pointer as *mut u8;

    // TODO: def. Impl PRG.
    let mut prg = MMO.from_slice(aes_keys);

    for line_counter in 0..stream_length {

        if op_id == 1 {
            let (key_a, key_b) = LeKey.generate_keypair(&mut prg);
        }
        else {
            let (key_a, key_b) = EqKey.generate_keypair(&mut prg);
        }

        unsafe {
            // TODO: keylen?
            &key_a.write_to_raw_line(key_a_p.add(&key_a.key_len * line_counter));
            &key_b.write_to_raw_line(key_b_p.add(&key_b.key_len * line_counter));
        }
    }
}


pub fn eval_key_stream(
    party_id: u8,
    aes_keys: &[u128],
    stream_id: usize,
    stream_length: usize,
    x_pointer: usize,
    key_pointer: usize,
    result_pointer: usize,
    op_id: usize,
) {
    assert!((party_id == 0u8) || (party_id == 1u8));

    let mut prg = MMO.from_slice(aes_keys);

    // Read, eval, write line by line

    let x_pointer_p = x_pointer as *const u8;
    let key_pointer_p = key_pointer as *const u8;
    let result_ptr_p = result_pointer as *mut i64;


    for line_counter in 0..stream_length {
        // Read key and value to evaluate
        unsafe {
            let x_ptr: *const [u8; N] =
                slice::from_raw_parts(x_pointer_p.add(N * line_counter), N).as_ptr() as *const [u8; N];
            let x: u32 = u32::from_le_bytes(*x_ptr);
            
            // TODO: :: or . call? Maybe method?
            let key = match op_id {
                1 => LeKey.from_raw_line(key_pointer_p.add(LeKey.key_len * line_counter))
                _ => EqKey.from_raw_line(key_pointer_p.add(EqKey.key_len * line_counter))
            }
            
            // let key_ptr: *const [u8; LE_KEY_LEN] =
            //     slice::from_raw_parts(key_pointer_p.add(LE_KEY_LEN * line_counter), LE_KEY_LEN).as_ptr()
            //         as *const [u8; LE_KEY_LEN];
            // let key = read_key_from_array(&*key_ptr);

            // Run the evaluation
            // TODO: Z/2Z
            let result: u8 = key.eval(&mut prg, party_id, x);

            // TODO: wrap around if too large

            // Write the result in a raw line for Numpy
            *(result_ptr_p.add(line_counter)) = result as i64;
        }
    }
}