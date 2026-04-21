use crate::generate::{SIMD_WIDTH, Simd64};
use crate::reproducibility::Reproducibility;
use crate::{TripleMixPrng, TripleMixSimdCore};

impl<R: Reproducibility> TripleMixPrng<R> {
    /// Advances the PRNG by `steps` internal sub-generator steps.
    /// Since the sub-generators are updated once per block of `OUTPUT_LEN` generated `u64` words,
    /// a single step here corresponds to moving past `OUTPUT_LEN` outputs.
    /// The unconsumed buffer of outputs is discarded.
    #[inline]
    pub fn advance(&mut self, steps: u128) {
        self.block_core.core.advance(steps);
        self.block_core.reset_and_skip(0);
    }

    /// Advances the PRNG by exactly `multiples` x 2<sup>128</sup> sub-generator steps.
    /// This is dramatically faster than ordinary `advance` because the 128-bit LCG
    /// exactly wraps its full state space and thus undergoes zero change for every 2^128 steps,
    /// while the matrices for TinyMT and Xoroshiro wrap tightly modulo their periods.
    /// The unconsumed buffer of outputs is discarded.
    #[inline]
    pub fn advance_2_128(&mut self, multiples: u128) {
        self.block_core.core.advance_2_128(multiples);
        self.block_core.reset_and_skip(0);
    }

    /// Advances the PRNG by exactly `multiples` x 2<sup>256</sup> sub-generator steps.
    /// This is dramatically faster than ordinary `advance` because the 128-bit LCG
    /// exactly wraps its full state space and thus undergoes zero change for every 2^128 steps,
    /// while the matrices for TinyMT and Xoroshiro wrap tightly modulo their periods.
    /// The unconsumed buffer of outputs is discarded.
    #[inline]
    pub fn advance_2_256(&mut self, multiples: u128) {
        self.block_core.core.advance_2_256(multiples);
        self.block_core.reset_and_skip(0);
    }
}

/// Represents a linear transformation on the 128-bit state:
/// new_state = matrix * state + constant
struct JumpMatrix<R: Reproducibility> {
    /// Multiplier part (how state is transformed)
    mult_low: Simd64,
    mult_high: Simd64,
    /// Constant part (how increment contributes)
    const_low: Simd64,
    const_high: Simd64,
    _reproducibility: core::marker::PhantomData<R>,
}

impl<R: Reproducibility> JumpMatrix<R> {
    /// Create identity matrix (jump by 0 steps)
    fn identity() -> Self {
        Self {
            mult_low: Simd64::splat(1),
            mult_high: Simd64::splat(0),
            const_low: Simd64::splat(0),
            const_high: Simd64::splat(0),
            _reproducibility: core::marker::PhantomData,
        }
    }

    /// Compose two jump matrices: this * other
    fn compose(&self, other: &Self) -> Self {
        // new_mult = self.mult * other.mult
        let (new_mult_low, new_mult_high) = TripleMixSimdCore::<R>::mul128x128(
            self.mult_high,
            self.mult_low,
            other.mult_high,
            other.mult_low,
        );

        // new_const = self.mult * other.const + self.const
        let (temp_low, temp_high) = TripleMixSimdCore::<R>::mul128x128(
            self.mult_high,
            self.mult_low,
            other.const_high,
            other.const_low,
        );

        let (new_const_low, carry) =
            TripleMixSimdCore::<R>::add128_with_carry(temp_low, self.const_low, Simd64::splat(0));
        let (new_const_high, _) =
            TripleMixSimdCore::<R>::add128_with_carry(temp_high, self.const_high, carry);

        Self {
            mult_low: new_mult_low,
            mult_high: new_mult_high,
            const_low: new_const_low,
            const_high: new_const_high,
            _reproducibility: core::marker::PhantomData,
        }
    }

    /// Apply this jump to a state
    fn apply(&self, state_low: Simd64, state_high: Simd64) -> (Simd64, Simd64) {
        // new_state = mult * state + const
        let (prod_low, prod_high) = TripleMixSimdCore::<R>::mul128x128(
            self.mult_high,
            self.mult_low,
            state_high,
            state_low,
        );

        let (new_low, carry) =
            TripleMixSimdCore::<R>::add128_with_carry(prod_low, self.const_low, Simd64::splat(0));
        let (new_high, _) =
            TripleMixSimdCore::<R>::add128_with_carry(prod_high, self.const_high, carry);

        (new_low, new_high)
    }
}

impl<R: Reproducibility> TripleMixSimdCore<R> {
    const WEYL_JUMP_2_128: u64 = 3481; // (BigUint::from(1) << 128) % Self::SCALAR_WEYL_MODULUS
    const WEYL_JUMP_2_256: u64 = 12117361; // (BigUint::from(1) << 128) % Self::SCALAR_WEYL_MODULUS
    /// 128-bit multiplication by PER-LANE 64-bit multipliers using simd_mulsmall
    /// (high, low) = (a_high, a_low) * b (where b is per lane)
    ///
    /// simd_mulsmall(left: Simd64, right: u32x4) -> (low: Simd64, high: Simd64)
    /// where right's 32-bit values are in the bottom half of each lane
    #[inline]
    pub(crate) fn mul128x64to128(a_high: Simd64, a_low: Simd64, b: Simd64) -> (Simd64, Simd64) {
        // Decompose b into 32-bit halves across all lanes simultaneously
        let b_lo = b & Simd64::splat(0xFFFF_FFFF);
        let b_hi = b >> 32;

        let (p1_lo, p1_hi) = Self::simd_mulsmall(a_low, b_lo);
        let (p2_lo, p2_hi) = Self::simd_mulsmall(a_low, b_hi);

        // p2 * 2^32 = p2_hi * 2^96 + p2_lo * 2^32
        let p2_shifted_lo = p2_lo << Simd64::splat(32);
        let p2_shifted_hi = (p2_hi << Simd64::splat(32)) | (p2_lo >> Simd64::splat(32));

        // low sum = p1_lo + p2_shifted_lo
        let (low_sum, carry1) = Self::add128_with_carry(p1_lo, p2_shifted_lo, Simd64::splat(0));

        let a_low_b_hi = p1_hi + p2_shifted_hi - carry1;

        // the final high part is a_low_b_hi + a_high * b
        // a_high * b = a_high * b_lo + a_high * b_hi * 2^32 (we only care about the low 64 bits of this)
        let a_high_b = crate::generate::simd_wrapping_mul(a_high, b);
        let final_high = a_low_b_hi + a_high_b;

        (final_high, low_sum)
    }

    /// Full 128x128 multiplication. Returns (low, high).
    fn mul128x128(
        a_high: Simd64,
        a_low: Simd64,
        b_high: Simd64,
        b_low: Simd64,
    ) -> (Simd64, Simd64) {
        let (h1, low) = Self::mul128x64to128(a_high, a_low, b_low);
        let h2 = crate::generate::simd_wrapping_mul(a_low, b_high);
        (low, h1 + h2)
    }

    // 2^128 == 2^1 mod (2^127 - 1)
    const TINYMT_JUMP_128_MAT: [u128; 128] = pow_mat_2_exp(Self::TINYMT_JUMP_MAT, 1);
    // 2^256 == 2^2 mod (2^127 - 1)
    const TINYMT_JUMP_256_MAT: [u128; 128] = pow_mat_2_exp(Self::TINYMT_JUMP_MAT, 2);

    fn jump_pcg(&mut self, steps: u128) {
        let mut result = JumpMatrix::<R>::identity();
        let mut base = JumpMatrix::<R> {
            mult_low: Self::PCG_MULTIPLIERS,
            mult_high: Simd64::splat(0),
            const_low: self.pcg_inc_lo,
            const_high: self.pcg_inc_hi,
            _reproducibility: core::marker::PhantomData,
        };

        let mut remaining = steps;
        while remaining > 0 {
            if remaining & 1 == 1 {
                result = result.compose(&base);
            }
            base = base.compose(&base);
            remaining >>= 1;
        }

        let (new_low, new_high) = result.apply(self.pcg_state_lo, self.pcg_state_hi);
        self.pcg_state_lo = new_low;
        self.pcg_state_hi = new_high;
    }

    /// Jump ahead by steps = x * 2^(128*k)
    /// x: u128, k: u64
    /// state_lo/state_hi: current state vectors
    /// a: multiplier (per lane)
    /// returns new (state_lo, state_hi)
    pub(crate) fn mwc_jump(state: Simd64, carry: Simd64, steps: u128, k: u64) -> (Simd64, Simd64) {
        if steps == 0 && k == 0 {
            return (state, carry);
        }

        let mut res_state = state;
        let mut res_carry = carry;

        // Correct MWC state correspondence for LCG jump:
        // The state (W, c) corresponds to the value V = a * W + c.
        // It satisfies the recurrence V_n = a * V_{n-1} mod m,
        // where m = a * 2^64 - 1.
        // Recover (W, c) from V as W = V / a, c = V % a.

        for i in 0..SIMD_WIDTH {
            let a = Self::MCG_MULTIPLIERS[i] as u128;
            let m = (a << 64) - 1;

            // Initial V_0 = a * W_0 + c_0
            let v_0 = a
                .wrapping_mul(state[i] as u128)
                .wrapping_add(carry[i] as u128)
                % m;

            let mut base = a % m;
            let mut exp = steps;

            for _ in 0..k {
                for _ in 0..128 {
                    base = mul_mod(base, base, m);
                }
            }

            let mut ak = 1u128;
            while exp > 0 {
                if exp & 1 != 0 {
                    ak = mul_mod(ak, base, m);
                }
                base = mul_mod(base, base, m);
                exp >>= 1;
            }

            let v_final = mul_mod(v_0, ak, m);

            res_state[i] = (v_final / a) as u64;
            res_carry[i] = (v_final % a) as u64;
        }

        (res_state, res_carry)
    }
}

fn mul_mod(mut x: u128, mut y: u128, m: u128) -> u128 {
    let mut res: u128 = 0;
    x %= m;
    while y > 0 {
        if y & 1 != 0 {
            let sum = res.wrapping_add(x);
            if sum < res || sum >= m {
                res = sum.wrapping_sub(m);
            } else {
                res = sum;
            }
        }
        let double_x = x.wrapping_add(x);
        if double_x < x || double_x >= m {
            x = double_x.wrapping_sub(m);
        } else {
            x = double_x;
        }
        y >>= 1;
    }
    res
}

impl<R: Reproducibility> TripleMixSimdCore<R> {
    #[inline]
    pub(crate) fn advance(&mut self, steps: u128) {
        if steps == 0 {
            return;
        }
        let (new_mwc_state, new_mwc_carry) =
            Self::mwc_jump(self.mwc_state, self.mwc_carry, steps, 0);
        let t_pow = pow_mat(Self::TINYMT_JUMP_MAT, steps);
        self.mwc_state = new_mwc_state;
        self.mwc_carry = new_mwc_carry;
        self.update_t_from_matrix(&t_pow);
        self.jump_pcg(steps);
        let x_pow = pow_mat_256(Self::XOSHIRO256_JUMP_MAT, steps);
        self.xoshiro256 = apply_mat_256(&x_pow, self.xoshiro256);
        let weyl_modulus_u128 = Self::SCALAR_WEYL_MODULUS as u128;
        self.scalar_weyl = ((self.scalar_weyl as u128 + (steps % (weyl_modulus_u128))) % (weyl_modulus_u128)) as u64;
    }

    #[inline]
    pub(crate) fn advance_2_128(&mut self, multiples: u128) {
        if multiples == 0 {
            return;
        }
        let (new_mwc_state, new_mwc_carry) =
            Self::mwc_jump(self.mwc_state, self.mwc_carry, multiples, 1);
        // 2^128 = 1 mod (2^128 - 1)
        let t_pow = pow_mat(Self::TINYMT_JUMP_128_MAT, multiples);
        self.mwc_state = new_mwc_state;
        self.mwc_carry = new_mwc_carry;
        self.update_t_from_matrix(&t_pow);
        // xoshiro256 period is 2^256-1; jump by multiples * 2^128 steps
        let x_pow = pow_mat_256(Self::XOSHIRO256_JUMP_128_MAT, multiples);
        self.xoshiro256 = apply_mat_256(&x_pow, self.xoshiro256);
                let weyl_modulus_128 = Self::SCALAR_WEYL_MODULUS as u128;
        let weyl_jump = mul_mod(Self::WEYL_JUMP_2_128 as u128, multiples, weyl_modulus_128);
        self.scalar_weyl = ((self.scalar_weyl as u128 + weyl_jump + weyl_modulus_128) % weyl_modulus_128) as u64;
    }

    #[inline]
    pub(crate) fn advance_2_256(&mut self, multiples: u128) {
        if multiples == 0 {
            return;
        }
        let (new_mwc_state, new_mwc_carry) =
            Self::mwc_jump(self.mwc_state, self.mwc_carry, multiples, 2);
        // 2^256 = 1 mod (2^128 - 1)
        let t_pow = pow_mat(Self::TINYMT_JUMP_256_MAT, multiples);
        self.mwc_state = new_mwc_state;
        self.mwc_carry = new_mwc_carry;
        self.update_t_from_matrix(&t_pow);
        // 2^256 ≡ 1 mod (2^256 - 1), so multiples * 2^256 ≡ multiples steps
        let x_pow = pow_mat_256(Self::XOSHIRO256_JUMP_256_MAT, multiples);
        self.xoshiro256 = apply_mat_256(&x_pow, self.xoshiro256);
        let weyl_modulus_128 = Self::SCALAR_WEYL_MODULUS as u128;
        let weyl_jump = mul_mod(Self::WEYL_JUMP_2_256 as u128, multiples, weyl_modulus_128);
        self.scalar_weyl = ((self.scalar_weyl as u128 + weyl_jump + weyl_modulus_128) % weyl_modulus_128) as u64;
    }

    #[inline]
    fn update_t_from_matrix(&mut self, t_pow: &[u128; 128]) {
        let tm0_arr = self.tm0.as_mut_array();
        let tm1_arr = self.tm1.as_mut_array();
        for i in 0..SIMD_WIDTH {
            let t_state = (tm0_arr[i] as u128) | ((tm1_arr[i] as u128) << 64);
            let t_new = apply_mat(t_pow, t_state);
            tm0_arr[i] = t_new as u64;
            tm1_arr[i] = (t_new >> 64) as u64;
        }
    }
    const TINYMT_JUMP_MAT: [u128; 128] = compute_tinymt_mat();
    const XOSHIRO256_JUMP_MAT: [[u64; 4]; 256] = compute_xoshiro256_mat();
    // 2^128-step xoshiro matrix, precomputed
    const XOSHIRO256_JUMP_128_MAT: [[u64; 4]; 256] =
        pow_mat_256_2_exp(Self::XOSHIRO256_JUMP_MAT, 128);
    // 2^256 ≡ 1 mod (2^256 - 1), so this equals the 1-step matrix
    const XOSHIRO256_JUMP_256_MAT: [[u64; 4]; 256] =
        pow_mat_256_2_exp(Self::XOSHIRO256_JUMP_MAT, 256);
}

// ============================================================================
// Jump-ahead helpers
// ============================================================================

const fn compute_tinymt_mat() -> [u128; 128] {
    let mut res = [0; 128];
    let mut i = 0;
    while i < 128 {
        let state = 1u128 << i;
        let mut tm0 = state as u64 & TINYMT64_LANE_MASK;
        let mut tm1 = (state >> 64) as u64;
        const TINYMT_MAT1: u64 = 0xdaa51b54;
        const TINYMT_MAT2: u64 = 0xfed47fb5 << 32;
        const TINYMT64_LANE_MASK: u64 = 0x7fff_ffff_ffff_ffff_u64;

        let mut x = tm0 ^ tm1;
        x ^= x << 12;
        x ^= x >> 32;
        x ^= x << 32;
        x ^= x << 11;
        let mask = (x & 1).wrapping_neg();
        tm0 = tm1 ^ (mask & TINYMT_MAT1);
        tm1 = x ^ (mask & TINYMT_MAT2);
        res[i] = (tm0 as u128) | ((tm1 as u128) << 64);
        i += 1;
    }
    res
}

const fn apply_mat(mat: &[u128; 128], mut vec: u128) -> u128 {
    let mut res = 0;
    let mut i = 0;
    while i < 128 {
        if vec & 1 != 0 {
            res ^= mat[i];
        }
        vec >>= 1;
        i += 1;
    }
    res
}

const fn mul_mat(a: &[u128; 128], b: &[u128; 128]) -> [u128; 128] {
    let mut res = [0; 128];
    let mut i = 0;
    while i < 128 {
        res[i] = apply_mat(a, b[i]);
        i += 1;
    }
    res
}

const fn pow_mat(mut a: [u128; 128], mut n: u128) -> [u128; 128] {
    let mut res = [0; 128];
    let mut i = 0;
    while i < 128 {
        res[i] = 1 << i;
        i += 1;
    }
    while n > 0 {
        if n & 1 != 0 {
            res = mul_mat(&a, &res);
        }
        a = mul_mat(&a, &a);
        n >>= 1;
    }
    res
}

const fn pow_mat_2_exp(mut a: [u128; 128], mut exp: u32) -> [u128; 128] {
    while exp > 0 {
        a = mul_mat(&a, &a);
        exp -= 1;
    }
    a
}

// ============================================================================
// 256-bit GF(2) matrix operations (for xoshiro256)
// ============================================================================

const fn apply_mat_256(mat: &[[u64; 4]; 256], vec: [u64; 4]) -> [u64; 4] {
    let mut res = [0u64; 4];
    let mut i = 0;
    while i < 256 {
        let word = i / 64;
        let bit = i % 64;
        if (vec[word] >> bit) & 1 != 0 {
            res[0] ^= mat[i][0];
            res[1] ^= mat[i][1];
            res[2] ^= mat[i][2];
            res[3] ^= mat[i][3];
        }
        i += 1;
    }
    res
}

const fn mul_mat_256(a: &[[u64; 4]; 256], b: &[[u64; 4]; 256]) -> [[u64; 4]; 256] {
    let mut res = [[0u64; 4]; 256];
    let mut i = 0;
    while i < 256 {
        res[i] = apply_mat_256(a, b[i]);
        i += 1;
    }
    res
}

const fn pow_mat_256(mut a: [[u64; 4]; 256], mut n: u128) -> [[u64; 4]; 256] {
    // Identity matrix
    let mut res = [[0u64; 4]; 256];
    let mut i = 0;
    while i < 256 {
        let word = i / 64;
        let bit = i % 64;
        res[i][word] = 1u64 << bit;
        i += 1;
    }
    while n > 0 {
        if n & 1 != 0 {
            res = mul_mat_256(&a, &res);
        }
        a = mul_mat_256(&a, &a);
        n >>= 1;
    }
    res
}

const fn pow_mat_256_2_exp(mut a: [[u64; 4]; 256], mut exp: u32) -> [[u64; 4]; 256] {
    while exp > 0 {
        a = mul_mat_256(&a, &a);
        exp -= 1;
    }
    a
}

const fn compute_xoshiro256_mat() -> [[u64; 4]; 256] {
    let mut res = [[0u64; 4]; 256];
    let mut i = 0;
    while i < 256 {
        // Set up basis vector: bit i is set
        let word = i / 64;
        let bit = i % 64;
        let mut state = [0u64; 4];
        state[word] = 1u64 << bit;

        // Apply one step of advance_xoshiro
        let t = state[1] << 17;
        state[2] ^= state[0];
        state[3] ^= state[1];
        state[1] ^= state[2];
        state[0] ^= state[3];
        state[2] ^= t;
        state[3] = state[3].rotate_left(45);

        res[i] = state;
        i += 1;
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::BLOCK_SIZE;
    use crate::TripleMixSimdCore;
    use crate::jump::{pow_mat_2_exp, pow_mat_256_2_exp};
    use crate::reproducibility::DefaultReproducibility;
    use rand_core::Rng;

    #[test]
    fn test_add128_with_carry_comprehensive() {
        use crate::generate::Simd64;
        let zero = Simd64::splat(0);
        let one = Simd64::splat(1);
        let mask = Simd64::splat(u64::MAX);
        let almost_max = Simd64::splat(u64::MAX - 1);
        let max = Simd64::splat(u64::MAX);

        // Basic addition without carry
        let (sum, c) =
            TripleMixSimdCore::<DefaultReproducibility>::add128_with_carry(one, one, zero);
        assert_eq!(sum[0], 2);
        assert_eq!(c[0], 0);

        // Addition with carry-in
        let (sum, c) =
            TripleMixSimdCore::<DefaultReproducibility>::add128_with_carry(one, one, mask);
        assert_eq!(sum[0], 3);
        assert_eq!(c[0], 0);

        // Addition that causes carry-out
        let (sum, c) =
            TripleMixSimdCore::<DefaultReproducibility>::add128_with_carry(max, one, zero);
        assert_eq!(sum[0], 0);
        assert_eq!(c[0], u64::MAX);

        // Addition that causes carry-out via carry-in
        let (sum, c) =
            TripleMixSimdCore::<DefaultReproducibility>::add128_with_carry(almost_max, one, mask);
        assert_eq!(sum[0], 0);
        assert_eq!(c[0], u64::MAX);

        // Adding two max values with carry-in
        let (sum, c) =
            TripleMixSimdCore::<DefaultReproducibility>::add128_with_carry(max, max, mask);
        // mask is effectively +1
        // FFFFFFFF + FFFFFFFF + 1 = 2^64 + FFFFFFFF = FFFFFFFF with carry 1
        assert_eq!(sum[0], u64::MAX);
        assert_eq!(c[0], u64::MAX);
    }

    #[test]
    fn test_jump_ahead_constants_miri_slow() {
        assert_eq!(
            TripleMixSimdCore::<DefaultReproducibility>::TINYMT_JUMP_128_MAT,
            pow_mat_2_exp(
                TripleMixSimdCore::<DefaultReproducibility>::TINYMT_JUMP_MAT,
                128
            )
        );
        assert_eq!(
            TripleMixSimdCore::<DefaultReproducibility>::TINYMT_JUMP_256_MAT,
            pow_mat_2_exp(
                TripleMixSimdCore::<DefaultReproducibility>::TINYMT_JUMP_MAT,
                256
            )
        );
        assert_eq!(
            TripleMixSimdCore::<DefaultReproducibility>::XOSHIRO256_JUMP_128_MAT,
            pow_mat_256_2_exp(
                TripleMixSimdCore::<DefaultReproducibility>::XOSHIRO256_JUMP_MAT,
                128
            )
        );
        // 2^256 ≡ 1 mod (2^256 - 1), so M^(2^256) should equal M^1
        assert_eq!(
            TripleMixSimdCore::<DefaultReproducibility>::XOSHIRO256_JUMP_256_MAT,
            TripleMixSimdCore::<DefaultReproducibility>::XOSHIRO256_JUMP_MAT,
        );
    }

    #[test]
    fn test_jump_ahead_miri_xslow() {
        #[cfg(not(miri))]
        const ITERATIONS_AFTER_LEAP: usize = 10_000;
        #[cfg(not(miri))]
        const BASIC_ADVANCE_BLOCKS: u128 = 12;
        #[cfg(miri)]
        const ITERATIONS_AFTER_LEAP: usize = 4;
        #[cfg(miri)]
        const BASIC_ADVANCE_BLOCKS: u128 = 2;

        for mut prng in crate::create_rngs::<DefaultReproducibility>(7) {
            let prng_large_jmp = prng.clone();
            let mut prng_jmp = prng.clone();

            // Advance sequential by 12 steps (meaning 12 * 8 = 96 next_u64 calls)
            for _ in 0..BASIC_ADVANCE_BLOCKS {
                for _ in 0..BLOCK_SIZE {
                    prng.next_u64();
                }
            }

            // Advance jumping by BASIC_ADVANCE_BLOCKS steps
            prng_jmp.advance(BASIC_ADVANCE_BLOCKS);
            prng.block_core.reset_and_skip(0);
            prng_jmp.block_core.reset_and_skip(0);

            for _ in 0..BLOCK_SIZE {
                assert_eq!(prng.next_u64(), prng_jmp.next_u64());
            }

            let prng = prng_large_jmp.clone();

            // Test advance_2_128 and advance consistency
            let mut base_a_for_2_128 = prng.clone();
            base_a_for_2_128.advance(1u128 << 127);
            base_a_for_2_128.advance(1u128 << 127);
            let mut base_b_for_2_128 = prng.clone();
            base_b_for_2_128.advance(1);
            base_b_for_2_128.advance(u128::MAX);

            let mut prng_2_128 = prng.clone();
            prng_2_128.advance_2_128(1);

            println!("base_a_for_2_128={:?}", base_a_for_2_128);
            println!("base_b_for_2_128={:?}", base_b_for_2_128);
            println!("prng_2_128={:?}", prng_2_128);

            for _ in 0..ITERATIONS_AFTER_LEAP {
                // Ensure internal state logic lines up perfectly equivalent.
                let prng_2_128_u64 = prng_2_128.next_u64();
                assert_eq!(base_a_for_2_128.next_u64(), prng_2_128_u64);
                assert_eq!(base_b_for_2_128.next_u64(), prng_2_128_u64);
            }

            // Test advance_2_256 and advance consistency
            let mut base_a_for_2_256 = prng.clone();
            base_a_for_2_256.advance_2_128(1u128 << 127);
            base_a_for_2_256.advance_2_128(1u128 << 127);
            let mut base_b_for_2_256 = prng.clone();
            base_b_for_2_256.advance_2_128(1u128 << 127);
            base_b_for_2_256.advance_2_128(1u128 << 127);

            let mut prng_2_256 = prng.clone();
            prng_2_256.advance_2_256(1);

            for _ in 0..ITERATIONS_AFTER_LEAP {
                // Ensure internal state logic lines up perfectly equivalent.
                let prng_2_256_u64 = prng_2_256.next_u64();
                assert_eq!(base_a_for_2_256.next_u64(), prng_2_256_u64);
                assert_eq!(base_b_for_2_256.next_u64(), prng_2_256_u64);
            }
        }
    }
}
