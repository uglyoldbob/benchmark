//! The code here is adapted from https://github.com/Mysticial/Flops/tree/master

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn reduce(x: core::arch::x86_64::__m128d) -> f64 {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;
    let x = _mm_add_pd(x, _mm_unpackhi_pd(x, x));
    return _mm_cvtsd_f64(x);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn load_sse2(count: usize) -> (usize, f64) {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mul0: __m128d = _mm_set1_pd(1.4142135623730950488);
    let mul1: __m128d = _mm_set1_pd(0.70710678118654752440);

    let mut d: [__m128d; 10] = [_mm_set1_pd(0.0); 10];
    for d in &mut d {
        *d = _mm_set1_pd(f64::from_bits(_rdtsc() % 256));
    }

    for _ in 0..count {
        d[0] = _mm_mul_pd(d[0], mul0);
        d[6] = _mm_add_pd(d[6], mul0);
        d[1] = _mm_mul_pd(d[1], mul0);
        d[7] = _mm_add_pd(d[7], mul0);
        d[2] = _mm_mul_pd(d[2], mul0);
        d[8] = _mm_add_pd(d[8], mul0);
        d[3] = _mm_mul_pd(d[3], mul0);
        d[9] = _mm_add_pd(d[9], mul0);
        d[4] = _mm_mul_pd(d[4], mul0);
        d[6] = _mm_sub_pd(d[6], mul0);
        d[5] = _mm_mul_pd(d[5], mul0);
        d[7] = _mm_sub_pd(d[7], mul0);

        d[0] = _mm_mul_pd(d[0], mul1);
        d[8] = _mm_sub_pd(d[8], mul0);
        d[1] = _mm_mul_pd(d[1], mul1);
        d[9] = _mm_sub_pd(d[9], mul0);
        d[2] = _mm_mul_pd(d[2], mul1);
        d[6] = _mm_add_pd(d[6], mul1);
        d[3] = _mm_mul_pd(d[3], mul1);
        d[7] = _mm_add_pd(d[7], mul1);
        d[4] = _mm_mul_pd(d[4], mul1);
        d[8] = _mm_add_pd(d[8], mul1);
        d[5] = _mm_mul_pd(d[5], mul1);
        d[9] = _mm_add_pd(d[9], mul1);

        d[0] = _mm_mul_pd(d[0], mul0);
        d[6] = _mm_sub_pd(d[6], mul1);
        d[1] = _mm_mul_pd(d[1], mul0);
        d[7] = _mm_sub_pd(d[7], mul1);
        d[2] = _mm_mul_pd(d[2], mul0);
        d[8] = _mm_sub_pd(d[8], mul1);
        d[3] = _mm_mul_pd(d[3], mul0);
        d[9] = _mm_sub_pd(d[9], mul1);
        d[4] = _mm_mul_pd(d[4], mul0);
        d[6] = _mm_add_pd(d[6], mul0);
        d[5] = _mm_mul_pd(d[5], mul0);
        d[7] = _mm_add_pd(d[7], mul0);

        d[0] = _mm_mul_pd(d[0], mul1);
        d[8] = _mm_add_pd(d[8], mul0);
        d[1] = _mm_mul_pd(d[1], mul1);
        d[9] = _mm_add_pd(d[9], mul0);
        d[2] = _mm_mul_pd(d[2], mul1);
        d[6] = _mm_sub_pd(d[6], mul0);
        d[3] = _mm_mul_pd(d[3], mul1);
        d[7] = _mm_sub_pd(d[7], mul0);
        d[4] = _mm_mul_pd(d[4], mul1);
        d[8] = _mm_sub_pd(d[8], mul0);
        d[5] = _mm_mul_pd(d[5], mul1);
        d[9] = _mm_sub_pd(d[9], mul0);
    }

    d[0] = _mm_add_pd(d[0], d[5]);
    d[1] = _mm_add_pd(d[1], d[6]);
    d[2] = _mm_add_pd(d[2], d[7]);
    d[3] = _mm_add_pd(d[3], d[8]);
    d[4] = _mm_add_pd(d[4], d[9]);

    d[0] = _mm_add_pd(d[0], d[3]);
    d[1] = _mm_add_pd(d[1], d[4]);

    d[0] = _mm_add_pd(d[0], d[1]);
    d[0] = _mm_add_pd(d[0], d[2]);

    return (96, reduce(d[0]));
}

pub fn rust_load_select(count: usize) -> (usize, f64) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("sse2") {
            return unsafe { load_sse2(count) };
        } else {
            return (1, 42.0);
        }
    }
    (1, 41.0)
}

pub fn load_select(count: usize) -> (usize, f64) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("sse2") {
            return unsafe { (96, cpuload::sse_load(count as u32)) };
        } else {
            return (1, 42.0);
        }
    }
    (1, 41.0)
}
