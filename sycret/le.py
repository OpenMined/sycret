import numpy as np
from abstract import AbstractFSSFactory

from .sycret import lib

# TODO: type annotation and comments

class LeFactory(AbstractFSSFactory):

    def __init__(n_threads=0):
        self.key_len = 1304
        super.__init__(keylen, n_threads)

    def keygen(n_values=1):
        # Allocate memory.
        t = time.time()
        keys_a = np.zeros((n_values + 1, self.key_len), dtype=self.key_type)
        keys_b = np.zeros((n_values + 1, self.key_len), dtype=self.key_type)

        # Convert types.
        r_keys_a = utils._as_u8_array(keys_a)
        r_keys_b = utils_as_u8_array(keys_b)
        r_n_values = utils._as_usize(n_values)
        r_n_threads = utils._as_usize(self.n_threads)

        # Call Rust on this memory.
        lib.le_keygen(r_keys_a, r_keys_b, r_n_values, r_n_threads)
        return keys_a, keys_b

    def eval(party_id, xs, keys, n_threads=0):
        results = np.zeros(n_values, dtype=self.result_type)

        # Warning: if the type is too big, the reshaping operation might be costly.
        np8_xs = np.ascontiguousarray(xs.view(dtype=np.uint8).reshape(n_values, -1)[:, 0:self.N])

        r_party_id = _as_usize(party_id)
        r_xs = _as_u8_array(np8_xs)
        r_keys = _as_u8_array(keys)
        r_results = _as_i64_array(results)
        r_n_values = _as_usize(n_values)
        r_n_threads = _as_usize(n_threads)

        # Call Rust on this memory.
        lib.le_eval(r_party_id, r_xs, r_keys, r_results, r_n_values, r_n_threads)
        return results
        
