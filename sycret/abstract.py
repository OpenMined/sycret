import numpy as np
from abc import ABC, abstractmethod


class AbstractFSSFactory(ABC):
    def __init__(
        self,
        key_len,
        n_threads=0,
        x_type=np.int64,
        key_type=np.uint8,
        result_type=np.int64,
        N=4,
        L=16,
    ):
        self.N = N
        self.L = L
        self.key_len = key_len
        self.x_type = x_type
        self.key_type = key_type
        self.result_type = result_type
        self.n_threads = n_threads
        return

    @abstractmethod
    def keygen(self, n_values):
        pass

    @abstractmethod
    def eval(self, party_id, xs, keys):
        pass
