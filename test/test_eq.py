import sycret
import numpy as np
import argparse
import pytest


def test_simple_raw_eq():

    eq = sycret.EqFactory(n_threads=6)

    for _ in range(16):
        keys_a, keys_b = eq.keygen(1)
        print(keys_a)
        alpha_a = np.frombuffer(keys_a[1][0 : eq.N], dtype=np.uint32)
        alpha_b = np.frombuffer(keys_b[1][0 : eq.N], dtype=np.uint32)
        alpha = alpha_a + alpha_b
        print(
            f"shares from buffer: {alpha_a}, {alpha_b}, alpha: {alpha}, back to buf: {alpha.view(dtype=np.uint8)}"
        )

        x = alpha.astype(np.int64)

        print(f"x: {x}, {x.dtype}")
        r_a, r_b = (
            eq.eval(0, x, keys_a),
            eq.eval(1, x, keys_b),
        )
        print(r_a, r_b, r_a + r_b)
        assert r_a + r_b == 1

        x = alpha.astype(np.int64) + 31
        r_a, r_b = (
            eq.eval(0, x, keys_a),
            eq.eval(1, x, keys_b),
        )
        print(r_a, r_b, r_a + r_b)
        assert r_a + r_b == 0


@pytest.mark.parametrize("n_values", [5, 10])
def test_multiline(n_values, n_loops=16):

    eq = sycret.EqFactory(n_threads=6)

    for _ in range(n_loops):
        keys_a, keys_b = eq.keygen(n_values)

        # Reshape to a C-contiguous array (necessary for from_buffer)

        alpha_a = np.frombuffer(
            np.ascontiguousarray(keys_a[1:, 0 : eq.N]), dtype=np.uint32
        )
        alpha_b = np.frombuffer(
            np.ascontiguousarray(keys_b[1:, 0 : eq.N]), dtype=np.uint32
        )
        alpha = alpha_a + alpha_b

        # print(
        #     f"shares from buffer: {alpha_a}, {alpha_b}, alpha: {alpha}, back to buf: {alpha.view(dtype=np.uint8)}"
        # )

        x = alpha.astype(np.int64)
        # We just modify some input values, the rest is on the special path.

        x[1] = x[1] + 5
        x[2] = x[2] - 1
        x[4] = x[4] + 1

        r_a, r_b = (
            eq.eval(0, x, keys_a),
            eq.eval(1, x, keys_b),
        )

        result = r_a + r_b

        expected_result = np.ones(n_values, dtype=np.uint64)
        expected_result[1] = 0
        expected_result[2] = 0
        expected_result[4] = 0

        # print(expected_result.dtype)
        # print(result.dtype)
        assert (result == expected_result).all()


if __name__ == "__main__":
    # test_simple_raw_eq()
    # test_u32_to_u8_list()
    # test_multiline_raw_eq()
    parser = argparse.ArgumentParser()
    parser.add_argument("--N", type=int, default=500_000, required=False)
    args = parser.parse_args()
    test_multiline(args.N, n_loops=100)
