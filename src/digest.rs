use hex::{FromHex, ToHex};

use crate::ColoredTLSH;
use crate::hash::TLSH;

const BODY_SIZE: usize = 32;
const HASH_SIZE: usize = 3 + BODY_SIZE;
const HEX_HASH_SIZE: usize = HASH_SIZE * 2;
const VERSIONED_HEX_HASH_SIZE: usize = HEX_HASH_SIZE + 2;
const COLORED_HASH_SIZE: usize = 1 + HASH_SIZE;
const HEX_COLORED_HASH_SIZE: usize = COLORED_HASH_SIZE * 2;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq)]
pub enum TLSHDigestError {
    InvalidLength,
    InvalidHex,
    InvalidVersion,
}

#[inline(always)]
fn swap_byte(x: u8) -> u8 {
    x >> 4 | x << 4
}

impl TLSH {
    /// Exports the hash object as its 35-byte raw representation
    pub fn to_raw(&self) -> [u8; HASH_SIZE] {
        let mut raw = [0u8; HASH_SIZE];
        raw[0] = swap_byte(self.checksum);
        raw[1] = swap_byte(self.lvalue);
        raw[2] = swap_byte(self.q_ratios);
        for (r, c) in raw[3..].iter_mut().rev().zip(self.codes.iter()) {
            *r = *c;
        }
        raw
    }

    /// Imports a hash object from its 35-byte raw representation.
    ///
    /// Panics if the input is not a valid hash
    pub fn from_raw(raw: &[u8]) -> Self {
        Self::try_from_raw(raw).unwrap()
    }

    /// Tries to import a hash object from its raw 35-byte representation
    pub fn try_from_raw(raw: &[u8]) -> Result<Self, TLSHDigestError> {
        if raw.len() != HASH_SIZE {
            return Err(TLSHDigestError::InvalidLength)
        }
        let mut codes = [0u8; BODY_SIZE];
        for (c, r) in codes.iter_mut().zip(raw[3..].iter().rev()) {
            *c = *r;
        }
        Ok(Self {
            checksum: swap_byte(raw[0]),
            lvalue: swap_byte(raw[1]),
            q_ratios: swap_byte(raw[2]),
            codes,
        })
    }

    /// Exports the hash object as a hex digest string
    pub fn to_digest(&self) -> String {
        self.to_raw().encode_hex_upper()
    }

    /// Export the hash object as a versioned (T1...) hex digest
    pub fn to_digest_versioned(&self, version: i32) -> String {
        format!("T{version}{}", self.to_digest())
    }

    /// Tries to import a hash object from a digest string
    ///
    pub fn try_from_digest(digest: &str) -> Result<Self, TLSHDigestError> {
        let tlsh_digest = match digest.len() {
            HEX_HASH_SIZE => digest,
            VERSIONED_HEX_HASH_SIZE if digest.starts_with("T1") => &digest[2..],
            VERSIONED_HEX_HASH_SIZE if digest.starts_with("T") => return Err(TLSHDigestError::InvalidVersion),
            _ => return Err(TLSHDigestError::InvalidLength),
        };
        let hash_bytes = hex::decode(tlsh_digest).map_err(|_| TLSHDigestError::InvalidHex)?;
        Self::try_from_raw(&hash_bytes)
    }

    /// Import a hash object from a digest string
    ///
    /// Panics if the digest is invalid
    pub fn from_digest(digest: &str) -> Self {
        Self::try_from_digest(digest).unwrap()
    }
}

impl ColoredTLSH {
    /// Exports the colored hash object to its 36-byte representation
    pub fn to_raw(&self) -> [u8; COLORED_HASH_SIZE] {
        let mut raw = [0u8; COLORED_HASH_SIZE];
        raw[0] = self.color;
        let tlsh = self.tlsh.to_raw();
        raw[1..].copy_from_slice(&tlsh);
        raw
    }

    /// Tries to import a hash object from its raw 35-byte representation
    pub fn try_from_raw(raw: &[u8]) -> Result<Self, TLSHDigestError> {
        if raw.len() != COLORED_HASH_SIZE {
            return Err(TLSHDigestError::InvalidLength);
        }
        let tlsh = TLSH::try_from_raw(&raw[1..])?;
        Ok(Self {
            color: raw[0],
            tlsh,
        })
    }

    /// Imports a hash object from its 35-byte raw representation.
    ///
    /// Panics if the input is not a valid hash
    pub fn from_raw(raw: &[u8]) -> Self {
        Self::try_from_raw(raw).unwrap()
    }

    /// Exports the hash object as a hex digest string
    pub fn to_digest(&self) -> String {
        self.to_raw().encode_hex_upper()
    }

    /// Tries to import a ColoredTLSH object from a digest string
    ///
    /// It supports loading the standard 70, the T1 versioned and the 72 long
    /// colored TLSH digests
    pub fn try_from_digest(digest: &str) -> Result<Self, TLSHDigestError> {
        let (color, digest) = match digest.len() {
            HEX_HASH_SIZE => (0, digest),
            VERSIONED_HEX_HASH_SIZE if digest.starts_with("T1") => (0,&digest[2..]),
            VERSIONED_HEX_HASH_SIZE if digest.starts_with("T") => return Err(TLSHDigestError::InvalidVersion),
            HEX_COLORED_HASH_SIZE => {
                let color = <[u8;1]>::from_hex(&digest[..2]).map_err(|_| TLSHDigestError::InvalidHex)?[0];
                (color, &digest[2..])
            },
            _ => return Err(TLSHDigestError::InvalidLength),
        };
        let hash_bytes = hex::decode(digest).map_err(|_| TLSHDigestError::InvalidHex)?;
        let tlsh = TLSH::try_from_raw(&hash_bytes)?;
        Ok(Self {
            color,
            tlsh,
        })
    }

    /// Import a ColoredTLSH object from a digest string
    /// 
    /// Panics if the digest is invalid
    pub fn from_digest(digest: &str) -> Self {
        Self::try_from_digest(digest).unwrap()
    }
    
}
