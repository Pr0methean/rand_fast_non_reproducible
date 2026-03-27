use crate::TripleMixSimdCore;
use bytemuck::{cast, cast_slice};
use rand_core::block::BlockRng;
use crate::generate::{Simd32, Simd64};

/// Levels of reproducibility for output of [`TripleMixPrng::fill_bytes`] and output after
/// fill_bytes has been called.
pub trait Reproducibility: Clone + Copy {
    type U8Slice<'a>: AsRef<[u8]>;
    type U64Slice<'a>: AsRef<[u64]>;
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
    type U64Slice<'a> = &'a [u64];
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
    use crate::reproducibility::{Reproducibility, fill_bytes_alignment_aware};
    use bytemuck::{cast, cast_slice};
    use rand_core::block::BlockRng;
    use crate::generate::{Simd32, Simd64};

    #[derive(Copy, Clone, Default, Debug)]
    pub struct SameEndianness;

    impl Reproducibility for SameEndianness {
        type U8Slice<'a> = &'a [u8];
        type U64Slice<'a> = &'a [u64];

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
    use bytemuck::cast;
    use crate::TripleMixSimdCore;
    use crate::reproducibility::{Reproducibility};
    use rand_core::block::BlockRng;
    #[cfg(target_endian = "little")]
    use bytemuck::cast_slice;
    use crate::generate::{Simd32, Simd64};
    #[cfg(target_endian = "little")]
    use crate::reproducibility::fill_bytes_alignment_aware;
    
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
        #[test]
        fn test_cross_platform_reproducibility() {
            use super::CrossPlatform;
            use crate::TripleMixPrng;
            use itertools::Itertools;
            use rand_core::Rng;
            let seed = [0u8; 72];
            let mut prng = TripleMixPrng::<CrossPlatform>::from(&seed);
            let expected = "C2FD4753704A5E6D84F8539ACB3DF624714A688C67B70DA344C49839BDF1EBDC1A79E2E6ED3E47ED26F46180F3952744BFBB67BA0E60513D3E1F0E45739BF570F793CF7E59F76743428B5427BB850AAFB57F67EBB8CE274D644DC21B06CEB8EA4DE725BDC1137BBA6EF0AB062ACEB37C0EC3BFF8680599C0D0656C80D62D862B8E4966808D38F6C3029C71E48A8DAE4E6A1DB79CBF03C8AA3AD0F1C2525D486D83F0E33AEE6191FAAE869EF0AFC49F94699B54634F634B10C37640BFD0A73B15231C4418E46DCA34F058648641A40C5C6DCFFDB18D550C4E84D2018D283E7210E6DE46A27F21AB8ADD1F70A2A80CEB88AD86CBE9C13156AFEDE4F8C7AB83D20E";
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
fn fill_bytes_alignment_aware<R: Reproducibility>(block_core: &mut BlockRng<TripleMixSimdCore<R>>, bytes: &mut [u8]) {
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
