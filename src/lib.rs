#![feature(portable_simd)]
#![allow(long_running_const_eval)]
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

use crate::reproducibility::DefaultReproducibility;
use const_format::formatcp;
use core::convert::Infallible;
use core::marker::PhantomData;
use generate::Simd64;
use rand_core::TryRng;
use rand_core::block::BlockRng;
use reproducibility::Reproducibility;
use crate::generate::{OUTPUTS_PER_STEP, SIMD_WIDTH};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TripleMixSimdCore {
    tm0: Simd64, // TinyMT64 state
    tm1: Simd64, // TinyMT64 state, with highest bit always 0
    mwc_state: Simd64,
    mwc_carry: Simd64,
    pcg_state_lo: Simd64,
    pcg_state_hi: Simd64,
    pcg_inc_lo: Simd64,
    pcg_inc_hi: Simd64,
}

impl std::fmt::Debug for TripleMixSimdCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x2 = self.mwc_carry;
        let x3 = self.mwc_state;
        let x4 = self.tm1;
        let x5 = self.tm0;
        let x6 = self.pcg_state_hi;
        let x7 = self.pcg_state_lo;
        let x8 = self.pcg_inc_hi;
        let x9 = self.pcg_inc_lo;
        f.debug_struct("TripleMixSimdCore")
            .field("pcg_state_lo", &x7.to_array())
            .field("pcg_state_hi", &x6.to_array())
            .field("pcg_inc_lo", &x9.to_array())
            .field("pcg_inc_hi", &x8.to_array())
            .field("tm0", &x5.to_array())
            .field("tm1", &x4.to_array())
            .field("mwc_state", &x3.to_array())
            .field("mwc_carry", &x2.to_array())
            .finish()
    }
}

impl TripleMixSimdCore {
    #[inline(always)]
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts((self as *const Self) as *const u8, size_of::<Self>()) }
    }
}

/// Instances must not be used again after being zeroized.
#[derive(Clone, Debug)]
pub struct TripleMixPrng<R: Reproducibility = DefaultReproducibility> {
    block_core: BlockRng<TripleMixSimdCore>,
    reproducibility: PhantomData<R>,
}

pub const TRIPLE_MIX_PRNG_OID: &str = "1.3.6.1.4.1.54392.5.3311";
pub const VERSION_OID: &str = formatcp!("{TRIPLE_MIX_PRNG_OID}.{MAJOR_VERSION}.{MINOR_VERSION}");

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
    let rng1 = TripleMixPrng::almost_all_zeroes_state();
    let rng2 = TripleMixPrng::from_core(TripleMixSimdCore {
        pcg_state_lo: Simd::splat(0),
        pcg_state_hi: Simd::splat(0),
        pcg_inc_lo: SMALLEST_DISTINCT_POSITIVE_DESCENDING,
        pcg_inc_hi: Simd::splat(0),
        tm0: Simd::splat(0),
        tm1: SMALLEST_DISTINCT_POSITIVE_DESCENDING,
        mwc_state: Simd::splat(0),
        mwc_carry: SMALLEST_DISTINCT_POSITIVE_DESCENDING,
    });
    let rng3 = TripleMixPrng::from_core(TripleMixSimdCore {
        pcg_state_lo: Simd::splat(u64::MAX),
        pcg_state_hi: Simd::splat(u64::MAX),
        pcg_inc_lo: LARGEST_DISTINCT,
        pcg_inc_hi: Simd::splat(u64::MAX),
        tm0: Simd::splat(u64::MAX),
        tm1: LARGEST_DISTINCT,
        mwc_state: TripleMixSimdCore::MCG_MULTIPLIERS - Simd::splat(2),
        mwc_carry: TripleMixSimdCore::MCG_MULTIPLIERS - Simd::splat(1),
    });
    let mut seed = [0u8; DEFAULT_SEED_SIZE];
    let rng4 = TripleMixPrng::from(&seed);
    SysRng.try_fill_bytes(&mut seed).unwrap();
    let rng5 = TripleMixPrng::from(&seed);
    [rng1, rng2, rng3, rng4, rng5]
}

const MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
const MINOR_VERSION: &str = env!("CARGO_PKG_VERSION_MINOR");
pub const BLOCK_SIZE: usize = OUTPUTS_PER_STEP * SIMD_WIDTH;