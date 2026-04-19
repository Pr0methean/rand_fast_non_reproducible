use crate::common::get_random_seed;
use rand_core::Rng;
use rand_triplemix::TripleMixPrng;
use rand_triplemix::reproducibility::NotReproducible;
use rand_triplemix::seed::LARGE_SEED_SIZE;
use std::env::args_os;
use std::ffi::OsString;
use std::io::{Write, stdout};
use std::str::FromStr;

mod common;

fn main() {
    let args: Vec<_> = args_os().collect();
    let total_output_bytes: u64 = if let Some(len_arg) = args.get(2)
        && let Some(len_str) = len_arg.to_str()
    {
        let mut s = len_str.to_uppercase();
        let multiplier = if s.ends_with('T') {
            s.pop();
            1u64 << 40
        } else if s.ends_with('G') {
            s.pop();
            1u64 << 30
        } else if s.ends_with('M') {
            s.pop();
            1u64 << 20
        } else if s.ends_with('K') {
            s.pop();
            1u64 << 10
        } else {
            1
        };
        s.parse::<u64>().expect("Invalid length argument") * multiplier
    } else {
        1 << 34 // 16 GiB default
    };

    let mut prng: TripleMixPrng<NotReproducible>;
    if let Some(seed_arg) = args.get(1)
        && let Some(seed_arg_utf8) = seed_arg.to_str()
        && let Ok(decoded_seed) = hex::decode(seed_arg_utf8)
    {
        let mut seed = [0u8; LARGE_SEED_SIZE];
        seed[0..(LARGE_SEED_SIZE.min(decoded_seed.len()))].copy_from_slice(&decoded_seed);
        eprintln!("Seed: {}", seed.map(|b| format!("{:02X}", b)).join(""));
        prng = TripleMixPrng::from(seed);
    } else if args.get(1) == Some(&OsString::from_str("z").unwrap()) {
        prng = TripleMixPrng::almost_all_zeroes_state();
    } else {
        let seed = get_random_seed();
        prng = TripleMixPrng::from(seed);
    }
    let mut output_so_far = 0;
    let mut stdout = stdout().lock();
    loop {
        let mut buffer = [0u8; 1 << 14];
        prng.fill_bytes(&mut buffer);
        if let Err(e) = stdout.write_all(&buffer) {
            eprintln!("Error writing to stdout: {}", e);
            return;
        }
        output_so_far += buffer.len() as u64;
        if output_so_far >= total_output_bytes {
            break;
        }
    }
}
