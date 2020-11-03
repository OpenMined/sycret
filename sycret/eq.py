from .abstract import AbstractFSSFactory
from .sycret import lib


class EqFactory(AbstractFSSFactory):
    def __init__(self, n_threads=0):
        self.lib_keygen = lib.eq_keygen
        self.lib_eval = lib.eq_eval
        super().__init__(key_len=597, n_threads=n_threads)

