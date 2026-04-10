#!/usr/bin/env bash
parallel -j 5 --lb './target/release/examples/endless_bytes {} 2> >(tee target/practrand_{#}.txt) \
  | /mnt/c/Users/cryoc/Downloads/PractRand_0.96/PractRand/RNG_test \
    stdin -multithreaded -tlshow 6T -tlshow 10T -tlshow 12T -tlshow 14T -tlshow 18T -tlshow 20T -tlshow 22T -tlshow 24T -tlshow 26T -tlshow 28T -tlshow 30T 2>&1 \
  | tee -a target/practrand_{#}.txt' \
  ::: z 00 01 r r r r r r r