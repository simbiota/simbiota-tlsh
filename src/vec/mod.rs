use crate::diff;
use ctor::ctor;
mod calc;
#[cfg(target_arch = "x86_64")]
mod x86 {
    use ctor::ctor;
    use std::arch::x86_64::_mm256_loadu_si256;
    use std::sync::atomic::{AtomicBool, Ordering};

    cpufeatures::new!(cpuid_avx2, "avx2");
    pub(crate) static HAS_AVX2: AtomicBool = AtomicBool::new(false);

    #[ctor]
    fn init_has_avx2() {
        let token: cpuid_avx2::InitToken = cpuid_avx2::init();
        let value = token.get();
        HAS_AVX2.store(value, Ordering::SeqCst);
    }

    pub(crate) fn tlsh_diff_codes_avx2(a: &[u8; 32], b: &[u8; 32]) -> u32 {
        unsafe {
            let a = _mm256_loadu_si256(a.as_ptr() as *const _);
            let b = _mm256_loadu_si256(b.as_ptr() as *const _);
            avx2::diff_codes_avx2(a, b)
        }
    }
    mod avx2;
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
        }
    }

    // overrides
    if std::env::var("TLSH_FORCE_CALC").is_ok() {
        unsafe {
            TLSH_DIFF_CODES_PTR = calc::tlsh_diff_codes_64;
            TLSH_DIFF_CODES_MODE = "calc";
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
