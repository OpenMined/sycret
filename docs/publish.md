# Publish Package

## Test

It is possible to test the publishing of a PyPI package with [TestPyPI](https://test.pypi.org/).

In order to do this register an account if you haven't done so already and run the following:

```bash
docker run --rm -v $(pwd):/io konstin2/maturin publish -b cffi --no-sdist -r https://test.pypi.org/legacy/ -u USERNAME -p PASSWORD --manylinux 2014
```

It's also possible to build a wheel for `aarch` as follows:

1. Run docker container with: 
```bash
docker run --rm -it -v $(pwd):/home/rust/src messense/manylinux_2_24-cross:aarch64
```
2. Downloaded Rust tools via Rustup:
```bash
curl https://sh.rustup.rs -sSf | bash -s -- -y
```
3. Configured shell:
```bash
echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
source $HOME/.cargo/env
```
4. Added target:
```bash
rustup target add aarch64-unknown-linux-gnu
```
5. Publish package: 
```bash
maturin publish -b cffi --no-sdist -r https://test.pypi.org/legacy/ -u USERNAME -p PASSWORD --manylinux 2014
```

## Production

In order to do this manually for a production-ready release, one can do the same, without specifying the https://test.pypi.org/legacy/ URL. That is:

```bash
docker run --rm -v $(pwd):/io konstin2/maturin publish -b cffi --no-sdist -u USERNAME -p PASSWORD --manylinux 2014
```

It's also possible to build a wheel for `aarch` as follows:

1. Run docker container with: 
```bash
docker run --rm -it -v $(pwd):/home/rust/src messense/manylinux_2_24-cross:aarch64
```
2. Downloaded Rust tools via Rustup:
```bash
curl https://sh.rustup.rs -sSf | bash -s -- -y
```
3. Configured shell:
```bash
echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
source $HOME/.cargo/env
```
4. Added target:
```bash
rustup target add aarch64-unknown-linux-gnu
```
5. Publish package: 
```bash
maturin publish -b cffi --no-sdist -u USERNAME -p PASSWORD --manylinux 2014
```

