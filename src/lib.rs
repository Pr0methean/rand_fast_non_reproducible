#![feature(portable_simd)]
#![feature(generic_const_exprs)]
#![allow(long_running_const_eval)]
#![allow(incomplete_features)]
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

use crate::generate::{MIX_OUTPUTS, SIMD_WIDTH, Simd32};
use crate::reproducibility::{DefaultReproducibility, NotReproducible};
use const_format::formatcp;
use core::convert::Infallible;
use core::marker::PhantomData;
use core::simd::{Simd, ToBytes, simd_swizzle};
use generate::Simd64;
use rand_core::TryRng;
use rand_core::block::BlockRng;
use reproducibility::Reproducibility;
use crate::seed::LARGE_SEED_SIZE;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TripleMixSimdCore<R: Reproducibility> {
    pub(crate) tm0: Simd64, // TinyMT64 state
    pub(crate) tm1: Simd64, // TinyMT64 state, with highest bit always 0
    pub(crate) mwc_state: Simd64,
    pub(crate) mwc_carry: Simd64,
    pub(crate) pcg_state_lo: Simd64,
    pub(crate) pcg_state_hi: Simd64,
    pub(crate) pcg_inc_lo: Simd64,
    pub(crate) pcg_inc_hi: Simd64,
    pub(crate) xoshiro256: [u64; 4],
    pub(crate) scalar_weyl: u64,
    pub(crate) reproducibility: PhantomData<R>,
}

impl<R: Reproducibility> core::fmt::Debug for TripleMixSimdCore<R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

impl<R: Reproducibility> TripleMixSimdCore<R> {
    #[allow(unused)]
    #[inline(always)]
    fn portable_mul_lo_hi(a: Simd32, b: Simd32) -> (Simd32, Simd32) {
        let a64: Simd64 = R::simd32_as_simd64(a);
        let b64: Simd64 = R::simd32_as_simd64(b);
        let mask32 = Simd64::splat(0xFFFF_FFFF);
        // Even lanes (lower 32 bits of each 64-bit word)
        let even = (a64 & mask32) * (b64 & mask32);

        // Odd lanes
        let a_hi = a64 >> Simd::splat(32);
        let b_hi = b64 >> Simd::splat(32);
        let odd = a_hi * b_hi;

        // Reinterpret to u32
        let even32: Simd32 = R::simd64_as_simd32(even);
        let odd32: Simd32 = R::simd64_as_simd32(odd);

        let lo = simd_swizzle!(even32, odd32, [0, 8, 2, 10, 4, 12, 6, 14]);
        let hi = simd_swizzle!(even32, odd32, [1, 9, 3, 11, 5, 13, 7, 15]);

        (lo, hi)
    }

    #[inline(always)]
    pub(crate) fn mul_lo_hi_triad(
        a: Simd32,
        b: Simd32,
        c: Simd32,
    ) -> (Simd32, Simd32, Simd32, Simd32) {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
        ))]
        {
            use bytemuck::cast;
            let (ab_lo, ab_hi, bc_lo, bc_hi) =
                unsafe { avx2::mul_lo_hi_triad_avx2(cast(a), cast(b), cast(c)) };
            (cast(ab_lo), cast(ab_hi), cast(bc_lo), cast(bc_hi))
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "avx2",
            not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
        )))]
        {
            let (ab_lo, ab_hi) = Self::portable_mul_lo_hi(a, b);
            let (bc_lo, bc_hi) = Self::portable_mul_lo_hi(b, c);
            (ab_lo, ab_hi, bc_lo, bc_hi)
        }
    }

    #[inline(always)]
    pub fn copy_to_le_bytes(&self, dst: &mut [u8]) {
        dst[0..32].copy_from_slice(self.tm0.to_le_bytes().as_array());
        dst[32..64].copy_from_slice(self.tm1.to_le_bytes().as_array());
        dst[64..96].copy_from_slice(self.mwc_state.to_le_bytes().as_array());
        dst[96..128].copy_from_slice(self.mwc_carry.to_le_bytes().as_array());
        dst[128..160].copy_from_slice(self.pcg_state_lo.to_le_bytes().as_array());
        dst[160..192].copy_from_slice(self.pcg_state_hi.to_le_bytes().as_array());
        dst[192..224].copy_from_slice(self.pcg_inc_lo.to_le_bytes().as_array());
        dst[224..256].copy_from_slice(self.pcg_inc_hi.to_le_bytes().as_array());
        let mut chunks = dst[256..288].chunks_exact_mut(8);
        for &word in &self.xoshiro256 {
            chunks.next().unwrap().copy_from_slice(&word.to_le_bytes());
        }
    }
}

/// Instances must not be used again after being zeroized.
#[derive(Clone, Debug)]
pub struct TripleMixPrng<R: Reproducibility = DefaultReproducibility> {
    pub(crate) block_core: BlockRng<TripleMixSimdCore<R>>,
    pub(crate) reproducibility: PhantomData<R>,
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
pub(crate) fn create_rngs<R: Reproducibility>() -> Vec<TripleMixPrng<R>> {
    let mut rngs = Vec::new();
    rngs.push(TripleMixPrng::<R>::almost_all_zeroes_state());
    #[cfg(not(miri))]
    {
        use crate::seed::DEFAULT_SEED_SIZE;
        use core::simd::Simd;
        use rand::rngs::SysRng;

        const SMALLEST_DISTINCT_POSITIVE_DESCENDING: Simd64 = Simd::from_array([7, 5, 3, 1]);
        const LARGEST_DISTINCT: Simd64 =
            Simd::from_array([u64::MAX - 6, u64::MAX - 4, u64::MAX - 2, u64::MAX]);
        rngs.push(TripleMixPrng::from_core(TripleMixSimdCore::<R> {
            pcg_state_lo: Simd::splat(0),
            pcg_state_hi: Simd::splat(0),
            pcg_inc_lo: SMALLEST_DISTINCT_POSITIVE_DESCENDING,
            pcg_inc_hi: Simd::splat(0),
            tm0: Simd::splat(0),
            tm1: SMALLEST_DISTINCT_POSITIVE_DESCENDING,
            mwc_state: Simd::splat(0),
            mwc_carry: SMALLEST_DISTINCT_POSITIVE_DESCENDING,
            xoshiro256: [0, 0, 0, 1],
            scalar_weyl: 0,
            reproducibility: PhantomData,
        }));
        rngs.push(TripleMixPrng::from_core(TripleMixSimdCore::<R> {
            pcg_state_lo: Simd::splat(u64::MAX),
            pcg_state_hi: Simd::splat(u64::MAX),
            pcg_inc_lo: LARGEST_DISTINCT,
            pcg_inc_hi: Simd::splat(u64::MAX),
            tm0: Simd::splat(u64::MAX),
            tm1: LARGEST_DISTINCT,
            mwc_state: TripleMixSimdCore::<R>::MCG_MULTIPLIERS - Simd::splat(2),
            mwc_carry: TripleMixSimdCore::<R>::MCG_MULTIPLIERS - Simd::splat(1),
            xoshiro256: [1, 0, 0, 0],
            scalar_weyl: 0,
            reproducibility: PhantomData,
        }));
        let mut seed = [0u8; DEFAULT_SEED_SIZE];
        rngs.push(TripleMixPrng::from(&seed));
        SysRng.try_fill_bytes(&mut seed).unwrap();
        rngs.push(TripleMixPrng::from(&seed));
        let mut large_seed = [0u8; LARGE_SEED_SIZE];
        rngs.push(TripleMixPrng::from(&large_seed));
        SysRng.try_fill_bytes(&mut large_seed).unwrap();
        rngs.push(TripleMixPrng::from(&large_seed));
    }
    rngs
}

#[cfg(all(test, not(miri)))]
pub(crate) fn rng() -> rand::rngs::ThreadRng {
    rand::rng()
}

#[cfg(all(test, miri))]
pub(crate) fn rng() -> rand::rngs::SmallRng {
    use rand::SeedableRng;
    rand::rngs::SmallRng::seed_from_u64(0x0dd_d00d5_1337_c0de)
}

const MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
const MINOR_VERSION: &str = env!("CARGO_PKG_VERSION_MINOR");
pub const BLOCK_SIZE: usize = MIX_OUTPUTS * SIMD_WIDTH;
