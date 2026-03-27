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
            let expected = "5315C4087B9F8E6E1B2233550C8876CBD24B098B4CF89ACE546BAB296AA8DD60520B6D67A569A1B752DE0F2B95BE8F903E3136B890C88DB410F68FD849BF3172CD027E6B8D410D7649E8F36D67A5307F5EE8F0CC734A575D9C1CBF35A906563D6EC2D5EF991E416266AA62A839788D4264EB9C4F23C1158203475184C256A3E1E1B543FAD942AB6C6DC9C7E85A8205C002E8E3CC25544FFF0500FA8E67F7CDE044A325ED3B35ABF10221BCE13A2E3F6A47C1D51A04D2C6D6095E57A2B8C12C317AF77C4566F98316FDBBB47E99940D93D56FE49695FF4A739B1F510B8C4794C02455CB4F88462D3D55B47604F734D836EBF3C08382852A57029FF418A9DF716F";
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
