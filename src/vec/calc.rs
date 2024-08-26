fn tlsh_diff_f3_64(i: u64, j: u64) -> u32 {
    let mut res = i ^ j;

    let i0_j1 = (!i) & j;
    let i1_j0 = i & (!j);

    let even_01 = i0_j1 & 0xAAAAAAAAAAAAAAAA;
    let odd_01 = (i0_j1 & 0x5555555555555555) << 1;
    let even_10 = i1_j0 & 0xAAAAAAAAAAAAAAAA;
    let odd_10 = (i1_j0 & 0x5555555555555555) << 1;

    let mask = (even_01 & odd_10) | (even_10 & odd_01);

    res &= (!mask);

    let odd_dups = res & 0x3333333333333333;
    let mut three = odd_dups & (odd_dups << 1);
    let mut six = three | (three << 1);
    let mut masked_originals = odd_dups & !(six >> 1);

    let mut s1 = six + masked_originals;

    let even_dups = (res >> 2) & 0x3333333333333333;
    three = even_dups & (even_dups << 1);
    six = three | (three << 1);
    masked_originals = even_dups & !(six >> 1);

    s1 += six + masked_originals;

    let mut even = s1 & 0xF0F0F0F0F0F0F0F0;
    let mut odd = s1 & 0x0F0F0F0F0F0F0F0F;
    s1 = (even >> 4) + odd;

    even = s1 & 0xFF00FF00FF00FF00;
    odd = s1 & 0x00FF00FF00FF00FF;
    s1 = (even >> 8) + odd;

    even = s1 & 0xFFFF0000FFFF0000;
    odd = s1 & 0x0000FFFF0000FFFF;

    s1 = (even >> 16) + odd;

    even = s1 & 0xFFFFFFFF00000000;
    odd = s1 & 0x00000000FFFFFFFF;

    ((even >> 32) + odd) as u32
}

pub(crate) fn tlsh_diff_codes_64(a: &[u8; 32], b: &[u8;32]) -> u32 {
    let mut d = 0u32;
    for i in 0..4 {
        d += tlsh_diff_f3_64(unsafe { *(a.as_ptr().add(i * 8) as *const u64)}, unsafe { *(b.as_ptr().add(i * 8) as *const u64)});
    }
    d
}