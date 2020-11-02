# Sycret

<!-- Rust implementation for [ARIANN: Low-Interaction Privacy-Preserving Deep Learning via Function Secret Sharing](https://arxiv.org/abs/2006.04593).

## Structure

- `src`: the Rust crate.
- `rustfss`: the Python package calling the Rust crate with [Maturin](https://github.com/PyO3/maturin). 
- `tests`: tests for the Rust crate.
- `test`: tests for the Python wrapper.

## Integration with PySyft

The Python package is called from [PySyft](https://github.com/OpenMined/PySyft), like in [this branch](https://github.com/OpenMined/PySyft/blob/49b1d03de1ba82c4043dc63772ed0ebba7aad6c7/syft/frameworks/torch/mpc/fss.py#L317).


## Build and test

- Create a Python environment from `dev-requirements.txt`
- `maturin develop -b cffi --release` to build the crate, bind it to the Python package and install the package locally.
- `cargo test` to test the Rust functionality.
- `pytest test` to test the Python package. -->