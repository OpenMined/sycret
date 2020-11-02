import numpy as np
from .sycret import ffi

def _as_u64_array(np_uint64_array):
    return ffi.cast("unsigned long *", np_uint64_array.ctypes.data)


def _as_i64_array(np_int64_array):
    return ffi.cast("int64_t *", np_int64_array.ctypes.data)


def _as_i8_array(np_int8_array):
    return ffi.cast("int8_t *", np_int8_array.ctypes.data)


def _as_usize(np_int):
    return ffi.cast("unsigned long", np_int)


def _as_u8_array(np_u8_array):
    return ffi.cast("uint8_t *", np_u8_array.ctypes.data)
