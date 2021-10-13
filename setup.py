# stdlib
from pathlib import Path

# third party
from setuptools import find_packages
from setuptools import setup

setup_dir = Path(__file__).resolve().parent

setup(
    name="sycret",
    version="0.2.3",
    author="Pierre Tholoniat",
    author_email="pierre@tholoniat.com",
    url="https://github.com/OpenMined/sycret",
    description="Function Secret Sharing library for Python and Rust with hardware acceleration",
    long_description=Path(setup_dir, "README.md").open().read(),
    long_description_content_type="text/markdown",
    license="Apache-2.0",
    python_requires=">=3.6",
    install_requires=["numpy>=1", "cffi>=1", "pycparser>=2"],
    packages=find_packages(),
)
