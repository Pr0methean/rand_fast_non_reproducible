use crate::reproducibility::Reproducibility;
use crate::{TripleMixPrng, TripleMixSimdCore};
use core::simd::Simd;

impl zeroize::Zeroize for TripleMixSimdCore {
    fn zeroize(&mut self) {
        self.pcg_state_lo = Simd::splat(0);
        self.pcg_state_hi = Simd::splat(0);
        self.pcg_inc_lo = Simd::splat(0);
        self.pcg_inc_hi = Simd::splat(0);
        self.tm0 = Simd::splat(0);
        self.tm1 = Simd::splat(0);
        self.mwc_state = Simd::splat(0);
        self.mwc_carry = Simd::splat(0);
        self.xoshiro256 = [0; 4];
        // Prevent dead-write elimination
        core::hint::black_box(&*self);
    }
}

impl<R: Reproducibility> zeroize::Zeroize for TripleMixPrng<R> {
    fn zeroize(&mut self) {
        self.block_core.core.zeroize();

        // Force next generation to overwrite the buffer using output derived from the zeroized core
        // (this won't be zero, but it won't reflect the previous core state either)
        self.block_core.reset_and_skip(0);

        // Prevent dead-write elimination
        core::hint::black_box(self.block_core.remaining_results());
    }
}

impl<R: Reproducibility> Drop for TripleMixPrng<R> {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use crate::generate::Simd64;
    use crate::reproducibility::DefaultReproducibility;
    use crate::{BLOCK_SIZE, TripleMixPrng, TripleMixSimdCore, create_rngs};
    use rand_core::Rng;
    use zeroize::Zeroize;

    #[test]
    fn test_zeroize() {
        let zero_core = TripleMixSimdCore {
            pcg_state_lo: Simd64::splat(0),
            pcg_state_hi: Simd64::splat(0),
            pcg_inc_lo: Simd64::splat(0),
            pcg_inc_hi: Simd64::splat(0),
            tm0: Simd64::splat(0),
            tm1: Simd64::splat(0),
            mwc_state: Simd64::splat(0),
            mwc_carry: Simd64::splat(0),
            xoshiro256: [0; 4],
        };
        let mut expected_output = [0u8; BLOCK_SIZE * size_of::<u64>() * 2];
        TripleMixPrng::<DefaultReproducibility>::from_core(zero_core)
            .fill_bytes(&mut expected_output);
        for mut prng in create_rngs::<DefaultReproducibility>() {
            let mut output = [0u8; BLOCK_SIZE * size_of::<u64>() * 2];
            prng.next_u64();
            prng.zeroize();
            prng.fill_bytes(&mut output);
            assert_eq!(&output[..], &expected_output[..]);
        }
    }
}
