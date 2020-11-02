use std::convert::TryInto;
use std::fmt;
use std::num::Wrapping;
use std::slice;

use super::stream::FSSKey;
use super::{L, N};

pub struct LeKey {
    pub alpha_share: u32,
    pub s: u128,
    pub cw: [CompressedCorrectionWord; N * 8],
    pub cw_leaf: [u32; N * 8 + 1],
}

impl fmt::Debug for LeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = &self.cw_leaf[..].to_vec();
        f.debug_struct("LeKey")
            .field("alpha_share", &self.alpha_share)
            .field("s", &self.s)
            .field("cw", &self.cw)
            .field("cw_leaf", &v)
            .finish()
    }
}

impl FSSKey for LeKey {
    // 4 + 16 + 36 * (4 * 8) + 1 * (4 * 8 + 1)
    const key_len: usize = 1205;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CorrectionWord {
    pub z_l: u128,
    pub u_l: u8,
    pub s_l: u128,
    pub t_l: u8,
    pub z_r: u128,
    pub u_r: u8,
    pub s_r: u128,
    pub t_r: u8,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CompressedCorrectionWord {
    pub u_l: u8,
    pub t_l: u8,
    pub u_r: u8,
    pub t_r: u8,
    pub z: u128,
    pub s: u128,
}
