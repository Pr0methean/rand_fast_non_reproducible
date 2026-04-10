rand_triplemix
==============

This is a vectorized pseudorandom number generator (PRNG) that combines, in each of 4 SIMD lanes, an instance of 
PCG64, TinyMT64 and a 128-bit multiply-with-carry generator (MCG) in each of 4 SIMD lanes. This PRNG has not been 
evaluated for cryptographic use.

Requires the `portable_simd` feature, which is currently nightly-only.

The PRNG has the following properties:

* The output block size is 64 bytes (8 u64's).
* The state size is 296 bytes: 508 bits of identity, 1852 bits of mutable state, 8 bits of overhead.
* The period is greater than 2<sup>1087</sup> - 2<sup>1036</sup> - 2<sup>1034</sup> blocks, because it's the product of 
  the subgenerators' coprime periods, which are:
  * PCG64: 2<sup>128</sup>
  * Xoshiro256**: 2<sup>256</sup> - 1
  * TinyMT64: 2<sup>127</sup> - 1
  * MCG (lane 0): 2<sup>128</sup> - 742×2<sup>64</sup> - 1
  * MCG (lane 1): 2<sup>128</sup> - 5572×2<sup>64</sup> - 1
  * MCG (lane 2): 2<sup>128</sup> - 1432×2<sup>64</sup> - 1
  * MCG (lane 3): 2<sup>128</sup> - 1108×2<sup>64</sup> - 1
  * Scalar Weyl sequence: 2<sup>64</sup> - 59
* Every possible 64-byte block is produced no more than 1 + 2<sup>-63</sup> times as often as any other possible block.
* Can be created with a seed of any length, but will initialize fastest with a seed of 72, 144, 216, 288, or 360 bytes.
* The initial identity and state are derived from the seed using a Feistel permutation. Each possible valid internal 
  state results from no more than 1 + 2<sup>-61</sup> (the _preimage ratio_) times as many 360-byte seeds as any other, 
  assuming that SHA3-512-KMAC behaves like a random function.
* The seeding function ensures no sub-generator will have the same state in two different SIMD lanes.
* `fork()` and `fork_with_domain_separation()` derive a statistically independent state from the current state and then
  update the parent PRNG's state.
* Within each lane, the mutable-state bits form a single cycle.
* The mixing function uses SIMD-lane-specific constants, so it will not generate closely related output in different 
  lanes even when those lanes have a similar internal state.
* Runs in 0.50-0.52 cycles per byte on Raptor Lake (Ubuntu and Windows), versus 0.59-0.71 for ChaCha12Rng.
* Byte-sequence entropy measurements (based on 16 GiB from an instance produced by 
  `TripleMixPrng::<NotReproducible>::almost_all_zeroes_state()` on an AVX2 CPU, calculated using 
  https://github.com/Pr0methean/EntroPy) are:
  | Entropy measure   | Value (bits/byte)     |
  |-------------------|-----------------------|
  | 0th-order H0      | 7.999 999 989 676 873 |
  | 1st-order H1\|0   | 7.999 997 290 432 957 |
  | 2nd-order H2\|1,0 | 7.999 298 245 455 499 |
* Passes PractRand 0.96 for at least 32 TiB (tested with 10 seeds).