use std::arch::aarch64::{uint8x16_t, vaddq_u32, vandq_u32, vandq_u8, veorq_u8, vld1q_dup_u32, vld1q_dup_u8, vmvnq_u32, vmvnq_u8, vorrq_u32, vreinterpretq_u32_u8, vshlq_n_u32, vshrq_n_u32, vst1q_u32};

pub(crate) fn diff_codes_neon(i: uint8x16_t, j: uint8x16_t) -> u32 {
    unsafe {
        let mut res = vreinterpretq_u32_u8(veorq_u8(i, j));
        let n_i = vmvnq_u8(i);
        let n_j = vmvnq_u8(j);

        let i0_j1 = vandq_u8(n_i, j);
        let i1_j0 = vandq_u8(i, n_j);

        let mask_a = vld1q_dup_u8(&0xaa);
        let mask_5 = vld1q_dup_u8(&0x55);

        let even_01 = vreinterpretq_u32_u8(vandq_u8(i0_j1, mask_a));
        let odd_01 = {
            let res = vandq_u8(i0_j1, mask_5);
            let as_32 = vreinterpretq_u32_u8(res);
            vshlq_n_u32::<1>(as_32)
        };

        let even_10 = vreinterpretq_u32_u8(vandq_u8(i1_j0, mask_a));
        let odd_10 = {
            let res = vandq_u8(i1_j0, mask_5);
            let as_32 = vreinterpretq_u32_u8(res);
            vshlq_n_u32::<1>(as_32)
        };

        let mask = vorrq_u32(
            vandq_u32(even_01, odd_10),
            vandq_u32(even_10, odd_01)
        );

        res = vandq_u32(vmvnq_u32(mask), res);

        let mask3= vld1q_dup_u32(&0x33333333);
        let odd_dups = vandq_u32(res, mask3);

        let mut three = vandq_u32(odd_dups, vshlq_n_u32::<1>(odd_dups));
        let mut six = vorrq_u32(three, vshlq_n_u32::<1>(three));

        let mut masked_originals = vandq_u32(vmvnq_u32(vshrq_n_u32::<1>(six)), odd_dups);

        let mut s1 = vaddq_u32(six, masked_originals);

        let even_dups = vandq_u32(vshrq_n_u32::<2>(res), mask3);

        three = vandq_u32(even_dups, vshlq_n_u32::<1>(even_dups));

        six = vorrq_u32(three, vshlq_n_u32::<1>(three));

        masked_originals = vandq_u32(vmvnq_u32(vshrq_n_u32::<1>(six)), even_dups);

        s1 = vaddq_u32(s1, six);
        s1 = vaddq_u32(s1, masked_originals);

        let mask_f0f0 = vld1q_dup_u32(&0xf0f0f0f0);
        let mask_0f0f = vld1q_dup_u32(&0x0f0f0f0f);
        let mask_ff00 = vld1q_dup_u32(&0xff00ff00);
        let mask_00ff = vld1q_dup_u32(&0x00ff00ff);
        let mask_ffff = vld1q_dup_u32(&0xffff0000);
        let mask_0000 = vld1q_dup_u32(&0x0000ffff);

        let mut even = vandq_u32(s1, mask_f0f0);
        let mut odd = vandq_u32(s1, mask_0f0f);

        s1 = vaddq_u32(vshrq_n_u32::<4>(even), odd);

        even = vandq_u32(s1, mask_ff00);
        odd = vandq_u32(s1, mask_00ff);

        s1 = vaddq_u32(vshrq_n_u32::<8>(even), odd);

        even = vandq_u32(s1, mask_ffff);
        odd = vandq_u32(s1, mask_0000);

        s1 = vaddq_u32(vshrq_n_u32::<16>(even), odd);

        let su = s1;
        let mut result = [0u32;4];
        vst1q_u32(result.as_mut_ptr(), su);

        result[0] + result[1] + result[2] + result[3]
    }
}
