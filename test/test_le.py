import numpy as np
import argparse
import pytest
import sys

import sycret


@pytest.mark.parametrize("n_values", [10, 15, 140, 1024, 32465])
def test_multiline(n_values, n_loops=16):

    le = sycret.LeFactory(n_threads=6)

    for _ in range(n_loops):
        keys_a, keys_b = le.keygen(n_values)

        # Reshape to a C-contiguous array (necessary for from_buffer)

        alpha_a = np.frombuffer(np.ascontiguousarray(keys_a[1:, 0:4]), dtype=np.uint32)
        alpha_b = np.frombuffer(np.ascontiguousarray(keys_b[1:, 0:4]), dtype=np.uint32)
        alpha = alpha_a + alpha_b

        x = alpha.astype(np.int64)

        # We just modify some input values, the rest is on the special path.
        x[1] = x[1] + 5
        x[2] = x[2] - 1
        x[4] = x[4] + 1
        x[8] = x[8] - 635435

        x[9] = x[9] + 1

        r_a, r_b = (
            le.eval(0, x, keys_a),
            le.eval(1, x, keys_b),
        )

        # In PySyft, the AdditiveSharingTensor class will take care of the modulo
        result = (r_a + r_b) % (2 ** (le.N * 8))

        expected_result = np.ones(n_values, dtype=np.int64)
        expected_result[1] = 0
        expected_result[2] = 1
        expected_result[4] = 0
        expected_result[8] = 1
        expected_result[9] = 0

        assert (result == expected_result).all()


if __name__ == "__main__":
    test_multiline(10)

