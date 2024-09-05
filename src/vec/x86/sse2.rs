use std::arch::x86_64::{__m128i, _mm_add_epi32, _mm_and_si128, _mm_andnot_si128, _mm_extract_epi32, _mm_or_si128, _mm_set1_epi32, _mm_slli_epi32, _mm_srli_epi32, _mm_xor_si128};

pub(crate) fn diff_codes_sse2(i: __m128i, j: __m128i) -> u32 {
    unsafe {
        let mut res = _mm_xor_si128(i, j);
        let i0_j1 = _mm_andnot_si128(i, j);
        let i1_j0 = _mm_andnot_si128(j, i);
        
        let mask_a = _mm_set1_epi32(0xaaaaaaaa_u32 as i32);
        let mask_5 = _mm_set1_epi32(0x55555555_u32 as i32);

        let even_01 = _mm_and_si128(i0_j1, mask_a);
        let odd_01 = _mm_slli_epi32::<1>(_mm_and_si128(i0_j1, mask_5));
        
        let even_10 = _mm_and_si128(i1_j0, mask_a);
        let odd_10 = _mm_slli_epi32::<1>(_mm_and_si128(i1_j0, mask_5));

        let mask = _mm_or_si128(
            _mm_and_si128(even_01, odd_10),
            _mm_and_si128(even_10, odd_01),
        );

        //res &= (~mask);
        res = _mm_andnot_si128(mask, res);

        //uint32_t odd_dups = res & 0x33333333;
        let mask3 = _mm_set1_epi32(0x33333333);
        let odd_dups = _mm_and_si128(res, mask3);

        //uint32_t three = odd_dups & (odd_dups << 1);
        let mut three = _mm_and_si128(odd_dups, _mm_slli_epi32::<1>(odd_dups));

        //uint32_t six = three | (three << 1);
        let mut six = _mm_or_si128(three, _mm_slli_epi32::<1>(three));

        //uint32_t masked_originals = odd_dups & ~(six >> 1);
        let mut masked_originals = _mm_andnot_si128(_mm_srli_epi32::<1>(six), odd_dups);

        //uint32_t s1 = six + masked_originals;
        let mut s1 = _mm_add_epi32(six, masked_originals);

        //uint32_t even_dups = (res >> 2) & 0x33333333;
        let even_dups = _mm_and_si128(_mm_srli_epi32::<2>(res), mask3);

        //three = even_dups & (even_dups << 1);
        three = _mm_and_si128(even_dups, _mm_slli_epi32::<1>(even_dups));

        //six = three | (three << 1);
        six = _mm_or_si128(three, _mm_slli_epi32::<1>(three));

        //masked_originals = even_dups & ~(six >> 1);
        masked_originals = _mm_andnot_si128(_mm_srli_epi32::<1>(six), even_dups);

        //s1 += six + masked_originals;
        s1 = _mm_add_epi32(s1, six);
        s1 = _mm_add_epi32(s1, masked_originals);

        let mask_f0f0f0f0 = _mm_set1_epi32(0xF0F0F0F0_u32 as i32);
        let mask0f0f0f0f = _mm_set1_epi32(0x0F0F0F0F);

        let mask_ff00ff00 = _mm_set1_epi32(0xFF00FF00_u32 as i32);
        let mask00ff00ff = _mm_set1_epi32(0x00FF00FF);

        let mask_ffff0000 = _mm_set1_epi32(0xFFFF0000_u32 as i32);
        let mask0000ffff = _mm_set1_epi32(0x0000FFFF);

        //uint32_t even = s1 & 0xF0F0F0F0;
        let mut even = _mm_and_si128(s1, mask_f0f0f0f0);

        //uint32_t odd = s1 & 0x0F0F0F0F;
        let mut odd = _mm_and_si128(s1, mask0f0f0f0f);

        //s1 = (even >> 4) + odd;
        s1 = _mm_add_epi32(_mm_srli_epi32::<4>(even), odd);

        //even = s1 & 0xFF00FF00;
        even = _mm_and_si128(s1, mask_ff00ff00);

        //odd = s1 & 0x00FF00FF;
        odd = _mm_and_si128(s1, mask00ff00ff);

        //s1 = (even >> 8) + odd;
        s1 = _mm_add_epi32(_mm_srli_epi32::<8>(even), odd);

        //even = s1 & 0xFFFF0000;
        even = _mm_and_si128(s1, mask_ffff0000);

        //odd = s1 & 0x0000FFFF;
        odd = _mm_and_si128(s1, mask0000ffff);

        s1 = _mm_add_epi32(_mm_srli_epi32::<16>(even), odd);

        let su = s1;

        let mut s = _mm_extract_epi32::<0>(su);
        s += _mm_extract_epi32::<1>(su);
        s += _mm_extract_epi32::<2>(su);
        s += _mm_extract_epi32::<3>(su);
        s as u32
    }
}
