pub const EFF_BUCKETS: usize = 128;

/// A comparable TLSH hash object
///
/// Use `TLSHBuilder` to calculate the hash object for any data.
///
/// A hash object can be converted to and parsed from raw bytes or a digest string.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct TLSH {
    pub checksum: u8,
    pub lvalue: u8,
    pub q_ratios: u8,
    pub codes: [u8; EFF_BUCKETS / 4],
}

/// A comparable, colored TLSH hash object
/// Use ColoredTLSHBuilder to calculate the hash object for any data.
///
/// A hash object can be converted to and parsed from raw bytes or a digest string.
#[derive(Copy, Clone, Debug)]
pub struct ColoredTLSH {
    pub color: u8,
    pub tlsh: TLSH,
}