mod builder;
mod diff;
mod digest;
mod hash;
mod util;

pub use crate::{
    builder::{ColoredTLSHBuilder, TLSHBuilder, TLSHError},
    hash::TLSH, hash::ColoredTLSH, digest::TLSHDigestError,
};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let hash1 = "53152333A0D13738E4B172B10F6AC6135BEF7A225664750839D69F8D8E3B6C8D56932C";
        let hash2 = "94052217B1A73B39E46588F54EA5C09C2CFF3F222934210EB1ACA9491F7F7C0955A792";

        let hash1 = TLSH::from_digest(hash1);
        let hash2 = TLSH::from_digest(hash2);
        assert_eq!(TLSH::diff(&hash1, &hash2), 118);
        assert_eq!(hash1.to_digest(), "53152333A0D13738E4B172B10F6AC6135BEF7A225664750839D69F8D8E3B6C8D56932C");
        assert_eq!(hash2.to_digest(), "94052217B1A73B39E46588F54EA5C09C2CFF3F222934210EB1ACA9491F7F7C0955A792");
    }
}