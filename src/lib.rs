#![no_std]
extern crate alloc;

pub mod reproducibility;
pub mod seed;
#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "zeroize")]
mod zeroize;

use core::convert::Infallible;
use core::marker::PhantomData;
use rand_core::block::{BlockRng, Generator};
use rand_core::{Rng, TryRng};
use rand_core::utils::Word;
use reproducibility::Reproducibility;

/// Instances must not be used again after being zeroized.
#[derive(Clone, Debug)]
pub struct FastBlockRng<R: Reproducibility, W: Word, const N: usize, G: Generator<Output = [W; N]>> where G::Output: Clone {
    pub(crate) block_core: BlockRng<G>,
    pub(crate) reproducibility: PhantomData<(R, G)>,
}

impl<R: Reproducibility, const N: usize, G: Generator<Output = [u64; N]>> TryRng for FastBlockRng<R, u64, N, G> {
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

impl<R: Reproducibility, const N: usize, G: Generator<Output = [u32; N]>> TryRng for FastBlockRng<R, u32, N, G> {
    type Error = Infallible;

    #[inline(always)]
    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok(self.block_core.next_word())
    }

    #[inline(always)]
    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        Ok((self.next_u32() as u64) | ((self.next_u32() as u64) << 32))
    }

    #[inline(always)]
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        R::fill_bytes(&mut self.block_core, dst);
        Ok(())
    }
}

#[cfg(test)]
pub type TestFastBlockRng<R: Reproducibility> = FastBlockRng<R, u32, 64, chacha20::ChaChaCore<chacha20::R12, chacha20::variants::Ietf>>;