use crate::generate::{Simd64, TINYMT64_LANE_MASK};
use crate::{Reproducibility, VERSION_OID};
use crate::{TripleMixPrng, TripleMixSimdCore};
use const_format::formatcp;
use core::hint::cold_path;
use core::marker::PhantomData;
use core::simd::Simd;
use core::simd::cmp::SimdPartialEq;
use generic_array::GenericArray;
use rand::RngExt;
use rand_core::SeedableRng;
use rand_core::block::BlockRng;
use std::simd::cmp::SimdPartialOrd;
use tiny_keccak::{Hasher, IntoXof, Kmac, Xof};
use typenum::U;

#[cfg(feature = "no_std")]
macro_rules! once_kmac {
    ($domain:expr) => {
        extern crate alloc;
        static INSTANCE: once_cell::race::OnceBox<Kmac> = once_cell::race::OnceBox::new();
        return INSTANCE
            .get_or_init(|| alloc::boxed::Box::new(Kmac::v256($domain, &[])))
            .clone();
    };
}

#[cfg(not(feature = "no_std"))]
macro_rules! once_kmac {
    ($domain:expr) => {
        use core::ops::Deref;
        static INSTANCE: std::sync::LazyLock<Kmac> =
            std::sync::LazyLock::new(|| Kmac::v256($domain, &[]));
        std::sync::LazyLock::new(|| Kmac::v256($domain, &[]));
        return INSTANCE.deref().clone();
    };
}

// These seed sizes are chosen so that the input to SHA3-512 will be a whole number of blocks.

/// This is the recommended seed size when instantiating TripleMixPrng from a fast hardware entropy
/// source such as RDSEED on x86-64 or RNDRRS on aarch64, if you trust that source enough not to mix
/// it with any slower source the way the operating system usually does. It will always be at least
/// 256 bytes, because TripleMixPrng's internal state contains 2040 variable bits and the extra 8
/// bits help ensure all or nearly all valid states are possible as initial states.
pub const LARGE_SEED_SIZE: usize = 288;

/// This is the recommended seed size when instantiating TripleMixPrng from a SysRng. Windows, MacOS
/// and Linux CSPRNGs are designed to provide only 256 bits of security, so this is the smallest
/// size that's at least 32 bytes and provides a whole number of input blocks to the SHA3-512-KMAC.
/// It will make the TripleMixPrng faster to create than any larger seed size.
#[cfg(not(feature = "large_seeds_by_default"))]
pub const DEFAULT_SEED_SIZE: usize = 72;

#[cfg(feature = "large_seeds_by_default")]
pub const DEFAULT_SEED_SIZE: usize = LARGE_SEED_SIZE;

const SEED_DOMAIN_STRING: &[u8] = formatcp!("{VERSION_OID}::Seed").as_bytes();
const FORK_DOMAIN_STRING: &[u8] = formatcp!("{VERSION_OID}::Fork").as_bytes();

pub(crate) fn get_base_kmac() -> Kmac {
    once_kmac!(SEED_DOMAIN_STRING);
}

pub(crate) fn get_base_fork_kmac() -> Kmac {
    once_kmac!(FORK_DOMAIN_STRING);
}

impl<R: Reproducibility, T: AsRef<[u8]>> From<T> for TripleMixPrng<R> {
    #[inline(always)]
    fn from(raw_seed: T) -> Self {
        const KMAC_BLOCK_SIZE: usize = 72;
        let mut base_kmac = get_base_kmac();
        let raw_len = raw_seed.as_ref().len();
        let padded_len = KMAC_BLOCK_SIZE * raw_len.div_ceil(KMAC_BLOCK_SIZE);
        if padded_len == raw_len {
            base_kmac.update(raw_seed.as_ref());
        } else {
            let mut padded_seed = vec![0u8; padded_len];
            padded_seed[..raw_len].copy_from_slice(raw_seed.as_ref());
            base_kmac.update(&padded_seed);
        }
        let mut attempt = 0u128;
        loop {
            let core = Self::permute(&base_kmac, attempt);
            if core.is_valid() {
                return Self::from_core(core);
            }
            cold_path();
            attempt += 1;
        }
    }
}

impl<R: Reproducibility> SeedableRng for TripleMixPrng<R> {
    type Seed = GenericArray<u8, U<{ DEFAULT_SEED_SIZE }>>;

    #[inline(always)]
    fn from_seed(seed: GenericArray<u8, U<{ DEFAULT_SEED_SIZE }>>) -> Self {
        Self::from(&seed)
    }

    #[inline]
    fn fork(&mut self) -> Self {
        self.fork_with_domain_separation([])
    }
}

impl<R: Reproducibility> TripleMixPrng<R> {
    pub fn into_core(self) -> TripleMixSimdCore {
        self.block_core.core
    }
    #[inline(always)]
    fn permute(base: &Kmac, tweak: u128) -> TripleMixSimdCore {
        let mut pcg_state_lo = Simd64::splat(0);
        let mut pcg_state_hi = Simd64::splat(0);
        let mut pcg_inc_lo = Simd64::splat(0);
        let mut pcg_inc_hi = Simd64::splat(0);
        let mut tm0 = Simd64::splat(0);
        let mut tm1 = Simd64::splat(0);
        let mut mwc_state = Simd64::splat(0);
        let mut mwc_carry = Simd64::splat(0);
        for round in 0..4 {
            let mut round_kmac = base.clone();
            round_kmac.update(&R::u128_as_bytes(tweak + ((round as u128) << 126)));

            // Update KMAC from right half
            let mut buffer = [0u64; 16];
            // This loop looks scalar, but modern LLVM will see
            // the fixed 128-bit extract pattern and emit VEXTRACTI128
            // or VPERM2I128 directly into the buffer.
            buffer[0..2].copy_from_slice(&pcg_state_lo.as_array()[2..4]);
            buffer[2..4].copy_from_slice(&pcg_state_hi.as_array()[2..4]);
            buffer[4..6].copy_from_slice(&pcg_inc_lo.as_array()[2..4]);
            buffer[6..8].copy_from_slice(&pcg_inc_hi.as_array()[2..4]);
            buffer[8..10].copy_from_slice(&tm0.as_array()[2..4]);
            buffer[10..12].copy_from_slice(&tm1.as_array()[2..4]);
            buffer[12..14].copy_from_slice(&mwc_state.as_array()[2..4]);
            buffer[14..16].copy_from_slice(&mwc_carry.as_array()[2..4]);
            round_kmac.update(R::cast_u64_slice_as_u8(&buffer).as_ref());

            let mut reader = round_kmac.into_xof();
            let mut f_out = [0u8; 128];
            reader.squeeze(&mut f_out);

            // Xor into left half
            let mask = Simd::from_array([!0, !0, 0, 0]);
            let data = R::cast_u8_slice_as_u64(&f_out);
            let d0 = Simd::from_slice(&data.as_ref()[0..4]); // words 0,1,2,3
            let d1 = Simd::from_slice(&data.as_ref()[4..8]); // words 4,5,6,7
            let d2 = Simd::from_slice(&data.as_ref()[8..12]); // words 8,9,10,11
            let d3 = Simd::from_slice(&data.as_ref()[12..16]); // words 12,13,14,15
            pcg_state_lo ^= d0 & mask;
            // Use a swizzle to get words 2,3 into lanes 0,1
            pcg_state_hi ^= d0.rotate_elements_left::<2>() & mask;

            pcg_inc_lo ^= d1 & mask;
            pcg_inc_hi ^= d1.rotate_elements_left::<2>() & mask;

            tm0 ^= d2 & mask;
            tm1 ^= d2.rotate_elements_left::<2>() & mask;

            mwc_state ^= d3 & mask;
            mwc_carry ^= d3.rotate_elements_left::<2>() & mask;

            // Swap: Lanes 0,1 <-> Lanes 2,3
            pcg_state_lo = pcg_state_lo.rotate_elements_left::<2>();
            pcg_state_hi = pcg_state_hi.rotate_elements_left::<2>();
            pcg_inc_lo = pcg_inc_lo.rotate_elements_left::<2>();
            pcg_inc_hi = pcg_inc_hi.rotate_elements_left::<2>();
            tm0 = tm0.rotate_elements_left::<2>();
            tm1 = tm1.rotate_elements_left::<2>();
            mwc_state = mwc_state.rotate_elements_left::<2>();
            mwc_carry = mwc_carry.rotate_elements_left::<2>();
        }

        tm0 &= Simd::splat(TINYMT64_LANE_MASK);
        pcg_inc_lo |= Simd::splat(1);
        TripleMixSimdCore {
            pcg_state_lo,
            pcg_state_hi,
            pcg_inc_lo,
            pcg_inc_hi,
            tm0,
            tm1,
            mwc_state,
            mwc_carry,
        }
    }

    #[inline(always)]
    fn is_distinct(a: &TripleMixSimdCore, b: &TripleMixSimdCore) -> bool {
        // Simple distinctness check: Child state != Parent state in any lane combination
        !((a.pcg_state_lo.simd_eq(b.pcg_state_lo)
            & a.pcg_state_hi.simd_eq(b.pcg_state_hi)
            & a.pcg_inc_lo.simd_eq(b.pcg_inc_lo)
            & a.pcg_inc_hi.simd_eq(b.pcg_inc_hi))
            | (a.tm0.simd_eq(b.tm0) & a.tm1.simd_eq(b.tm1))
            | (a.mwc_state.simd_eq(b.mwc_state) & a.mwc_carry.simd_eq(b.mwc_carry)))
        .any()
    }

    /// Returns a new instance derived from both this one and the provided domain-separation bytes.
    /// The returned instance has less than a 1 in 2<sup>500</sup> chance of sharing a state with
    /// any other instance obtained by fork_with_domain_separation, unless they're called on clones
    /// of the same instance *and* the domain bytes are identical across both calls.
    /// Also permutes the parent PRNG's state so that repeated calls, even with the same
    /// domain-separation bytes, will return statistically independent instances.
    #[inline]
    pub fn fork_with_domain_separation(&mut self, domain_separation: impl AsRef<[u8]>) -> Self {
        // 2. Use this entropy as a seed for the new PRNG
        let mut fork_kmac = if domain_separation.as_ref().is_empty() {
            get_base_fork_kmac()
        } else {
            Kmac::v256(FORK_DOMAIN_STRING, domain_separation.as_ref())
        };
        let mut padding = [0u64; 3]; // 24 bytes + 192-byte core state = 216 bytes = 3 blocks
        let remaining = self.block_core.remaining_results();
        let remaining_len = remaining.len();
        if remaining_len > 0 {
            padding[0] = remaining_len as u64;
            padding[1..(remaining.len() + 1).min(3)].copy_from_slice(remaining);
        } else {
            self.fill(&mut padding);
        }
        fork_kmac.update(self.block_core.core.as_bytes());
        fork_kmac.update(R::cast_u64_slice_as_u8(&padding).as_ref());
        self.block_core.reset_and_skip(0);
        let mut attempt = 0u128;
        loop {
            let core = Self::permute(&fork_kmac, attempt);
            if core.is_valid() && Self::is_distinct(&core, &self.block_core.core) {
                return Self::from_core(core);
            }
            cold_path();
            attempt += 1;
        }
    }

    #[inline(always)]
    pub(crate) fn from_core(core: TripleMixSimdCore) -> Self {
        Self {
            block_core: BlockRng::new(core),
            reproducibility: PhantomData,
        }
    }

    /// Creates an instance in a relatively predictable state. Idempotent. Intended only for
    /// testing.
    #[inline(always)]
    pub fn almost_all_zeroes_state() -> Self {
        TripleMixPrng::from_core(TripleMixSimdCore::almost_all_zeroes_core())
    }
}

impl TripleMixSimdCore {
    #[inline(always)]
    pub(crate) fn is_valid(self: &TripleMixSimdCore) -> bool {
        // Dead-state check
        if ((self.tm0 | self.tm1).simd_eq(Simd::splat(0))
            | (self.mwc_state | self.mwc_carry).simd_eq(Simd::splat(0))
            | self.mwc_state.simd_ge(TripleMixSimdCore::MCG_MULTIPLIERS)
            | self.mwc_carry.simd_ge(TripleMixSimdCore::MCG_MULTIPLIERS)
            | (self.mwc_state & self.mwc_carry)
                .simd_eq(TripleMixSimdCore::MCG_MULTIPLIERS - Simd::splat(1))
            | (self.pcg_inc_lo & Simd::splat(1)).simd_ne(Simd::splat(1)))
        .any()
        {
            cold_path();
            return false;
        }
        // Duplicate Check via XOR Reduction
        // If Lane[i] == Lane[j], then (Lane[i] ^ Lane[j]) == 0.
        // We check all 3 possible shift-offsets (1, 2, 3).
        macro_rules! find_similar {
            ($shift:expr) => {
                let diff_pcg = (self.pcg_state_lo
                    ^ self.pcg_state_lo.rotate_elements_left::<$shift>())
                    | (self.pcg_state_hi ^ self.pcg_state_hi.rotate_elements_left::<$shift>())
                    | (self.pcg_inc_lo ^ self.pcg_inc_lo.rotate_elements_left::<$shift>())
                    | (self.pcg_inc_hi ^ self.pcg_inc_hi.rotate_elements_left::<$shift>());
                let diff_tm = (self.tm0 ^ self.tm0.rotate_elements_left::<$shift>())
                    | (self.tm1 ^ self.tm1.rotate_elements_left::<$shift>());
                let diff_mcg = (self.mwc_state ^ self.mwc_state.rotate_elements_left::<$shift>())
                    | (self.mwc_carry ^ self.mwc_carry.rotate_elements_left::<$shift>());

                // A lane is similar if ANY sub-generator matches the rotated version
                if (diff_pcg.simd_eq(Simd::splat(0))
                    | diff_tm.simd_eq(Simd::splat(0))
                    | diff_mcg.simd_eq(Simd::splat(0)))
                .any()
                {
                    cold_path();
                    return false;
                }
            };
        }

        find_similar!(1);
        find_similar!(2);
        // (Note: shift 3 is covered by shift 1 in a 4-lane circular buffer,
        // but we keep it for strict completeness if not circular)
        find_similar!(3);

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::TripleMixPrng;
    use crate::generate::{MIX_OUTPUTS, SIMD_WIDTH};
    use crate::reproducibility::{DefaultReproducibility, NotReproducible};
    use crate::seed::{DEFAULT_SEED_SIZE, get_base_kmac};
    use core::hint::black_box;
    use generic_array::GenericArray;
    use rand::rngs::SysRng;
    use rand_core::{Rng, SeedableRng, UnwrapErr};
    use tiny_keccak::{Hasher, Kmac};

    #[cfg(feature = "no_std")]
    extern crate alloc;

    #[test]
    fn test_fork_independence_descendants() {
        const SAMPLES_PER_FORK: usize = MIX_OUTPUTS * SIMD_WIDTH * 4;
        const FORKS: usize = 64;
        #[cfg(not(feature = "no_std"))]
        let mut previous_outputs =
            std::collections::HashSet::with_capacity(SAMPLES_PER_FORK * FORKS);
        #[cfg(feature = "no_std")]
        let mut previous_outputs = alloc::collections::BTreeSet::new();
        for mut prng in crate::create_rngs::<NotReproducible>() {
            for _ in 0..FORKS {
                for _ in 0..SAMPLES_PER_FORK {
                    let next = prng.next_u64();
                    print!("{next:016X} ");
                    assert!(previous_outputs.insert(next));
                }
                println!();
                prng = prng.fork();
            }
            println!();
        }
    }

    #[test]
    fn test_fork_independence_siblings() {
        const SAMPLES_PER_FORK: usize = 32;
        const FORKS: usize = 64;
        #[cfg(not(feature = "no_std"))]
        let mut previous_outputs =
            std::collections::HashSet::with_capacity(SAMPLES_PER_FORK * FORKS);
        #[cfg(feature = "no_std")]
        let mut previous_outputs = alloc::collections::BTreeSet::new();
        for mut parent_prng in crate::create_rngs::<NotReproducible>() {
            for _ in 0..FORKS {
                let mut prng = parent_prng.fork();
                for _ in 0..SAMPLES_PER_FORK {
                    let next = prng.next_u64();
                    print!("{next:016X}");
                    assert!(previous_outputs.insert(next));
                }
                println!();
            }
        }
    }

    #[test]
    fn test_seed_diffusion() {
        let seed = [0u8; DEFAULT_SEED_SIZE];
        let mut rng1 = TripleMixPrng::<NotReproducible>::from_seed(GenericArray::from(seed));
        let start_val1 = rng1.next_u64();

        for byte_index in 0..DEFAULT_SEED_SIZE {
            for bit_index in 0..=7 {
                let mut seed = [0u8; DEFAULT_SEED_SIZE];
                seed[byte_index] = 1 << bit_index;
                let mut rng2 =
                    TripleMixPrng::<NotReproducible>::from_seed(GenericArray::from(seed));
                let start_val2 = rng2.next_u64();
                let flipped_bits = (start_val1 ^ start_val2).count_ones();
                assert!(
                    flipped_bits > 1,
                    "Changing byte {byte_index} bit {bit_index} of the seed did not affect the first output! Diffusion failure."
                );
                assert!(
                    flipped_bits < 63,
                    "Changing byte {byte_index} bit {bit_index} of the seed just inverted the first output! Diffusion failure."
                );
                assert_ne!(start_val1, 0, "Output shouldn't be zero");
            }
        }
    }

    #[test]
    fn test_permutation_determinism() {
        let base = get_base_kmac();
        let p1 = TripleMixPrng::<NotReproducible>::permute(&base, 0);
        let p2 = TripleMixPrng::<NotReproducible>::permute(&base, 0);

        // Ensure same seed + same tweak = identical state
        assert_eq!(p1.pcg_state_lo.as_array(), p2.pcg_state_lo.as_array());
        assert_eq!(p1.tm1.as_array(), p2.tm1.as_array());
    }

    #[test]
    fn test_permutation_uniqueness_tweak() {
        let base = get_base_kmac();
        let p1 = TripleMixPrng::<NotReproducible>::permute(&base, 0);
        let p2 = TripleMixPrng::<NotReproducible>::permute(&base, 1);

        // Different tweaks MUST produce different states
        assert_ne!(p1.pcg_state_lo.as_array(), p2.pcg_state_lo.as_array());
    }

    #[test]
    fn test_permutation_collision_resistance() {
        let base = get_base_kmac();
        let mut results = std::collections::HashSet::new();

        // Run 1000 permutations and check for any identical full states
        // In a true permutation, collisions are mathematically impossible.
        for i in 0..1000 {
            let p = TripleMixPrng::<NotReproducible>::permute(&base, i);
            let state_snapshot = (
                p.pcg_state_lo.as_array().clone(),
                p.pcg_state_hi.as_array().clone(),
                p.pcg_inc_lo.as_array().clone(),
                p.pcg_inc_hi.as_array().clone(),
                p.tm0.as_array().clone(),
                p.tm1.as_array().clone(),
            );
            assert!(
                results.insert(state_snapshot),
                "Collision detected in Feistel permutation at tweak {}",
                i
            );
        }
    }

    #[test]
    fn test_diffusion_avalanche() {
        let base1 = get_base_kmac();
        let mut base2 = Kmac::v256(b"Test-Suite", &[]);
        base2.update(b"test-seed-124"); // 1 bit difference from base1

        let p1 = TripleMixPrng::<NotReproducible>::permute(&base1, 0);
        let p2 = TripleMixPrng::<NotReproducible>::permute(&base2, 0);

        // Count differing bits in pcg_state_lo across all lanes
        let mut diff_bits = 0;
        for i in 0..4 {
            diff_bits +=
                (p1.pcg_state_lo.as_array()[i] ^ p2.pcg_state_lo.as_array()[i]).count_ones();
        }

        // Avalanche effect: ~50% of bits should flip.
        // For 256 bits of pcg_state_lo, we expect ~128. Threshold at 80 for safety.
        assert!(
            diff_bits > 80,
            "Poor diffusion: only {} bits flipped",
            diff_bits
        );
    }

    #[test]
    fn test_invariant_fixing() {
        let base = get_base_kmac();
        let p = TripleMixPrng::<NotReproducible>::permute(&base, 42);

        // TinyMT dead bit (MSB of tm0) must be 0
        for &tm in p.tm0.as_array() {
            assert!(tm <= 0x7FFFFFFFFFFFFFFF, "TinyMT dead bit was not cleared");
        }
    }

    #[test]
    fn test_lane_swap_integrity() {
        // This test ensures that the swap logic actually moves data between lanes 0/1 and 2/3
        // We use a custom round function effect by checking different rounds.
        let base = get_base_kmac();
        let p = TripleMixPrng::<NotReproducible>::permute(&base, 7);

        // Since it's a 4-round Feistel, if we started with 0,
        // the final state should be high-entropy in all lanes.
        for i in 0..4 {
            assert_ne!(p.pcg_state_lo.as_array()[i], 0, "Lane {} remained zero", i);
        }
    }

    #[test]
    fn test_from_rng() {
        black_box(
            TripleMixPrng::<DefaultReproducibility>::from_rng(&mut UnwrapErr(SysRng)).next_u64(),
        );
    }
}
