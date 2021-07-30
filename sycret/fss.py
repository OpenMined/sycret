# third party
import numpy as np

from .sycret import lib
from .utils import _as_i64_array
from .utils import _as_u8_array
from .utils import _as_usize

# TODO: add some utilities to interact with the keys,
# (e.g. get alpha share for the tests)
# No need for a class for the key (overhead)


class FSSFactory:
    """A generic class wrapping some constants and methods for FSS key
    generation and evaluation."""

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
        """Initializes some constants for FSS.

        Args:
            key_len ([type]): [description]
            n_threads (int, optional): [description]. Defaults to 0.
            x_type ([type], optional): [description]. Defaults to np.int64.
            key_type ([type], optional): [description]. Defaults to np.uint8.
            result_type ([type], optional): [description]. Defaults to np.int64.
            N (int, optional): [description]. Defaults to 4.
            L (int, optional): [description]. Defaults to 16.
            lib_keygen ([type], optional): [description]. Defaults to lib.keygen.
            lib_eval ([type], optional): [description]. Defaults to lib.eval.
            op_id (int, optional): [description]. Defaults to 1.
        """
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
        """[summary]

        Args:
            n_values (int, optional): [description]. Defaults to 1.

        Returns:
            [type]: [description]
        """
        # Allocate memory.
        keys_a = np.zeros((n_values, self.key_len), dtype=self.key_type)
        keys_b = np.zeros((n_values, self.key_len), dtype=self.key_type)

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
        """[summary]

        Args:
            party_id ([type]): [description]
            xs ([type]): [description]
            keys ([type]): [description]
            n_threads (int, optional): [description]. Defaults to 0.

        Returns:
            [type]: [description]
        """
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

    def alpha(self, keys_a: np.array, keys_b: np.array) -> np.array:
        """Calculate the alpha value of the given key.

        Arguments:
            keys_a: Values of the first piece of the key
            keys_b: Values of the second piece of the key

        Returns:
            Alpha values in an array
        """
        key_values = (
            lambda self, key: key[0][0 : self.N]
            if key.shape[0] == 1
            else np.ascontiguousarray(key[:, 0 : self.N])
        )

        alpha_a = np.frombuffer(key_values(self, keys_a), dtype=np.uint32)
        alpha_b = np.frombuffer(key_values(self, keys_b), dtype=np.uint32)
        alpha = alpha_a + alpha_b

        return alpha


class EqFactory(FSSFactory):
    """Distributed Point Function."""

    def __init__(self, n_threads=0):
        super().__init__(key_len=621, n_threads=n_threads, op_id=0)


class LeFactory(FSSFactory):
    """Distributed Interval Functino."""

    def __init__(self, n_threads=0):
        super().__init__(key_len=920, n_threads=n_threads, op_id=1)
