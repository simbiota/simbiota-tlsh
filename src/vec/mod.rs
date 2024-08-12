use std::sync::atomic::Ordering;
use ctor::ctor;
use crate::diff;

#[cfg(target_arch = "x86_64")]
mod x86 {
    use std::arch::x86_64::_mm256_loadu_si256;
    use std::sync::atomic::{AtomicBool, Ordering};
    use ctor::ctor;

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
            avx2::diff_codes_avx2(a,b)
        }
    }
    mod avx2;
}

type tlsh_diff_codes_fn = fn(a: &[u8; 32], b: &[u8;32]) -> u32;
static mut TLSH_DIFF_CODES_PTR: tlsh_diff_codes_fn = diff::tlsh_diff_codes_lut;

#[ctor]
fn init_diff_codes() {
    #[cfg(target_arch = "x86_64")]
    if x86::HAS_AVX2.load(Ordering::Relaxed) {
        unsafe { TLSH_DIFF_CODES_PTR = x86::tlsh_diff_codes_avx2 };
    }
}

pub fn tlsh_diff_codes(a: &[u8; 32], b: &[u8; 32]) -> u32 {
    unsafe {
        TLSH_DIFF_CODES_PTR(a,b)
    }
}