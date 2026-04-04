use gungraun::{main, library_benchmark_group, library_benchmark};

#[cfg(feature = "bench_include_threadrng")]
use rand::rng;
use rand::rngs::SysRng;
use rand_core::{Rng, SeedableRng, UnwrapErr};
use rand_triplemix::reproducibility::NotReproducible;
#[cfg(feature = "reproducibility_cross_platform")]
use rand_triplemix::reproducibility::cross_platform::CrossPlatform;
#[cfg(feature = "reproducibility_same_endianness")]
use rand_triplemix::reproducibility::same_endianness::SameEndianness;
use rand_triplemix::{BLOCK_SIZE, TripleMixPrng};
use std::hint::black_box;
use rand::RngExt;

const BENCHMARK_OUTPUT_SIZE: usize = 128 * BLOCK_SIZE;

#[library_benchmark]
#[bench::triple_mix_prng_not_reproducible(TripleMixPrng::<NotReproducible>::from_rng(&mut UnwrapErr(SysRng)), &mut [0u64; BENCHMARK_OUTPUT_SIZE])]
#[cfg_attr(feature = "reproducibility_same_endianness", bench::triple_mix_prng_same_endianness(TripleMixPrng::<SameEndianness>::from_rng(&mut UnwrapErr(SysRng)), &mut [0u64; BENCHMARK_OUTPUT_SIZE]))]
#[cfg_attr(feature = "reproducibility_cross_platform", bench::triple_mix_prng_cross_platform(TripleMixPrng::<CrossPlatform>::from_rng(&mut UnwrapErr(SysRng)), &mut [0u64; BENCHMARK_OUTPUT_SIZE]))]
#[cfg_attr(feature = "bench_include_threadrng", bench::thread_prng(rand::rng(), &mut [0u64; BENCHMARK_OUTPUT_SIZE]))]
fn fill_16k<T: Rng>(mut prng: T, buffer: &mut [u64; BENCHMARK_OUTPUT_SIZE]) -> &[u64; BENCHMARK_OUTPUT_SIZE] {
    prng.fill(buffer.as_mut_slice());
    black_box(&*buffer)
}

library_benchmark_group!(name = generate, benchmarks = fill_16k);
main!(library_benchmark_groups = generate);