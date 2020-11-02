from abstract import AbstractFSSFactory

class EqFactory(AbstractFSSFactory):

    def __init__(n_threads=0):
        self.lib_keygen = lib.eq_keygen
        self.lib_eval = lib.eq_eval
        self.keylen = 597
        super.__init__(keylen, n_threads)

