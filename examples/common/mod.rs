use rand::rngs::SysRng;
use rand_core::TryRng;
use rand_triplemix::seed::LARGE_SEED_SIZE;
use std::thread;

pub fn get_random_seed() -> [u8; LARGE_SEED_SIZE] {
    const OS_ENTROPY_BYTES: usize = 32;
    let mut seed = [0u8; LARGE_SEED_SIZE];
    for (index, chunk) in seed.chunks_mut(OS_ENTROPY_BYTES).enumerate() {
        #[cfg_attr(
            not(any(target_arch = "x86_64", target_arch = "x86")),
            allow(unused_mut)
        )]
        let mut seeded = false;
        if index >= 2 {
            #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
            if let Ok(mut rd_seed) = rdrand::RdSeed::new()
                && rd_seed.try_fill_bytes(chunk).is_ok()
            {
                eprintln!("Generated a seed chunk using RDSEED");
                seeded = true;
            } else {
                eprintln!("RDSEED failed.");
            }
        }
        if !seeded {
            SysRng.try_fill_bytes(chunk).unwrap();
            eprintln!("Generated a seed chunk using OS RNG");
            thread::yield_now();
        }
    }
    eprintln!("Seed: {}", seed.map(|b| format!("{:02X}", b)).join(""));
    seed
}
