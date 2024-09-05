use std::arch::x86_64::{
    __m256i, _mm256_add_epi32, _mm256_and_si256, _mm256_andnot_si256, _mm256_extracti128_si256,
    _mm256_or_si256, _mm256_set1_epi32, _mm256_slli_epi32, _mm256_srli_epi32, _mm256_xor_si256,
    _mm_add_epi32, _mm_extract_epi32,
};

#[inline(always)]
pub(crate) fn diff_codes_avx2(i: __m256i, j: __m256i) -> u32 {
    unsafe {
        let mut res = _mm256_xor_si256(i, j);
        let i0_j1 = _mm256_andnot_si256(i, j);
        let i1_j0 = _mm256_andnot_si256(j, i);

        let mask_a = _mm256_set1_epi32(0xAAAAAAAA_u32 as i32);
        let mask_5 = _mm256_set1_epi32(0x55555555);

        let even_01 = _mm256_and_si256(i0_j1, mask_a);
        let odd_01 = _mm256_slli_epi32::<1>(_mm256_and_si256(i0_j1, mask_5));

        let even_10 = _mm256_and_si256(i1_j0, mask_a);
        let odd_10 = _mm256_slli_epi32::<1>(_mm256_and_si256(i1_j0, mask_5));

        //uint32_t mask = (even_01 & odd_10) | (even_10 & odd_01);
        let mask = _mm256_or_si256(
            _mm256_and_si256(even_01, odd_10),
            _mm256_and_si256(even_10, odd_01),
        );

        //res &= (~mask);
        res = _mm256_andnot_si256(mask, res);

        //uint32_t odd_dups = res & 0x33333333;
        let mask3 = _mm256_set1_epi32(0x33333333);
        let odd_dups = _mm256_and_si256(res, mask3);

        //uint32_t three = odd_dups & (odd_dups << 1);
        let mut three = _mm256_and_si256(odd_dups, _mm256_slli_epi32::<1>(odd_dups));

        //uint32_t six = three | (three << 1);
        let mut six = _mm256_or_si256(three, _mm256_slli_epi32::<1>(three));

        //uint32_t masked_originals = odd_dups & ~(six >> 1);
        let mut masked_originals = _mm256_andnot_si256(_mm256_srli_epi32::<1>(six), odd_dups);

        //uint32_t s1 = six + masked_originals;
        let mut s1 = _mm256_add_epi32(six, masked_originals);

        //uint32_t even_dups = (res >> 2) & 0x33333333;
        let even_dups = _mm256_and_si256(_mm256_srli_epi32::<2>(res), mask3);

        //three = even_dups & (even_dups << 1);
        three = _mm256_and_si256(even_dups, _mm256_slli_epi32::<1>(even_dups));

        //six = three | (three << 1);
        six = _mm256_or_si256(three, _mm256_slli_epi32::<1>(three));

        //masked_originals = even_dups & ~(six >> 1);
        masked_originals = _mm256_andnot_si256(_mm256_srli_epi32::<1>(six), even_dups);

        //s1 += six + masked_originals;
        s1 = _mm256_add_epi32(s1, six);
        s1 = _mm256_add_epi32(s1, masked_originals);

        let mask_f0f0f0f0 = _mm256_set1_epi32(0xF0F0F0F0_u32 as i32);
        let mask0f0f0f0f = _mm256_set1_epi32(0x0F0F0F0F);

        let mask_ff00ff00 = _mm256_set1_epi32(0xFF00FF00_u32 as i32);
        let mask00ff00ff = _mm256_set1_epi32(0x00FF00FF);

        let mask_ffff0000 = _mm256_set1_epi32(0xFFFF0000_u32 as i32);
        let mask0000ffff = _mm256_set1_epi32(0x0000FFFF);

        //uint32_t even = s1 & 0xF0F0F0F0;
        let mut even = _mm256_and_si256(s1, mask_f0f0f0f0);

        //uint32_t odd = s1 & 0x0F0F0F0F;
        let mut odd = _mm256_and_si256(s1, mask0f0f0f0f);

        //s1 = (even >> 4) + odd;
        s1 = _mm256_add_epi32(_mm256_srli_epi32::<4>(even), odd);

        //even = s1 & 0xFF00FF00;
        even = _mm256_and_si256(s1, mask_ff00ff00);

        //odd = s1 & 0x00FF00FF;
        odd = _mm256_and_si256(s1, mask00ff00ff);

        //s1 = (even >> 8) + odd;
        s1 = _mm256_add_epi32(_mm256_srli_epi32::<8>(even), odd);

        //even = s1 & 0xFFFF0000;
        even = _mm256_and_si256(s1, mask_ffff0000);

        //odd = s1 & 0x0000FFFF;
        odd = _mm256_and_si256(s1, mask0000ffff);

        s1 = _mm256_add_epi32(_mm256_srli_epi32::<16>(even), odd);

        let s1_lo_128 = _mm256_extracti128_si256::<0>(s1);
        let s1_hi_128 = _mm256_extracti128_si256::<1>(s1);

        let su = _mm_add_epi32(s1_lo_128, s1_hi_128);

        let mut s = _mm_extract_epi32::<0>(su);
        s += _mm_extract_epi32::<1>(su);
        s += _mm_extract_epi32::<2>(su);
        s += _mm_extract_epi32::<3>(su);
        s as u32
    }
}
