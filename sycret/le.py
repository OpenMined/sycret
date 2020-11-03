import numpy as np

from .abstract import AbstractFSSFactory
from .sycret import lib
from .utils import _as_u8_array, _as_usize, _as_i64_array

# TODO: type annotation and comments


class LeFactory(AbstractFSSFactory):
    def __init__(self, n_threads=0):
        super().__init__(key_len=1304, n_threads=n_threads)

    def keygen(self, n_values=1):
        # Allocate memory.
        keys_a = np.zeros((n_values + 1, self.key_len), dtype=self.key_type)
        keys_b = np.zeros((n_values + 1, self.key_len), dtype=self.key_type)

        # Convert types.
        r_keys_a = _as_u8_array(keys_a)
        r_keys_b = _as_u8_array(keys_b)
        r_n_values = _as_usize(n_values)
        r_n_threads = _as_usize(self.n_threads)
        r_op_id = _as_usize(1)

        # Call Rust on this memory.
        lib.keygen(r_keys_a, r_keys_b, r_n_values, r_n_threads, r_op_id)
        return keys_a, keys_b

    def eval(self, party_id, xs, keys, n_threads=0):
        n_values = xs.shape[0]
        results = np.zeros(n_values, dtype=self.result_type)

        # Warning: if the type is too big, the reshaping operation might be costly.
        np8_xs = np.ascontiguousarray(
            xs.view(dtype=np.uint8).reshape(n_values, -1)[:, 0 : self.N]
        )

        r_party_id = _as_usize(party_id)
        r_xs = _as_u8_array(np8_xs)
        r_keys = _as_u8_array(keys)
        r_results = _as_i64_array(results)
        r_n_values = _as_usize(n_values)
        r_n_threads = _as_usize(n_threads)
        r_op_id = _as_usize(1)

        # Call Rust on this memory.
        lib.eval(r_party_id, r_xs, r_keys, r_results, r_n_values, r_n_threads, r_op_id)
        return results
