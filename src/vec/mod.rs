use crate::diff;
use ctor::ctor;
use std::arch::is_aarch64_feature_detected;
mod cpu;
#[cfg(target_arch = "x86_64")]
mod x86 {
    use ctor::ctor;
    use std::arch::x86_64::_mm256_loadu_si256;
    use std::arch::x86_64::_mm_loadu_si128;
    use std::sync::atomic::{AtomicBool, Ordering};

    cpufeatures::new!(cpuid_avx2, "avx2");
    cpufeatures::new!(cpuid_sse2, "sse2");
    pub(crate) static HAS_AVX2: AtomicBool = AtomicBool::new(false);
    pub(crate) static HAS_SSE2: AtomicBool = AtomicBool::new(false);

    #[ctor]
    fn init_cpuid() {
        let token: cpuid_avx2::InitToken = cpuid_avx2::init();
        let value = token.get();
        HAS_AVX2.store(value, Ordering::SeqCst);

        let sse2_token = cpuid_sse2::init();
        let sse2_value = sse2_token.get();
        HAS_SSE2.store(sse2_value, Ordering::SeqCst);
    }

    pub(crate) fn tlsh_diff_codes_avx2(a: &[u8; 32], b: &[u8; 32]) -> u32 {
        unsafe {
            let a = _mm256_loadu_si256(a.as_ptr() as *const _);
            let b = _mm256_loadu_si256(b.as_ptr() as *const _);
            avx2::diff_codes_avx2(a, b)
        }
    }
    pub(crate) fn tlsh_diff_codes_sse2(a: &[u8; 32], b: &[u8; 32]) -> u32 {
        unsafe {
            let a1 = _mm_loadu_si128(a.as_ptr().cast());
            let a2 = _mm_loadu_si128(a.as_ptr().add(128 / 8).cast());

            let b1 = _mm_loadu_si128(b.as_ptr().cast());
            let b2 = _mm_loadu_si128(b.as_ptr().add(128 / 8).cast());

            sse2::diff_codes_sse2(a1, b1) + sse2::diff_codes_sse2(a2, b2)
        }
    }
    mod avx2;
    mod sse2;
}

#[cfg(target_arch = "aarch64")]
mod aarch64 {
    use std::arch::aarch64::{uint8x16_t, vld1q_u8};
    pub(crate) fn tlsh_diff_codes_neon(a: &[u8;32], b: &[u8;32]) -> u32 {
        unsafe {
            let a1: uint8x16_t = vld1q_u8(a.as_ptr().cast());
            let a2: uint8x16_t = vld1q_u8(a.as_ptr().add(16).cast());

            let b1: uint8x16_t = vld1q_u8(b.as_ptr().cast());
            let b2: uint8x16_t = vld1q_u8(b.as_ptr().add(16).cast());
            
            neon::diff_codes_neon(a1, b1) + neon::diff_codes_neon(a2, b2)
        }
    }
    mod neon;
}

type TlshDiffCodesFn = fn(a: &[u8; 32], b: &[u8; 32]) -> u32;
static mut TLSH_DIFF_CODES_PTR: TlshDiffCodesFn = diff::tlsh_diff_codes_lut;
static mut TLSH_DIFF_CODES_MODE: &str = "LUT";

#[ctor]
fn init_diff_codes() {
    #[cfg(target_arch = "x86_64")]
    {
        use std::sync::atomic::Ordering;
        if x86::HAS_AVX2.load(Ordering::Relaxed) && std::env::var("TLSH_DISABLE_AVX").is_err() {
            unsafe {
                TLSH_DIFF_CODES_PTR = x86::tlsh_diff_codes_avx2;
                TLSH_DIFF_CODES_MODE = "avx2";
            }
        } else if x86::HAS_SSE2.load(Ordering::Relaxed)
            && std::env::var("TLSH_DISABLE_SSE").is_err()
        {
            unsafe {
                TLSH_DIFF_CODES_PTR = x86::tlsh_diff_codes_sse2;
                TLSH_DIFF_CODES_MODE = "sse2";
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") && std::env::var("TLSH_DISABLE_NEON").is_err() {
            unsafe {
                TLSH_DIFF_CODES_PTR = aarch64::tlsh_diff_codes_neon;
                TLSH_DIFF_CODES_MODE = "neon";
            }
        }
    }

    // overrides
    if std::env::var("TLSH_FORCE_CPU").is_ok() {
        unsafe {
            TLSH_DIFF_CODES_PTR = cpu::tlsh_diff_codes_64;
            TLSH_DIFF_CODES_MODE = "cpu";
        }
    }
}

#[inline(always)]
pub fn tlsh_diff_mode() -> &'static str {
    unsafe { TLSH_DIFF_CODES_MODE }
}

#[inline(always)]
pub fn tlsh_diff_codes(a: &[u8; 32], b: &[u8; 32]) -> u32 {
    unsafe { TLSH_DIFF_CODES_PTR(a, b) }
}
