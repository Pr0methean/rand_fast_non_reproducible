use crate::generate::Simd64;
use core::arch::x86_64::*;
use core::mem::transmute;

// ============================================================================
// AVX2-optimized 64-bit multiplication (the ONLY operation that differs)
// ============================================================================

/// Multiply two vectors of u64 lanes, keeping the low 64 bits of each product.
/// This uses `_mm256_mul_epu32` to avoid the scalarization that portable SIMD does.
#[inline(always)]
pub fn wrapping_mul(a: Simd64, b: Simd64) -> Simd64 {
    unsafe {
        let a = transmute::<Simd64, __m256i>(a);
        let b = transmute::<Simd64, __m256i>(b);
        transmute::<__m256i, Simd64>(mullo_u64x4_avx2(a, b))
    }
}

#[inline(always)]
unsafe fn mullo_u64x4_avx2(a: __m256i, b: __m256i) -> __m256i {
    unsafe {
        let a_hi = _mm256_srli_epi64(a, 32);
        let b_hi = _mm256_srli_epi64(b, 32);

        let p0 = _mm256_mul_epu32(a, b);
        let p1 = _mm256_mul_epu32(a_hi, b);
        let p2 = _mm256_mul_epu32(a, b_hi);

        let p12 = _mm256_add_epi64(p1, p2);
        let p12_shift = _mm256_slli_epi64(p12, 32);

        _mm256_add_epi64(p0, p12_shift)
    }
}

/// Multiplies two vectors. Requires that all elements of b be less than 2^32. Returns (low, hi)
/// halves of result.
pub fn mul_small(a: Simd64, b: Simd64) -> (Simd64, Simd64) {
    unsafe {
        let a = transmute::<Simd64, __m256i>(a);
        let b = transmute::<Simd64, __m256i>(b);
        let (lo, hi) = mul_small_avx2(a, b);
        (
            transmute::<__m256i, Simd64>(lo),
            transmute::<__m256i, Simd64>(hi),
        )
    }
}

/// Requires that all elements of kvec as a u64x4 be less than 2^32.
#[inline(always)]
unsafe fn mul_small_avx2(x: __m256i, kvec: __m256i) -> (__m256i, __m256i) {
    unsafe {
        // x = x_hi * 2^32 + x_lo
        let x_hi = _mm256_srli_epi64(x, 32);

        // p0 = x_lo * k
        let p0 = _mm256_mul_epu32(x, kvec);

        // p1 = x_hi * k
        let p1 = _mm256_mul_epu32(x_hi, kvec);

        // kx = p1 * 2^32 + p0 = (p1 + (p0 >> 32)) * 2^32 + (p0 & 0xffffffff)
        let p0_hi = _mm256_srli_epi64(p0, 32);
        let s = _mm256_add_epi64(p1, p0_hi);

        // construct low word: lo = (s << 32) | (p0 & 0xffffffff)
        let lo_low = _mm256_and_si256(p0, _mm256_set1_epi64x(0xffffffff));
        let lo_high = _mm256_slli_epi64(s, 32);
        let lo = _mm256_or_si256(lo_low, lo_high);

        // construct high word: hi = s >> 32
        let hi = _mm256_srli_epi64(s, 32);

        (lo, hi)
    }
}

/// Optimized multi-product: calculates (a*b, b*c) sharing the preparation of 'b'.
#[inline(always)]
pub unsafe fn mul_lo_hi_triad_avx2(
    a: __m256i,
    b: __m256i,
    c: __m256i,
) -> (__m256i, __m256i, __m256i, __m256i) {
    unsafe {
        let b_odd = _mm256_srli_epi64(b, 32);

        // a*b even/odd
        let ab_even = _mm256_mul_epu32(a, b);
        let a_odd = _mm256_srli_epi64(a, 32);
        let ab_odd = _mm256_mul_epu32(a_odd, b_odd);

        // b*c even/odd
        let bc_even = _mm256_mul_epu32(b, c);
        let c_odd = _mm256_srli_epi64(c, 32);
        let bc_odd = _mm256_mul_epu32(b_odd, c_odd);

        let ab_lo = _mm256_blend_epi32(ab_even, _mm256_slli_epi64(ab_odd, 32), 0xAA);
        let ab_hi = _mm256_blend_epi32(_mm256_srli_epi64(ab_even, 32), ab_odd, 0xAA);

        let bc_lo = _mm256_blend_epi32(bc_even, _mm256_slli_epi64(bc_odd, 32), 0xAA);
        let bc_hi = _mm256_blend_epi32(_mm256_srli_epi64(bc_even, 32), bc_odd, 0xAA);

        (ab_lo, ab_hi, bc_lo, bc_hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Helper: check that mullo(a, b) == element-wise a[i].wrapping_mul(b[i])
    fn assert_mullo_eq(a_arr: [u64; 4], b_arr: [u64; 4]) {
        let a = Simd64::from_array(a_arr);
        let b = Simd64::from_array(b_arr);
        let result = wrapping_mul(a, b).to_array();
        let expected: [u64; 4] = std::array::from_fn(|i| a_arr[i].wrapping_mul(b_arr[i]));
        assert_eq!(result, expected, "mullo({a_arr:?}, {b_arr:?})");
    }

    #[test]
    fn test_mullo_identity() {
        let a = [3, 7, 0x1_0000_0001, 0xFFFF_FFFF_FFFF_FFFF];
        let ones = [1; 4];
        assert_mullo_eq(a, ones);
        // Commutativity
        assert_mullo_eq(ones, a);
    }

    #[test]
    fn test_mullo_zero() {
        let a = [42, u64::MAX, 1, 0x1234_5678_9ABC_DEF0];
        let zeros = [0; 4];
        assert_mullo_eq(a, zeros);
        assert_mullo_eq(zeros, a);
    }

    #[test]
    fn test_mullo_small_values() {
        assert_mullo_eq([2, 3, 4, 5], [10, 20, 30, 40]);
    }

    #[test]
    fn test_mullo_large_overflow() {
        // Products that overflow 64 bits (only low 64 bits should be kept)
        assert_mullo_eq(
            [u64::MAX, u64::MAX, 0x8000_0000_0000_0000, 0xFFFF_FFFF],
            [2, u64::MAX, 2, 0xFFFF_FFFF],
        );
    }

    #[test]
    fn test_mullo_commutativity() {
        let a = [0x1234_5678_9ABC_DEF0, 0xFEDC_BA98_7654_3210, 42, 0];
        let b = [0x0F0F_0F0F_0F0F_0F0F, 0xAAAA_BBBB_CCCC_DDDD, 99, 1];
        let r1 = wrapping_mul(Simd64::from_array(a), Simd64::from_array(b)).to_array();
        let r2 = wrapping_mul(Simd64::from_array(b), Simd64::from_array(a)).to_array();
        assert_eq!(r1, r2, "mullo should be commutative");
    }

    #[test]
    fn test_mullo_powers_of_two() {
        // Multiplying by powers of two should be equivalent to left shifts
        let a = [0x0123_4567_89AB_CDEF; 4];
        let pows = [1 << 0, 1 << 1, 1 << 16, 1 << 32];
        assert_mullo_eq(a, pows);
    }

    #[test]
    fn test_mullo_max_squared() {
        // MAX * MAX = 1 (mod 2^64)
        let maxes = [u64::MAX; 4];
        let result = wrapping_mul(Simd64::from_array(maxes), Simd64::from_array(maxes)).to_array();
        assert_eq!(result, [1; 4], "(-1)^2 ≡ 1 (mod 2^64)");
    }

    #[test]
    fn test_mullo_const_basic() {
        let a = [10, 20, 30, 40];
        let c = 7u64;
        let p0 = Simd64::from_array(a);
        let result = wrapping_mul(p0, Simd64::splat(c)).to_array();
        let expected: [u64; 4] = std::array::from_fn(|i| a[i].wrapping_mul(c));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mullo_const_overflow() {
        let a = [u64::MAX, 0x8000_0000_0000_0001, 1, 0];
        let c = u64::MAX;
        let p0 = Simd64::from_array(a);
        let result = wrapping_mul(p0, Simd64::splat(c)).to_array();
        let expected: [u64; 4] = std::array::from_fn(|i| a[i].wrapping_mul(c));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mullo_const_zero() {
        let a = [42, u64::MAX, 1, 0x1234_5678_9ABC_DEF0];
        let p0 = Simd64::from_array(a);
        let result = wrapping_mul(p0, Simd64::splat(0)).to_array();
        assert_eq!(result, [0; 4]);
    }

    #[test]
    fn test_mullo_const_one() {
        let a = [42, u64::MAX, 1, 0x1234_5678_9ABC_DEF0];
        let p0 = Simd64::from_array(a);
        let result = wrapping_mul(p0, Simd64::splat(1)).to_array();
        assert_eq!(result, a);
    }

    #[test]
    fn test_mullo_mixed_hi_lo_bits() {
        // Specifically exercises the cross-term (a_hi*b_lo + a_lo*b_hi) logic
        assert_mullo_eq(
            [
                0x0000_0001_0000_0000,
                0x0000_0000_0000_0001,
                0x8000_0000_8000_0000,
                0x1_0000_0001,
            ],
            [
                0x0000_0000_0000_0002,
                0x0000_0001_0000_0000,
                0x0000_0002_0000_0002,
                0x1_0000_0001,
            ],
        );
    }

    #[test]
    fn test_mulsmall() {
        assert_eq!(
            mul_small(Simd64::splat((1 << 63) + 1), Simd64::splat((1 << 32) - 1)),
            (
                Simd64::splat((1 << 63) | (1 << 32) - 1),
                Simd64::splat((1 << 31) - 1)
            )
        );
    }

    proptest! {
        #[test]
        fn test_mul_small_proptest(a in any::<[u64; 4]>(), b in any::<[u32; 4]>()) {
            let a_simd = Simd64::from_array(a);
            let b_u64 = b.map(|x| x as u64);
            let b_simd = Simd64::from_array(b_u64);
            let (lo, hi) = mul_small(a_simd, b_simd);
            let lo_arr = lo.to_array();
            let hi_arr = hi.to_array();

            for i in 0..4 {
                let expected = (a[i] as u128) * (b_u64[i] as u128);
                let actual = (lo_arr[i] as u128) | ((hi_arr[i] as u128) << 64);
                assert_eq!(actual, expected, "mul_small failed for a={} b={}", a[i], b_u64[i]);
            }
        }
    }

    proptest! {
        #[test]
        fn test_wrapping_mul_proptest(a in any::<[u64; 4]>(), b_u64 in any::<[u64; 4]>()) {
            let a_simd = Simd64::from_array(a);
            let b_simd = Simd64::from_array(b_u64);
            let lo = wrapping_mul(a_simd, b_simd);
            let lo_arr = lo.to_array();

            for i in 0..4 {
                let expected = (((a[i] as u128) * (b_u64[i] as u128)) & (u64::MAX as u128)) as u64;
                let actual = lo_arr[i];
                assert_eq!(actual, expected, "mul_small failed for a={} b={}", a[i], b_u64[i]);
            }
        }
    }
}
