#![feature(portable_simd)]
#![feature(generic_const_exprs)]
#![allow(long_running_const_eval)]
#![allow(incomplete_features)]
#![cfg_attr(feature = "jump", allow(long_running_const_eval))]
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
))]
mod avx2;
mod generate;
#[cfg(feature = "jump")]
pub mod jump;
pub mod reproducibility;
pub mod seed;
#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "zeroize")]
mod zeroize;

use crate::generate::{Simd32, MIX_OUTPUTS, SIMD_WIDTH};
use crate::reproducibility::{DefaultReproducibility, NotReproducible};
use const_format::formatcp;
use core::convert::Infallible;
use core::marker::PhantomData;
use std::simd::{simd_swizzle, Simd};
use bytemuck::cast;
use generate::Simd64;
use rand_core::TryRng;
use rand_core::block::BlockRng;
use reproducibility::Reproducibility;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TripleMixSimdCore<R: Reproducibility> {
    tm0: Simd64, // TinyMT64 state
    tm1: Simd64, // TinyMT64 state, with highest bit always 0
    mwc_state: Simd64,
    mwc_carry: Simd64,
    pcg_state_lo: Simd64,
    pcg_state_hi: Simd64,
    pcg_inc_lo: Simd64,
    pcg_inc_hi: Simd64,
    xoshiro256: [u64; 4],
    reproducibility: PhantomData<R>,
}

impl <R: Reproducibility> std::fmt::Debug for TripleMixSimdCore<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x2 = self.mwc_carry;
        let x3 = self.mwc_state;
        let x4 = self.tm1;
        let x5 = self.tm0;
        let x6 = self.pcg_state_hi;
        let x7 = self.pcg_state_lo;
        let x8 = self.pcg_inc_hi;
        let x9 = self.pcg_inc_lo;
        let x10 = self.xoshiro256;
        f.debug_struct("TripleMixSimdCore")
            .field("pcg_state_lo", &x7.to_array())
            .field("pcg_state_hi", &x6.to_array())
            .field("pcg_inc_lo", &x9.to_array())
            .field("pcg_inc_hi", &x8.to_array())
            .field("tm0", &x5.to_array())
            .field("tm1", &x4.to_array())
            .field("mwc_state", &x3.to_array())
            .field("mwc_carry", &x2.to_array())
            .field("xoshiro256", &x10)
            .finish()
    }
}

impl <R: Reproducibility> TripleMixSimdCore<R> {

    #[inline(always)]
    pub(crate) fn mul_lo_hi(a: Simd32, b: Simd32) -> (Simd32, Simd32) {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
        ))]
        {
            let (lo, hi) = unsafe { avx2::mul_lo_hi_interleaved_avx2(cast(a), cast(b)) };
            (cast(lo), cast(hi))
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
        )))]
        {
            portable_mul_lo_hi(a, b)
        }
    }

    #[allow(unused)]
    #[inline(always)]
    fn portable_mul_lo_hi(a: Simd32, b: Simd32) -> (Simd32, Simd32) {
        let a64: Simd64 = cast(a);
        let b64: Simd64 = cast(b);
        let mask32 = Simd64::splat(0xFFFF_FFFF);
        // Even lanes (lower 32 bits of each 64-bit word)
        let even = (a64 & mask32) * (b64 & mask32);

        // Odd lanes
        let a_hi = a64 >> Simd::splat(32);
        let b_hi = b64 >> Simd::splat(32);
        let odd = a_hi * b_hi;

        // Reinterpret to u32
        let even32: Simd32 = cast(even);
        let odd32: Simd32 = cast(odd);

        let lo = simd_swizzle!(even32, odd32, [0, 8, 2, 10, 4, 12, 6, 14]);
        let hi = simd_swizzle!(even32, odd32, [1, 9, 3, 11, 5, 13, 7, 15]);

        (lo, hi)
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn mix(
        x0: Simd64,
        x1: Simd64,
        x2: Simd64,
        x3: Simd64,
        x4: Simd64,
        x5: Simd64,
        x6: Simd64,
        scalar: u64,
    ) -> (Simd64, Simd64, Simd64) {
        // Convert inputs to u32x8 (portable)
        let xi = [
            cast(x0),
            cast(x1),
            cast(x2),
            cast(x3),
            cast(x4),
            cast(x5),
            cast(x6),
        ];

        // Rotation helper
        #[inline(always)]
        fn rotl32(x: Simd32, k: u32) -> Simd32 {
            (x << Simd32::splat(k)) | (x >> Simd32::splat(32 - k))
        }

        #[allow(clippy::too_many_arguments)]
        #[inline(always)]
        fn round3<R: Reproducibility>(
            mut a: Simd32,
            mut b: Simd32,
            mut c: Simd32,
            x: &[Simd32; 7],
            shift1: u32,
            shift2: u32,
            shift3: u32,
            shift4: u32,
            shift5: u32,
            shift6: u32,
        ) -> (Simd32, Simd32, Simd32) {
            // --- First nonlinear layer ---
            let (m0_lo, m0_hi) = TripleMixSimdCore::<R>::mul_lo_hi(a, b);
            let (m1_lo, m1_hi) = TripleMixSimdCore::<R>::mul_lo_hi(b, c);

            a ^= b.rotate_elements_left::<1>();
            b += c.rotate_elements_right::<3>();
            c ^= a.rotate_elements_left::<3>();

            // --- Input injection ---
            a += x[0];
            b ^= x[1];
            c += x[2];

            a ^= m1_hi + b.rotate_elements_left::<2>();
            b ^= m0_lo ^ c.rotate_elements_right::<3>();
            c ^= m0_hi + a.rotate_elements_left::<1>();

            a ^= x[3];
            b += x[4];
            c ^= x[5];

            // --- Rotate ---
            a = rotl32(a, shift1);
            c = rotl32(c, shift3);

            // --- Second nonlinear layer ---
            let (m2_lo, m2_hi) = TripleMixSimdCore::<R>::mul_lo_hi(a, c);
            b += m1_lo + a.rotate_elements_right::<2>();
            b = rotl32(b, shift2);
            let b_rotated = b.rotate_elements_left::<3>();
            a += m2_hi ^ x[6];
            c += m2_lo ^ b_rotated;

            // --- Final rotate ---
            a = rotl32(a, shift4);
            b = rotl32(b, shift5);
            c = rotl32(c, shift6);

            (a, b, c)
        }
        let scalar_hi = (scalar >> 32) as u32;
        let scalar_lo = scalar as u32;
        let mut a = Simd32::splat(0x243f6a88);
        let scalar_mix_1 = Simd32::from_array([0, scalar_lo, scalar_hi, 0, scalar_hi, 0, scalar_lo, 0]);
        let mut b = Simd32::splat(0x9e3779b9);
        let scalar_mix_2 = Simd32::from_array([scalar_hi, 0, 0, scalar_hi, 0, scalar_lo, 0, scalar_lo]);
        a ^= scalar_mix_1;
        let mut c = Simd32::splat(0xb7e15162);
        b += scalar_mix_2;

        (a, b, c) = round3::<R>(a, b, c, &xi, 7, 19, 26, 11, 23, 31);
        c += scalar_mix_1;
        (a, b, c) = round3::<R>(
            a,
            b,
            c,
            &[xi[3], xi[4], xi[5], xi[6], xi[0], xi[1], xi[2]],
            5,
            17,
            29,
            9,
            21,
            27,
        );
        a ^= scalar_mix_2;
        (a, b, c) = round3::<R>(
            a,
            b,
            c,
            &[xi[6], xi[2], xi[5], xi[0], xi[4], xi[1], xi[3]],
            3,
            13,
            25,
            15,
            27,
            9,
        );

        // --- Strong final cross-lane avalanche ---
        a ^= b.rotate_elements_right::<2>();
        b += c.rotate_elements_left::<3>();
        c += a.rotate_elements_right::<4>();

        // Convert back to u64x4 by casting and packing
        (cast(a), cast(b), cast(c))
    }

    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((self as *const Self) as *const u8, size_of::<Self>()) }
    }
}

/// Instances must not be used again after being zeroized.
#[derive(Clone, Debug)]
pub struct TripleMixPrng<R: Reproducibility = DefaultReproducibility> {
    block_core: BlockRng<TripleMixSimdCore<R>>,
    reproducibility: PhantomData<R>,
}

pub const TRIPLE_MIX_PRNG_OID: &str = "1.3.6.1.4.1.54392.5.3311";
pub const VERSION_OID: &str = formatcp!("{TRIPLE_MIX_PRNG_OID}.{MAJOR_VERSION}.{MINOR_VERSION}");

impl TripleMixPrng<NotReproducible> {
    #[inline(always)]
    pub fn fill_blocks_unbuffered(&mut self, blocks: &mut [[u64; BLOCK_SIZE]]) {
        self.block_core.core.fill_blocks(blocks);
    }
}

impl<R: Reproducibility> TryRng for TripleMixPrng<R> {
    type Error = Infallible;

    #[inline(always)]
    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        let next_u64 = self.try_next_u64()?;
        Ok((next_u64 >> 32 ^ next_u64) as u32)
    }

    #[inline(always)]
    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        Ok(self.block_core.next_word())
    }

    #[inline(always)]
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        R::fill_bytes(&mut self.block_core, dst);
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn create_rngs<R: Reproducibility>() -> [TripleMixPrng<R>; 5] {
    use crate::seed::DEFAULT_SEED_SIZE;
    use core::simd::Simd;
    use rand::rngs::SysRng;

    const SMALLEST_DISTINCT_POSITIVE_DESCENDING: Simd64 = Simd::from_array([7, 5, 3, 1]);
    const LARGEST_DISTINCT: Simd64 =
        Simd::from_array([u64::MAX - 6, u64::MAX - 4, u64::MAX - 2, u64::MAX]);
    let rng1 = TripleMixPrng::<R>::almost_all_zeroes_state();
    let rng2 = TripleMixPrng::from_core(TripleMixSimdCore::<R> {
        pcg_state_lo: Simd::splat(0),
        pcg_state_hi: Simd::splat(0),
        pcg_inc_lo: SMALLEST_DISTINCT_POSITIVE_DESCENDING,
        pcg_inc_hi: Simd::splat(0),
        tm0: Simd::splat(0),
        tm1: SMALLEST_DISTINCT_POSITIVE_DESCENDING,
        mwc_state: Simd::splat(0),
        mwc_carry: SMALLEST_DISTINCT_POSITIVE_DESCENDING,
        xoshiro256: [0, 0, 0, 1],
        reproducibility: PhantomData,
    });
    let rng3 = TripleMixPrng::from_core(TripleMixSimdCore::<R> {
        pcg_state_lo: Simd::splat(u64::MAX),
        pcg_state_hi: Simd::splat(u64::MAX),
        pcg_inc_lo: LARGEST_DISTINCT,
        pcg_inc_hi: Simd::splat(u64::MAX),
        tm0: Simd::splat(u64::MAX),
        tm1: LARGEST_DISTINCT,
        mwc_state: TripleMixSimdCore::<R>::MCG_MULTIPLIERS - Simd::splat(2),
        mwc_carry: TripleMixSimdCore::<R>::MCG_MULTIPLIERS - Simd::splat(1),
        xoshiro256: [1, 0, 0, 0],
        reproducibility: PhantomData,
    });
    let mut seed = [0u8; DEFAULT_SEED_SIZE];
    let rng4 = TripleMixPrng::from(&seed);
    SysRng.try_fill_bytes(&mut seed).unwrap();
    let rng5 = TripleMixPrng::from(&seed);
    [rng1, rng2, rng3, rng4, rng5]
}

const MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
const MINOR_VERSION: &str = env!("CARGO_PKG_VERSION_MINOR");
pub const BLOCK_SIZE: usize = MIX_OUTPUTS * SIMD_WIDTH;
