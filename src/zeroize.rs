use rand_core::block::Generator;
use crate::reproducibility::Reproducibility;
use crate::{FastBlockRng};
use rand_core::utils::Word;
use zeroize::Zeroize;

impl<R: Reproducibility, W: Word, const N: usize, G: Zeroize + Generator<Output = [W; N]>> zeroize::Zeroize for FastBlockRng<R, W, N, G> {
    fn zeroize(&mut self) {
        self.block_core.core.zeroize();

        // Force next generation to overwrite the buffer using output derived from the zeroized core
        // (this won't be zero, but it won't reflect the previous core state either)
        self.block_core.reset_and_skip(0);

        // Prevent dead-write elimination
        core::hint::black_box(self.block_core.remaining_results());
    }
}
