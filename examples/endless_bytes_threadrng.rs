// This example is used as a baseline for comparing TripleMixPrng's speed with ThreadRng.

use rand::rngs::ThreadRng;
use rand::rng;
use rand_core::Rng;
use std::io::{Write, stdout};

fn main() {
    let mut prng: ThreadRng = rng();
    let mut stdout = stdout().lock();
    let mut buffer = [0u8; 1 << 16];
    loop {
        #[cfg(feature = "llvm-mca")]
        llvm_mca::llvm_mca_begin!("fill_bytes");
        prng.fill_bytes(&mut buffer);
        #[cfg(feature = "llvm-mca")]
        llvm_mca::llvm_mca_end!("fill_bytes");
        if let Err(e) = stdout.write_all(&buffer) {
            eprintln!("Error writing to stdout: {}", e);
            return;
        }
    }
}
