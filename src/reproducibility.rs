use bytemuck::cast_slice;
use rand_core::block::{BlockRng, Generator};
use rand_core::utils::Word;

/// Levels of reproducibility for output of [`TripleMixPrng::fill_bytes`] and output after
/// fill_bytes has been called.
pub trait Reproducibility: Clone + Copy {
    type U8Slice<'a>: AsRef<[u8]>;
    fn fill_bytes<W: Word, const N: usize, G: Generator<Output = [W; N]>>(core: &mut BlockRng<G>, bytes: &mut [u8]);
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
    #[inline(always)]
    fn fill_bytes<W: Word, const N: usize, G: Generator<Output = [W; N]>>(block_core: &mut BlockRng<G>, bytes: &mut [u8]) {
        let (prefix, words, suffix) = unsafe { bytes.align_to_mut::<W>() };
        if words.is_empty() {
            // There's no benefit to bypassing the buffer or consolidating
            // writes if we can't write at least one aligned u64.
            block_core.fill_bytes(bytes);
            return;
        }
        if !prefix.is_empty() {
            block_core.fill_bytes(prefix);
        }
        fill_bytes_inner::<NotReproducible, W, N, G>(block_core, words, suffix);
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
    use crate::reproducibility::{fill_bytes_alignment_aware, Reproducibility};
    use bytemuck::cast_slice;
    use rand_core::block::{BlockRng, Generator};
    use rand_core::utils::Word;

    #[derive(Copy, Clone, Default, Debug)]
    pub struct SameEndianness;

    impl Reproducibility for SameEndianness {
        type U8Slice<'a> = &'a [u8];

        fn fill_bytes<W: Word, const N: usize, G: Generator<Output = [W; N]>>(block_core: &mut BlockRng<G>, bytes: &mut [u8]) {
            fill_bytes_alignment_aware::<SameEndianness, W, N, G>(block_core, bytes);
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
    #[cfg(target_endian = "little")]
    use crate::reproducibility::fill_bytes_alignment_aware;
    use crate::reproducibility::Reproducibility;
    #[cfg(target_endian = "little")]
    use bytemuck::cast_slice;
    use rand_core::block::{BlockRng, Generator};
    use rand_core::utils::Word;

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
        fn fill_bytes<W: Word, const N: usize, G: Generator<Output = [W; N]>>(block_core: &mut BlockRng<G>, bytes: &mut [u8]) {
            fill_bytes_alignment_aware::<CrossPlatform, W, N, G>(block_core, bytes);
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
    }

    #[test]
    fn test_equivalence_cross_platform() {
        super::test_equivalence_generic::<CrossPlatform>();
    }
}

#[cfg(any(
    all(feature = "reproducibility_cross_platform", target_endian = "little"),
    feature = "reproducibility_same_endianness"
))]
#[inline(always)]
fn fill_bytes_alignment_aware<R: Reproducibility, W: Word, const N: usize, G: Generator<Output = [W; N]>>(
    block_core: &mut BlockRng<G>,
    bytes: &mut [u8],
) {
    let (prefix, words, suffix) = unsafe { bytes.align_to_mut::<W>() };
    if !prefix.is_empty() {
        block_core.fill_bytes(bytes);
        return;
    }
    fill_bytes_inner::<R, W, N, G>(block_core, words, suffix);
}

#[inline(always)]
fn fill_bytes_inner<R: Reproducibility, W: Word, const N: usize, G: Generator<Output = [W; N]>>(
    block_core: &mut BlockRng<G>,
    output_words: &mut [W],
    suffix: &mut [u8],
) {
    let remaining = block_core.remaining_results();
    if output_words.len() <= remaining.len() {
        for word in output_words.iter_mut() {
            *word = block_core.next_word();
        }
    } else {
        output_words[0..remaining.len()].copy_from_slice(remaining);
        let (dst_blocks, tail) = output_words[remaining.len()..].as_chunks_mut::<N>();
        if !dst_blocks.is_empty() {
            for block in dst_blocks {
                block_core.core.generate(block);
            }
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
    extern crate alloc;
    use alloc::vec;
    use crate::TestFastBlockRng;
    use bytemuck::cast_slice_mut;
    use rand_core::Rng;
    use chacha20::{ChaChaCore, KeyIvInit};
    use rand_core::UnwrapErr;

    use rand::rngs::SysRng;
    let mut os_rng = UnwrapErr(SysRng);
    let mut key = [0u8; 32];
    let mut iv = [0u8; 12];
    os_rng.fill_bytes(&mut key);
    os_rng.fill_bytes(&mut iv);
    let mut prng1 = TestFastBlockRng::<R>::from_core(ChaChaCore::new(&key.into(), &iv.into()));
    let mut prng2 = TestFastBlockRng::<R>::from_core(ChaChaCore::new(&key.into(), &iv.into()));
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
