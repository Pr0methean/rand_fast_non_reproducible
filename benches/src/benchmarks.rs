use criterion::measurement::Measurement;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use const_format::formatcp;
use core::time::Duration;
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use criterion_cycles_per_byte::CyclesPerByte;
use dyn_clone::{clone_box, DynClone};
#[cfg(feature = "bench_include_threadrng")]
use rand::rng;
use rand::rngs::SysRng;
use rand_core::{Rng, SeedableRng, TryRng};
#[cfg(feature = "reproducibility_cross_platform")]
use rand_triplemix::reproducibility::cross_platform::CrossPlatform;
#[cfg(feature = "reproducibility_same_endianness")]
use rand_triplemix::reproducibility::same_endianness::SameEndianness;
use rand_triplemix::reproducibility::NotReproducible;
use rand_triplemix::seed::{DEFAULT_SEED_SIZE, LARGE_SEED_SIZE};
use rand_triplemix::{TripleMixPrng, BLOCK_SIZE};
use std::env::consts::{ARCH, OS};
use core::hint::black_box;
use core::mem::size_of;

const PLATFORM: &str = formatcp!("{ARCH}:{OS}");

trait DynCloneRng: DynClone + Rng {}
impl<T: DynClone + Rng> DynCloneRng for T {}

fn generate<T: Measurement + 'static>(c: &mut Criterion<T>) {
    // Allocate buffer as u64's so that it's aligned
    const MAX_ALIGNMENT: usize = size_of::<u64>() - 1;
    const BUFFER_LEN: usize = 16 * 1024;

    const LARGE_FILL_LEN: usize = 1024 * 1024;
    let prngs = create_prngs();
    for alignment in [0, 1, MAX_ALIGNMENT] {
        let mut group = c.benchmark_group(format!("{PLATFORM}: fill_bytes 16KiB (misalignment {alignment})"));
        group.throughput(Throughput::Bytes(BUFFER_LEN as u64));
        for (prng_name, prng) in prngs.iter() {
            let mut prng = clone_box(&**prng);
            group.bench_function(prng_name.to_string(), move |b| {
                let mut buffer = vec![0u64; BUFFER_LEN / size_of::<u64>() + 1]; // 1 MiB plus de-alignment padding
                let (_, buffer, _) = unsafe { buffer.align_to_mut::<u8>() };
                let misaligned_buffer = &mut buffer[alignment..(BUFFER_LEN + alignment)];
                b.iter(|| {
                    prng.fill_bytes(misaligned_buffer);
                    black_box(&*misaligned_buffer);
                })
            });
        }
        group.finish();
    }
    let mut group = c.benchmark_group(formatcp!("{PLATFORM}: fill_bytes 1MiB"));
    group.throughput(Throughput::Bytes(LARGE_FILL_LEN as u64));
    for (prng_name, prng) in prngs.iter() {
        let mut fill_bytes_prng = clone_box(&**prng);
        const ITERATIONS: usize = LARGE_FILL_LEN / BUFFER_LEN;
        group.bench_function(prng_name.to_string(), move |b| {
            let mut buffer = vec![0u64; BUFFER_LEN / size_of::<u64>()];
            let (_, buffer, _) = unsafe { buffer.align_to_mut::<u8>() };
            b.iter(|| {
                let mut acc = 0u8;
                for i in 0..ITERATIONS {
                    fill_bytes_prng.fill_bytes(buffer);
                    acc ^= black_box(buffer[black_box(i)]);
                }
                acc
            })
        });
    }
    group.finish();
    const U64_ITERATIONS: usize = 12;
    let mut group = c.benchmark_group(formatcp!("{PLATFORM}: next_u64"));
    group.throughput(Throughput::Bytes(
        (size_of::<u64>() * U64_ITERATIONS) as u64,
    ));
    for (prng_name, mut prng) in prngs.into_iter() {
        group.bench_function(prng_name.to_string(), move |b| {
            b.iter(|| {
                let mut accum = prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum ^= prng.next_u64();
                accum
            })
        });
    }
    group.finish();
}

fn create_prngs() -> Vec<(&'static str, Box<dyn DynCloneRng>)> {
    let mut seed = [0u8; DEFAULT_SEED_SIZE];
    SysRng.try_fill_bytes(&mut seed).unwrap();

    let mut prngs = Vec::<(&'static str, Box<dyn DynCloneRng>)>::new();
    prngs.push((
        "TripleMixPrng",
        Box::new(TripleMixPrng::<NotReproducible>::from(&seed)),
    ));
    #[cfg(feature = "reproducibility_same_endianness")]
    prngs.push((
        "TripleMixPrng<SameEndianness>",
        Box::new(TripleMixPrng::<SameEndianness>::from(&seed)),
    ));
    #[cfg(feature = "reproducibility_cross_platform")]
    prngs.push((
        "TripleMixPrng<CrossPlatform>",
        Box::new(TripleMixPrng::<CrossPlatform>::from(&seed)),
    ));
    #[cfg(feature = "bench_include_threadrng")]
    prngs.push(("ThreadRng", Box::new(rng())));
    prngs
}

fn core<T: Measurement>(c: &mut Criterion<T>) {
    let mut seed = [0u8; DEFAULT_SEED_SIZE];
    SysRng.try_fill_bytes(&mut seed).unwrap();
    let mut prng = TripleMixPrng::<NotReproducible>::from(&seed);
    let mut group = c.benchmark_group("core");
    group.throughput(Throughput::Bytes((BLOCK_SIZE * size_of::<u64>()) as u64));
    let mut block = [[0u64; BLOCK_SIZE]];
    group.bench_function("fill_blocks", move |b| {
        b.iter(|| {
            prng.fill_blocks_unbuffered(&mut block);
            black_box(block);
        })
    });
}

fn init<T: Measurement>(c: &mut Criterion<T>) {
    let mut group = c.benchmark_group("initialization");

    // Seed and instance setup
    let mut seed_4096 = [0u8; 512];
    SysRng.try_fill_bytes(&mut seed_4096).unwrap();

    // Benchmark from_seed with various sizes
    for size in [
        8,
        16,
        32,
        64,
        DEFAULT_SEED_SIZE,
        128,
        256,
        LARGE_SEED_SIZE,
        512,
    ] {
        let input_seed = &seed_4096[..size];
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("from_seed", size),
            input_seed,
            move |b, s| b.iter(|| black_box(TripleMixPrng::<NotReproducible>::from(black_box(s)))),
        );
    }

    let mut parent = TripleMixPrng::<NotReproducible>::from(&seed_4096);

    // Benchmark fork()
    group.bench_function("fork", move |b| {
        b.iter(|| {
            // Using black_box to ensure the compiler doesn't optimize away the result
            black_box(parent.fork())
        })
    });

    group.finish();
}

// Using criterion_cycles_per_byte on aarch64 requires a custom Linux kernel module, so it's not an
// option on GitHub Actions hosted runners; and aarch64 on other OSs isn't currently supported.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
criterion_group!(
    name = benches;
    config = Criterion::default().with_measurement(CyclesPerByte).warm_up_time(Duration::from_secs(10));
    targets = generate, init, core
);
#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
criterion_group!(
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(10));
    targets = generate, init, core
);
criterion_main!(benches);
