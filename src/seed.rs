use crate::FastBlockRng;
use crate::Reproducibility;
use core::marker::PhantomData;
use rand_core::block::{BlockRng, Generator};
use rand_core::SeedableRng;
use rand_core::utils::Word;

impl<R: Reproducibility, W: Word, const N: usize, G: Generator<Output = [W; N]>> FastBlockRng<R, W, N, G> {
    #[inline(always)]
    pub(crate) fn from_core(core: G) -> Self {
        Self {
            block_core: BlockRng::new(core),
            reproducibility: PhantomData,
        }
    }
}

impl<R: Reproducibility, W: Word, const N: usize, G: Generator<Output = [W; N]> + SeedableRng> SeedableRng for FastBlockRng<R, W, N, G> {
    type Seed = G::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        Self::from_core(G::from_seed(seed))
    }
}