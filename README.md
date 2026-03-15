rand_triplemix
==============

This is a vectorized pseudorandom number generator (PRNG) that combines, in each of 4 SIMD lanes, an instance of 
PCG64, TinyMT64 and a 128-bit multiply-with-carry generator (MCG) in each of 4 SIMD lanes. This PRNG has not been 
evaluated for cryptographic use.

Requires the `portable_simd` feature, which is currently nightly-only.

The PRNG has the following properties:

* The output block size is 64 bytes (8 u64's).
* The state size is 256 bytes: 508 bits of identity, 1532 bits of mutable state, 8 bits of overhead.
* The period is greater than 2<sup>763</sup> - 2<sup>712</sup> - 2<sup>710</sup> blocks, because it's the product of the subgenerators' coprime periods:
  * PCG64: 2<sup>128</sup>
  * TinyMT64: 2<sup>127</sup> - 1
  * MCG (lane 0): 2<sup>127</sup> - 742×2<sup>63</sup> - 1
  * MCG (lane 1): 2<sup>127</sup> - 5571×2<sup>63</sup> - 1
  * MCG (lane 2): 2<sup>127</sup> - 1431×2<sup>63</sup> - 1
  * MCG (lane 3): 2<sup>127</sup> - 1107×2<sup>63</sup> - 1
* The 3-step output mapping achieves an average linear rank of 1531.94, with standard deviation less than 0.3.
* The generator is approximately k-equidistributed for 64-bit outputs for 2 ≤ k ≤ 11. This means that over its full 
  period for any given seed, every possible sequence of k consecutive 64-bit values occurs, and no sequence 
  occurs fewer than 1 − 2<sup>-59</sup> times as often as any other.
* The state size is 192 bytes, of which only 4 bits is overhead.
* The period is greater than 2<sup>763</sup> - 2<sup>712</sup> - 2<sup>710</sup> blocks, which is the product of the subgenerators' coprime periods:
  * Xoroshiro++: 2<sup>128</sup> - 1
  * TinyMT64: 2<sup>127</sup> - 1
  * MCG (lane 0): 2<sup>127</sup> - 742×2<sup>63</sup> - 1
  * MCG (lane 1): 2<sup>127</sup> - 5571×2<sup>63</sup> - 1
  * MCG (lane 2): 2<sup>127</sup> - 1431×2<sup>63</sup> - 1
  * MCG (lane 3): 2<sup>127</sup> - 1107×2<sup>63</sup> - 1
* The 3-step output mapping achieves an average linear rank of 1531.94, with standard deviation less than 0.3.
* Can be created with a seed of any length.
* The initial identity and state are derived from the seed using a Feistel permutation. A 2048-bit seed should make all
  but one in 10<sup>111</sup> valid states possible, and a 2100-bit seed should make all valid states possible.
* The seeding function ensures no sub-generator will have the same state in two different SIMD lanes.
* `fork()` and `fork_with_domain_separation()` derive a statistically independent state from the current state and then
  update the parent PRNG's state.
* Within each lane, the mutable-state bits form a single cycle.
* The mixing function uses SIMD-lane-specific constants, so it will not generate closely related output in different 
  lanes even when those lanes have a similar internal state.
* Runs in 0.6 cycles per byte on Ubuntu with AVX2, faster than ChaCha12Rng.
* Byte-sequence entropy measurements (based on 16 GiB from an instance produced by 
  `TripleMixPrng::<NotReproducible>::almost_all_zeroes_state()` on an AVX2 CPU, calculated using 
  https://github.com/Pr0methean/EntroPy) are:
  | Entropy measure   | Value (bits/byte)     |
  |-------------------|-----------------------|
  | 0th-order H0      | 7.999 999 987 438 837 |
  | 1st-order H1\|0   | 7.999 997 282 359 925 |
  | 2nd-order H2\|1,0 | 7.999 297 800 208 081 |
* Passes PractRand 0.96 for at least 32 TiB (tested with 10 seeds).