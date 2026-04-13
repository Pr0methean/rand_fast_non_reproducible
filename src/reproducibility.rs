use crate::TripleMixSimdCore;
use crate::generate::{Simd32, Simd64};
use bytemuck::{cast, cast_slice};
use rand_core::block::BlockRng;

/// Levels of reproducibility for output of [`TripleMixPrng::fill_bytes`] and output after
/// fill_bytes has been called.
pub trait Reproducibility: Clone + Copy {
    type U8Slice<'a>: AsRef<[u8]>;
    fn fill_bytes(core: &mut BlockRng<TripleMixSimdCore<Self>>, bytes: &mut [u8]);
    fn cast_u64_slice_as_u8(slice: &[u64]) -> Self::U8Slice<'_>;
    fn u64_as_bytes(input: u64) -> [u8; 8];
    fn u128_as_bytes(input: u128) -> [u8; 16];
    fn simd64_as_simd32(input: Simd64) -> Simd32;
    fn simd32_as_simd64(input: Simd32) -> Simd64;
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
    #[inline(always)]
    fn fill_bytes(block_core: &mut BlockRng<TripleMixSimdCore<Self>>, bytes: &mut [u8]) {
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
        fill_bytes_inner::<Self>(block_core, u64s, suffix);
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

    #[inline(always)]
    fn simd64_as_simd32(input: Simd64) -> Simd32 {
        cast(input)
    }

    #[inline(always)]
    fn simd32_as_simd64(input: Simd32) -> Simd64 {
        cast(input)
    }
}

/// Output of the PRNG will be the same as for an instance created with the same seed and receiving
/// the same calls (counting two `fill_bytes` as the same when they write to slices of the same
/// length), as long as both instances are created on machines with the same endianness.
#[cfg(feature = "reproducibility_same_endianness")]
pub mod same_endianness {
    use crate::TripleMixSimdCore;
    use crate::generate::{Simd32, Simd64};
    use crate::reproducibility::{Reproducibility, fill_bytes_alignment_aware};
    use bytemuck::{cast, cast_slice};
    use rand_core::block::BlockRng;

    #[derive(Copy, Clone, Default, Debug)]
    pub struct SameEndianness;

    impl Reproducibility for SameEndianness {
        type U8Slice<'a> = &'a [u8];

        fn fill_bytes(core: &mut BlockRng<TripleMixSimdCore<Self>>, bytes: &mut [u8]) {
            fill_bytes_alignment_aware::<Self>(core, bytes);
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

        #[inline(always)]
        fn simd64_as_simd32(input: Simd64) -> Simd32 {
            cast(input)
        }

        #[inline(always)]
        fn simd32_as_simd64(input: Simd32) -> Simd64 {
            cast(input)
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
    use crate::generate::{Simd32, Simd64};
    use crate::reproducibility::Reproducibility;
    #[cfg(target_endian = "little")]
    use crate::reproducibility::fill_bytes_alignment_aware;
    use bytemuck::cast;
    #[cfg(target_endian = "little")]
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
        #[inline(always)]
        fn fill_bytes(block_core: &mut BlockRng<TripleMixSimdCore<Self>>, bytes: &mut [u8]) {
            fill_bytes_alignment_aware::<Self>(block_core, bytes);
        }
        #[cfg(target_endian = "big")]
        #[inline(always)]
        fn fill_bytes(block_core: &mut BlockRng<TripleMixSimdCore<Self>>, bytes: &mut [u8]) {
            block_core.fill_bytes(bytes);
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

        #[cfg(target_endian = "little")]
        #[inline(always)]
        fn simd64_as_simd32(input: Simd64) -> Simd32 {
            cast(input)
        }

        #[cfg(target_endian = "big")]
        #[inline(always)]
        fn simd64_as_simd32(input: Simd64) -> Simd32 {
            use core::simd::simd_swizzle;
            simd_swizzle!(cast::<Simd64, Simd32>(input), [1, 0, 3, 2, 5, 4, 7, 6])
        }

        #[cfg(target_endian = "little")]
        #[inline(always)]
        fn simd32_as_simd64(input: Simd32) -> Simd64 {
            cast(input)
        }

        #[cfg(target_endian = "big")]
        #[inline(always)]
        fn simd32_as_simd64(input: Simd32) -> Simd64 {
            use core::simd::simd_swizzle;
            let swizzled = simd_swizzle!(input, [1, 0, 3, 2, 5, 4, 7, 6]);
            cast(swizzled)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::seed::SMALL_SEED_SIZE;

        #[test]
        fn test_cross_platform_reproducibility() {
            use super::CrossPlatform;
            use crate::TripleMixPrng;
            use itertools::Itertools;
            use rand_core::Rng;
            let seed = [0u8; SMALL_SEED_SIZE];
            let mut prng = TripleMixPrng::<CrossPlatform>::from(&seed);
            let expected = "F1744F0C4A9091AE086BF10FCD91DCC65DF1EFFCC0A89D3563F4E2B02B3AF5BDECA5951F2F56CA3ECB57FC72635BD2FD39F04B175D504AE1349F02D78493918883AEB55B3BAF6EE025B1234DA24368564D084742358D1EB47254D891E8B3F69793822C0F7F1656D4F49CB682C9592B18B9DF9452830B9986A2DF29FB7A72D2D6034E7F2AC8D12CCDF4EF96A6CDDAEF25D5E8CD3E67D23C200A4682870F3B37EA10CCBF116C5B96731C757D18C44D427498B10459AF02B9DE4E3B7534C775ABAE1EC436A4C0C145047554FE188BAFB4B5F6E036032F9722540CA078880E5C50314A4071F6B1E50D683416B4758EED4CE3A999E9561CDDC854DA53B8D2C4840F69";
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
    all(feature = "reproducibility_cross_platform", target_endian = "little"),
    feature = "reproducibility_same_endianness"
))]
#[inline(always)]
fn fill_bytes_alignment_aware<R: Reproducibility>(
    block_core: &mut BlockRng<TripleMixSimdCore<R>>,
    bytes: &mut [u8],
) {
    let (prefix, u64s, suffix) = unsafe { bytes.align_to_mut::<u64>() };
    if !prefix.is_empty() {
        block_core.fill_bytes(bytes);
        return;
    }
    fill_bytes_inner::<R>(block_core, u64s, suffix);
}

#[inline(always)]
fn fill_bytes_inner<R: Reproducibility>(
    block_core: &mut BlockRng<TripleMixSimdCore<R>>,
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
    #[cfg(not(miri))]
    const LENGTHS: &[usize] = &[1, 2, 4, 8, 16, 32, 64, 1024];
    #[cfg(miri)]
    const LENGTHS: &[usize] = &[4];
    for &length in LENGTHS {
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
