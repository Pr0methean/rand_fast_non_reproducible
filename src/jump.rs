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
struct JumpMatrix {
    /// Multiplier part (how state is transformed)
    mult_low: Simd64,
    mult_high: Simd64,
    /// Constant part (how increment contributes)
    const_low: Simd64,
    const_high: Simd64,
}

impl JumpMatrix {
    /// Create identity matrix (jump by 0 steps)
    fn identity() -> Self {
        Self {
            mult_low: Simd64::splat(1),
            mult_high: Simd64::splat(0),
            const_low: Simd64::splat(0),
            const_high: Simd64::splat(0),
        }
    }

    /// Compose two jump matrices: this * other
    fn compose(&self, other: &Self) -> Self {
        // new_mult = self.mult * other.mult
        let (new_mult_low, new_mult_high) = mul128x128(
            self.mult_high,
            self.mult_low,
            other.mult_high,
            other.mult_low,
        );

        // new_const = self.mult * other.const + self.const
        let (temp_low, temp_high) = mul128x128(
            self.mult_high,
            self.mult_low,
            other.const_high,
            other.const_low,
        );

        let (new_const_low, carry) =
            TripleMixSimdCore::add128_with_carry(temp_low, self.const_low, Simd64::splat(0));
        let (new_const_high, _) =
            TripleMixSimdCore::add128_with_carry(temp_high, self.const_high, carry);

        Self {
            mult_low: new_mult_low,
            mult_high: new_mult_high,
            const_low: new_const_low,
            const_high: new_const_high,
        }
    }

    /// Apply this jump to a state
    fn apply(&self, state_low: Simd64, state_high: Simd64) -> (Simd64, Simd64) {
        // new_state = mult * state + const
        let (prod_low, prod_high) =
            mul128x128(self.mult_high, self.mult_low, state_high, state_low);

        let (new_low, carry) =
            TripleMixSimdCore::add128_with_carry(prod_low, self.const_low, Simd64::splat(0));
        let (new_high, _) = TripleMixSimdCore::add128_with_carry(prod_high, self.const_high, carry);

        (new_low, new_high)
    }
}

/// Full 128x128 multiplication. Returns (low, high).
fn mul128x128(a_high: Simd64, a_low: Simd64, b_high: Simd64, b_low: Simd64) -> (Simd64, Simd64) {
    let (h1, low) = TripleMixSimdCore::mul128x64to128(a_high, a_low, b_low);
    let h2 = crate::generate::simd_wrapping_mul(a_low, b_high);
    (low, h1 + h2)
}

impl TripleMixSimdCore {
    // 2^128 == 2^1 mod (2^127 - 1)
    const TINYMT_JUMP_128_MAT: [u128; 128] = pow_mat_2_exp(Self::TINYMT_JUMP_MAT, 1);
    // 2^256 == 2^2 mod (2^127 - 1)
    const TINYMT_JUMP_256_MAT: [u128; 128] = pow_mat_2_exp(Self::TINYMT_JUMP_MAT, 2);
    const XOSHIRO256_JUMP_128: [u64; 4] =             [
        0x180ec6d33cfd0aba,
        0xd5a61266f0c9392c,
        0xa9582618e03fc9aa,
        0x39abdc4529b1661c
    ];

    fn jump_pcg(&mut self, steps: u128) {
        let mut result = JumpMatrix::identity();
        let mut base = JumpMatrix {
            mult_low: Self::PCG_MULTIPLIERS,
            mult_high: Simd64::splat(0),
            const_low: self.pcg_inc_lo,
            const_high: self.pcg_inc_hi,
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
            let a = TripleMixSimdCore::MCG_MULTIPLIERS[i] as u128;
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
    
    fn jump_xoshiro_128(state: &mut [u64; 4]) {
        let mut s0 = 0;
        let mut s1 = 0;
        let mut s2 = 0;
        let mut s3 = 0;
        for j in Self::XOSHIRO256_JUMP_128 {
            for b in 0..64 {
                if (j & 1 << b) != 0 {
                    s0 ^= state[0];
                    s1 ^= state[1];
                    s2 ^= state[2];
                    s3 ^= state[3];
                }
                Self::advance_xoshiro(state);
            }
        }
        state[0] = s0;
        state[1] = s1;
        state[2] = s2;
        state[3] = s3;
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

impl TripleMixSimdCore {
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
        for _ in 0..steps {
            // FIXME: Linear-time
            Self::advance_xoshiro(&mut self.xoshiro256);
        }
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
        Self::jump_xoshiro_128(&mut self.xoshiro256);
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

        // Xoshiro256 period is 2^256 - 1, so each multiple of 2^256 is 1 full period (no-op) + 1 step
        for _ in 0..multiples {
            // FIXME: Linear-time
            Self::advance_xoshiro(&mut self.xoshiro256);
        }
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

#[cfg(test)]
mod tests {
    use crate::TripleMixSimdCore;
    use crate::BLOCK_SIZE;
    use crate::jump::pow_mat_2_exp;
    use crate::reproducibility::NotReproducible;
    use rand_core::Rng;

    #[test]
    fn test_jump_ahead_constants() {
        assert_eq!(
            TripleMixSimdCore::TINYMT_JUMP_128_MAT,
            pow_mat_2_exp(TripleMixSimdCore::TINYMT_JUMP_MAT, 128)
        );
        assert_eq!(
            TripleMixSimdCore::TINYMT_JUMP_256_MAT,
            pow_mat_2_exp(TripleMixSimdCore::TINYMT_JUMP_MAT, 256)
        );
    }

    #[test]
    fn test_jump_ahead() {
        for mut prng in crate::create_rngs::<NotReproducible>() {
            let prng_large_jmp = prng.clone();
            let mut prng_jmp = prng.clone();

            // Advance sequential by 12 steps (meaning 12 * 8 = 96 next_u64 calls)
            for _ in 0..12 {
                for _ in 0..BLOCK_SIZE {
                    prng.next_u64();
                }
            }

            // Advance jumping by 12 steps
            prng_jmp.advance(12);
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

            for _ in 0..10_000 {
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

            for _ in 0..10_000 {
                // Ensure internal state logic lines up perfectly equivalent.
                let prng_2_256_u64 = prng_2_256.next_u64();
                assert_eq!(base_a_for_2_256.next_u64(), prng_2_256_u64);
                assert_eq!(base_b_for_2_256.next_u64(), prng_2_256_u64);
            }
        }
    }
}
