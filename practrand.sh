#!/usr/bin/env bash
mkdir -p target
(
  # First 50 seeds/lengths
  for _ in {1..16}; do echo "r 1G"; echo "r 16G"; echo "r 128G"; done
  echo "r 1T"
  echo "r 32T"
  # Remaining, descending by output length
  echo "z 32T"
  echo "00 32T"
  echo "01 32T"
  for _ in {1..4}; do echo "r 32T"; done
  for _ in {1..31}; do echo "r 1T"; done
  for _ in {1..112}; do echo "r 128G"; done
  for _ in {1..1008}; do echo "r 16G"; done
  for _ in {1..4080}; do echo "r 1G"; done
) | parallel -j 5 --colsep ' ' --lb ' \
  ./target/release/examples/endless_bytes {1} 2> >(tee target/practrand_{#}.txt) \
  | /mnt/c/Users/cryoc/Downloads/PractRand_0.96/PractRand/RNG_test \
    stdin -multithreaded -tlmax {2} -tlshow 6T -tlshow 10T -tlshow 12T -tlshow 14T -tlshow 18T -tlshow 20T -tlshow 22T -tlshow 24T -tlshow 26T -tlshow 28T -tlshow 30T 2>&1 \
  | tee -a target/practrand_{#}.txt'