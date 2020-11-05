import numpy as np
from .sycret import lib
from .utils import _as_u8_array, _as_usize, _as_i64_array

# TODO: type annotation and comments

# TODO: add some utilities to interact with the keys,
# (e.g. get alpha share for the tests)
# No need for a class for the key (overhead)
class FSSFactory:
    def __init__(
        self,
        key_len,
        n_threads=0,
        x_type=np.int64,
        key_type=np.uint8,
        result_type=np.int64,
        N=4,
        L=16,
        lib_keygen=lib.keygen,
        lib_eval=lib.eval,
        op_id=1,
    ):
        # NOTE: these defaults work for both equality and comparison,
        # but new primitives can override them if necessary.

        self.N = N
        self.L = L
        self.key_len = key_len
        self.x_type = x_type
        self.key_type = key_type
        self.result_type = result_type
        self.n_threads = n_threads
        self.lib_keygen = lib_keygen
        self.lib_eval = lib_eval
        self.op_id = op_id
        return

    def keygen(self, n_values=1):
        # Allocate memory.
        keys_a = np.zeros((n_values + 1, self.key_len), dtype=self.key_type)
        keys_b = np.zeros((n_values + 1, self.key_len), dtype=self.key_type)

        # Convert types.
        r_keys_a = _as_u8_array(keys_a)
        r_keys_b = _as_u8_array(keys_b)
        r_n_values = _as_usize(n_values)
        r_n_threads = _as_usize(self.n_threads)
        r_op_id = _as_usize(self.op_id)

        # Call Rust on this memory.
        self.lib_keygen(r_keys_a, r_keys_b, r_n_values, r_n_threads, r_op_id)
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
        r_op_id = _as_usize(self.op_id)

        # Call Rust on this memory.
        self.lib_eval(
            r_party_id, r_xs, r_keys, r_results, r_n_values, r_n_threads, r_op_id
        )
        return results


class EqFactory(FSSFactory):
    def __init__(self, n_threads=0):
        super().__init__(key_len=597, n_threads=n_threads, op_id=0)


class LeFactory(FSSFactory):
    def __init__(self, n_threads=0):
        # super().__init__(key_len=1304, n_threads=n_threads)
        super().__init__(key_len=1205, n_threads=n_threads, op_id=1)

