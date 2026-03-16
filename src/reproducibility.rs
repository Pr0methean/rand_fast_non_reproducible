use crate::TripleMixSimdCore;
use bytemuck::cast_slice;
use rand_core::block::BlockRng;

/// Levels of reproducibility for output of [`TripleMixPrng::fill_bytes`] and output after
/// fill_bytes has been called.
pub trait Reproducibility: Clone + Copy {
    type U8Slice<'a>: AsRef<[u8]>;
    type U64Slice<'a>: AsRef<[u64]>;
    fn fill_bytes(core: &mut BlockRng<TripleMixSimdCore>, bytes: &mut [u8]);
    fn cast_u8_slice_as_u64(slice: &[u8]) -> Self::U64Slice<'_>;
    fn cast_u64_slice_as_u8(slice: &[u64]) -> Self::U8Slice<'_>;
    fn u64_as_bytes(input: u64) -> [u8; 8];
    fn u128_as_bytes(input: u128) -> [u8; 16];
}

#[cfg(feature = "reproducibility_cross_platform")]
pub type DefaultReproducibility = cross_platform::CrossPlatform;

#[cfg(all(
    feature = "reproducibility_same_endianness",
    not(feature = "reproducibility_cross_platform")
))]
pub type DefaultReproducibility = same_endianness::SameEndianness;

#[cfg(not(any(
    feature = "reproducibility_same_endianness",
    feature = "reproducibility_cross_platform"
)))]
pub type DefaultReproducibility = NotReproducible;

/// Output of [`TripleMixPrng::fill_bytes`] and the state of the PRNG afterward may depend on the
/// address alignment where the byte slice starts and ends and the machine endianness.
#[derive(Copy, Clone, Default, Debug)]
pub struct NotReproducible;

impl Reproducibility for NotReproducible {
    type U8Slice<'a> = &'a [u8];
    type U64Slice<'a> = &'a [u64];
    #[inline(always)]
    fn fill_bytes(block_core: &mut BlockRng<TripleMixSimdCore>, bytes: &mut [u8]) {
        let (prefix, u64s, suffix) = unsafe { bytes.align_to_mut::<u64>() };
        if u64s.is_empty() {
            // There's no benefit to bypassing the buffer or consolidating
            // writes if we can't write at least one aligned u64.
            block_core.fill_bytes(bytes);
            return;
        }
        if !prefix.is_empty() {
            block_core.fill_bytes(prefix);
        }
        fill_bytes_inner(block_core, u64s, suffix);
    }

    #[inline(always)]
    fn cast_u8_slice_as_u64(slice: &[u8]) -> &[u64] {
        cast_slice(slice)
    }

    #[inline(always)]
    fn cast_u64_slice_as_u8(slice: &[u64]) -> &[u8] {
        cast_slice(slice)
    }

    #[inline(always)]
    fn u64_as_bytes(input: u64) -> [u8; 8] {
        input.to_ne_bytes()
    }

    #[inline(always)]
    fn u128_as_bytes(input: u128) -> [u8; 16] {
        input.to_ne_bytes()
    }
}

/// Output of the PRNG will be the same as for an instance created with the same seed and receiving
/// the same calls (counting two `fill_bytes` as the same when they write to slices of the same
/// length), as long as both instances are created on machines with the same endianness.
#[cfg(feature = "reproducibility_same_endianness")]
pub mod same_endianness {
    use crate::TripleMixSimdCore;
    use crate::reproducibility::{Reproducibility, fill_bytes_alignment_aware};
    use bytemuck::cast_slice;
    use rand_core::block::BlockRng;

    #[derive(Copy, Clone, Default, Debug)]
    pub struct SameEndianness;

    impl Reproducibility for SameEndianness {
        type U8Slice<'a> = &'a [u8];
        type U64Slice<'a> = &'a [u64];

        fn fill_bytes(core: &mut BlockRng<TripleMixSimdCore>, bytes: &mut [u8]) {
            fill_bytes_alignment_aware(core, bytes);
        }

        #[inline(always)]
        fn cast_u8_slice_as_u64(slice: &[u8]) -> &[u64] {
            cast_slice(slice)
        }

        #[inline(always)]
        fn cast_u64_slice_as_u8(slice: &[u64]) -> &[u8] {
            cast_slice(slice)
        }

        #[inline(always)]
        fn u64_as_bytes(input: u64) -> [u8; 8] {
            input.to_ne_bytes()
        }

        #[inline(always)]
        fn u128_as_bytes(input: u128) -> [u8; 16] {
            input.to_ne_bytes()
        }
    }

    #[test]
    fn test_equivalence_same_endianness() {
        super::test_equivalence_generic::<SameEndianness>();
    }
}

/// Output of the PRNG will be the same as for an instance created with the same seed and receiving
/// the same calls (counting two `fill_bytes` as the same when they write to slices of the same
/// length) on another machine, even if that machine has a different CPU architecture.
#[cfg(feature = "reproducibility_cross_platform")]
pub mod cross_platform {
    use crate::TripleMixSimdCore;
    use crate::reproducibility::{Reproducibility, fill_bytes_alignment_aware};
    use bytemuck::cast_slice;
    use rand_core::block::BlockRng;

    #[derive(Copy, Clone, Default, Debug)]
    pub struct CrossPlatform;

    #[cfg(feature = "reproducibility_cross_platform")]
    impl Reproducibility for CrossPlatform {
        #[cfg(target_endian = "little")]
        type U8Slice<'a> = &'a [u8];
        #[cfg(target_endian = "big")]
        type U8Slice<'a> = Vec<u8>;
        #[cfg(target_endian = "little")]
        type U64Slice<'a> = &'a [u64];
        #[cfg(target_endian = "big")]
        type U64Slice<'a> = Vec<u64>;
        #[cfg(target_endian = "little")]
        #[inline(always)]
        fn fill_bytes(block_core: &mut BlockRng<TripleMixSimdCore>, bytes: &mut [u8]) {
            fill_bytes_alignment_aware(block_core, bytes);
        }
        #[cfg(target_endian = "big")]
        #[inline(always)]
        fn fill_bytes(block_core: &mut BlockRng<TripleMixSimdCore>, bytes: &mut [u8]) {
            block_core.fill_bytes(bytes);
        }

        #[cfg(target_endian = "little")]
        #[inline(always)]
        fn cast_u8_slice_as_u64(slice: &[u8]) -> &[u64] {
            cast_slice(slice)
        }

        #[cfg(target_endian = "big")]
        #[inline(always)]
        fn cast_u8_slice_as_u64(slice: &[u8]) -> Vec<u64> {
            slice
                .chunks_exact(8)
                .map(|chunk| u64::from_le_bytes(*chunk.as_array().unwrap()))
                .collect()
        }

        #[cfg(target_endian = "little")]
        #[inline(always)]
        fn cast_u64_slice_as_u8(slice: &[u64]) -> &[u8] {
            cast_slice(slice)
        }

        #[cfg(target_endian = "big")]
        #[inline(always)]
        fn cast_u64_slice_as_u8(slice: &[u64]) -> Vec<u8> {
            slice.iter().copied().flat_map(u64::to_le_bytes).collect()
        }

        #[inline(always)]
        fn u64_as_bytes(input: u64) -> [u8; 8] {
            input.to_le_bytes()
        }

        #[inline(always)]
        fn u128_as_bytes(input: u128) -> [u8; 16] {
            input.to_le_bytes()
        }
    }

    #[cfg(test)]
    mod tests {
        #[cfg(not(feature = "large_seeds_by_default"))]
        #[test]
        fn test_cross_platform_reproducibility() {
            use super::CrossPlatform;
            use crate::TripleMixPrng;
            use crate::seed::DEFAULT_SEED_SIZE;
            use itertools::Itertools;
            use rand_core::Rng;
            let seed = [0u8; DEFAULT_SEED_SIZE];
            let mut prng = TripleMixPrng::<CrossPlatform>::from(&seed);
            let expected = "7F6DE25C02AFFE443C1BFBA359002CE18457326E5157A0D627C7FE17E667D8447F62CD0AA84B8A06A4C1F955A9A74C49A693F761FF5C9ACEE736E1BCDAAD7C59AF4858AFB695ADB3BBC21D9B0F1A979C6A83E8890B8CF972F657E99A3D79BBE73B061E743D47DDC7B6DD794F71D2F569E30E61DBFA83C66FB7ECCDC1716544C8A6FDE5E8615544440D2DFC84355449902CFF771D0BEB10637FF3DE5F2532224147437154EE783E70D38BEA9FA9DE22677393CB0EE81545CBAC480FFD907F4A3127B7E12D046A49BD2A505969E5B4D3CCBF73B39F7BC16B282A249D06E4E3DFCC729D811B7601464353B2819BD47349ED8835E339F49D4BA2DF185A0E2596EED9";
            let mut actual = vec![0u8; expected.len() / 2];
            prng.fill_bytes(&mut actual);
            assert_eq!(
                &actual.iter().map(|byte| format!("{byte:02X}")).join(""),
                expected
            );
        }

        #[cfg(feature = "reproducibility_cross_platform")]
        #[test]
        fn test_equivalence_cross_platform() {
            super::super::test_equivalence_generic::<super::CrossPlatform>();
        }
    }
}

#[cfg(any(
    feature = "reproducibility_cross_platform",
    feature = "reproducibility_same_endianness"
))]
#[inline(always)]
fn fill_bytes_alignment_aware(block_core: &mut BlockRng<TripleMixSimdCore>, bytes: &mut [u8]) {
    let (prefix, u64s, suffix) = unsafe { bytes.align_to_mut::<u64>() };
    if !prefix.is_empty() {
        block_core.fill_bytes(bytes);
        return;
    }
    fill_bytes_inner(block_core, u64s, suffix);
}

#[inline(always)]
fn fill_bytes_inner(
    block_core: &mut BlockRng<TripleMixSimdCore>,
    u64s: &mut [u64],
    suffix: &mut [u8],
) {
    let remaining = block_core.remaining_results();
    if u64s.len() <= remaining.len() {
        for word in u64s.iter_mut() {
            *word = block_core.next_word();
        }
    } else {
        u64s[0..remaining.len()].copy_from_slice(remaining);
        let (dst_blocks, tail) = u64s[remaining.len()..].as_chunks_mut();
        if !dst_blocks.is_empty() {
            block_core.core.fill_blocks(dst_blocks);
        }
        block_core.reset_and_skip(0); // mark the buffer contents as used
        for tail_u64 in tail {
            *tail_u64 = block_core.next_word();
        }
    }
    if !suffix.is_empty() {
        block_core.fill_bytes(suffix);
    }
}

#[cfg(all(
    test,
    any(
        feature = "reproducibility_cross_platform",
        feature = "reproducibility_same_endianness"
    )
))]
fn test_equivalence_generic<R: Reproducibility>() {
    use crate::TripleMixPrng;
    use crate::seed::DEFAULT_SEED_SIZE;
    use bytemuck::cast_slice_mut;
    use rand_core::Rng;
    let seed = [0u8; DEFAULT_SEED_SIZE];
    let mut prng1 = TripleMixPrng::<R>::from(&seed);
    let mut prng2 = TripleMixPrng::<R>::from(&seed);
    for length in [1, 2, 4, 8, 16, 32, 64, 1024] {
        for misalignment in 0..size_of::<u64>() {
            // Force buffer edges to be aligned on 64 bits, so that written portion will be misaligned
            let mut buf1 = vec![0u64; (length + size_of::<u64>()) / size_of::<u64>() + 1];
            let buf1: &mut [u8] = cast_slice_mut(&mut buf1);
            let buf1 = &mut buf1[misalignment..(length + misalignment)];
            prng1.fill_bytes(buf1);
            let mut buf2 = vec![0u64; (length + size_of::<u64>()) / size_of::<u64>() + 1];
            let buf2: &mut [u8] = cast_slice_mut(&mut buf2);
            let buf2 = &mut buf2[0..length];
            if length.is_multiple_of(size_of::<u64>()) {
                for chunk in buf2.chunks_exact_mut(size_of::<u64>()) {
                    let next_word = prng2.next_u64();
                    chunk.copy_from_slice(&R::u64_as_bytes(next_word));
                }
            } else {
                prng2.fill_bytes(buf2);
            }
            assert_eq!(&*buf1, &*buf2);
        }
    }
}
