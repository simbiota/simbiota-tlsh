use crate::tlsh::TLSH;

use ::hex::{FromHex, ToHex};

const BODY_SIZE: usize = 32;
const HASH_SIZE: usize = 3 + BODY_SIZE;
const COLORED_HASH_SIZE: usize = 1 + HASH_SIZE;

fn swap_byte(x: u8) -> u8 {
    x >> 4 | x << 4
}

impl TLSH {
    /// Exports the hash object as its 36-byte colored raw representation
    pub fn to_raw(&self) -> [u8; COLORED_HASH_SIZE] {
        let mut raw = [0u8; COLORED_HASH_SIZE];
        raw[0] = self.color;
        raw[1] = swap_byte(self.checksum);
        raw[2] = swap_byte(self.lvalue);
        raw[3] = swap_byte(self.q_ratios);
        for (r, c) in raw[4..].iter_mut().rev().zip(self.codes.iter()) {
            *r = *c;
        }
        raw
    }

    /// Imports a hash object from its 35-byte or 36-byte (colored) raw representation
    pub fn from_raw(raw: &[u8]) -> Self {
        let (color, hash_bytes) = match raw.len() {
            HASH_SIZE => (0, raw),
            COLORED_HASH_SIZE => (raw[0], &raw[1..]),
            _ => panic!("Trying to import hash from raw bytes with invalid length"),
        };
        let mut codes = [0u8; BODY_SIZE];
        for (c, r) in codes.iter_mut().zip(hash_bytes[3..].iter().rev()) {
            *c = *r;
        }
        Self {
            color,
            checksum: swap_byte(hash_bytes[0]),
            lvalue: swap_byte(hash_bytes[1]),
            q_ratios: swap_byte(hash_bytes[2]),
            codes,
        }
    }

    /// Exports the hash object as a digest string
    pub fn to_digest(&self) -> String {
        self.to_raw().encode_hex_upper()
    }

    /// Imports a hash object from a digest string
    pub fn from_digest(digest: &str) -> Result<Self, String> {
        if digest.len() == 2 * HASH_SIZE {
            match <[u8; HASH_SIZE]>::from_hex(digest) {
                Ok(raw) => Ok(Self::from_raw(&raw)),
                _ => Err(String::from("Error parsing hex")),
            }
        } else if digest.len() == 2 * HASH_SIZE + 2 {
            let prefix = &digest[..2];
            let digest: String = match prefix {
                "T1" => format!("00{}", &digest[2..]),
                _ => digest.to_owned(),
            };
            match <[u8; COLORED_HASH_SIZE]>::from_hex(digest) {
                Ok(raw) => Ok(Self::from_raw(&raw)),
                _ => Err(String::from("Error parsing hex")),
            }
        } else {
            return Err(String::from(
                "Trying to import hash from digest string with invalid length",
            ));
        }
    }
}
