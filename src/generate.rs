#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
))]
use crate::avx2;
use crate::{BLOCK_SIZE, TripleMixSimdCore};
use core::simd::Simd;
use core::simd::cmp::SimdPartialOrd;
use core::simd::num::SimdInt;
use core::simd::num::SimdUint;
use core::slice::from_mut;
use rand_core::block::Generator;
use crate::reproducibility::Reproducibility;
use core::marker::PhantomData;

impl<R: Reproducibility> TripleMixSimdCore<R> {
    const TINYMT_MAT1: u64 = 0xdaa51b54;
    const TINYMT_MAT2: u64 = 0xfed47fb5 << 32;
    const TINYMT_TMAT: u64 = 0xa853e7ffeffefffe;

    // The MCG multiplier for each lane i is (1u128 << 64) - LANE_CONSTANTS[i];
    pub(crate) const MULTIPLIER_COMPLEMENT_0: u64 = 742;
    pub(crate) const MULTIPLIER_COMPLEMENT_1: u64 = 5572;
    pub(crate) const MULTIPLIER_COMPLEMENT_2: u64 = 1432;
    pub(crate) const MULTIPLIER_COMPLEMENT_3: u64 = 1108;
    pub(crate) const MWC_MULTIPLIER_COMPLEMENTS: Simd64 = Simd64::from_array([
        Self::MULTIPLIER_COMPLEMENT_0,
        Self::MULTIPLIER_COMPLEMENT_1,
        Self::MULTIPLIER_COMPLEMENT_2,
        Self::MULTIPLIER_COMPLEMENT_3,
    ]);
    pub(crate) const MCG_MULTIPLIERS: Simd64 = Simd64::from_array([
        u64::MAX - Self::MULTIPLIER_COMPLEMENT_0 + 1,
        u64::MAX - Self::MULTIPLIER_COMPLEMENT_1 + 1,
        u64::MAX - Self::MULTIPLIER_COMPLEMENT_2 + 1,
        u64::MAX - Self::MULTIPLIER_COMPLEMENT_3 + 1,
    ]);

    const PCG_MULT_LO_0: u64 = 0x1FC6_5DA5;
    const PCG_MULT_HI_0: u64 = 0x2360_ED05;
    const PCG_MULT_LO_1: u64 = 0x4C95_7F2D;
    const PCG_MULT_HI_1: u64 = 0x8F1C_5E95;
    const PCG_MULT_LO_2: u64 = 0xE5A7_4D29;
    const PCG_MULT_HI_2: u64 = 0xA3E7_9B3D;
    const PCG_MULT_LO_3: u64 = 0x9B3C_D8F1;
    const PCG_MULT_HI_3: u64 = 0x2360_ED05;

    const PCG_MULT_LO: Simd64 = Simd64::from_array([
        Self::PCG_MULT_LO_0,
        Self::PCG_MULT_LO_1,
        Self::PCG_MULT_LO_2,
        Self::PCG_MULT_LO_3,
    ]);
    const PCG_MULT_HI: Simd64 = Simd64::from_array([
        Self::PCG_MULT_HI_0,
        Self::PCG_MULT_HI_1,
        Self::PCG_MULT_HI_2,
        Self::PCG_MULT_HI_3,
    ]);

    pub(crate) const PCG_MULTIPLIERS: Simd64 = Simd64::from_array([
        (Self::PCG_MULT_HI_0 << 32) | Self::PCG_MULT_LO_0,
        (Self::PCG_MULT_HI_1 << 32) | Self::PCG_MULT_LO_1,
        (Self::PCG_MULT_HI_2 << 32) | Self::PCG_MULT_LO_2,
        (Self::PCG_MULT_HI_3 << 32) | Self::PCG_MULT_LO_3,
    ]);

    /// Multiplies two vectors. Requires that all elements of b be less than 2^32. Returns (low, hi)
/// halves of result.
#[inline(always)]
    pub(crate) fn simd_mulsmall(a: Simd64, b: Simd64) -> (Simd64, Simd64) {
    debug_assert!(b.simd_lt(Simd::splat(1 << 32)).all());
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
    ))]
    {
        avx2::mul_small(a, b)
    }
    #[cfg(not(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
    )))]
    {
        let a_lo = a & Simd64::splat(0xffffffff);
        let a_hi = a >> 32;

        let p0 = a_lo * b;
        let p1 = a_hi * b;

        let lo = (p1 << 32) + p0;
        let mask = lo.simd_lt(p0);
        let carry = mask.to_simd().cast::<u64>();
        let hi = (p1 >> 32) - carry;

        (lo, hi)
    }
}

    /// 128-bit addition with carry: (result, carry_out) = a + b - carry_in.cast::<i64>()
    #[inline(always)]
    pub(crate) fn add128_with_carry(a: Simd64, b: Simd64, carry_in: Simd64) -> (Simd64, Simd64) {
        let sum1 = a + b;
        let sum = sum1 - carry_in;

        // carry_mask is either 0 or -1 (all bits 1)
        let carry_mask = (sum1.simd_lt(a) | sum.simd_lt(sum1)).to_simd().cast();

        (sum, carry_mask)
    }

    pub(crate) fn almost_all_zeroes_core() -> TripleMixSimdCore<R> {
        const SMALLEST_2BIT_POSITIVE: [u64; SIMD_WIDTH] = [3, 5, 7, 9];
        TripleMixSimdCore {
            pcg_state_lo: Simd::splat(0),
            pcg_state_hi: Simd::splat(0),
            pcg_inc_lo: Simd::from_array(SMALLEST_2BIT_POSITIVE),
            pcg_inc_hi: Simd::splat(0),
            tm0: Simd::splat(0),
            tm1: Simd64::from_array(SMALLEST_2BIT_POSITIVE),
            mwc_state: Simd64::from_array(SMALLEST_2BIT_POSITIVE),
            mwc_carry: Simd::splat(0),
            xoshiro256: [0, 0, 0, 1],
            reproducibility: PhantomData,
        }
    }

    #[inline(always)]
    pub(crate) fn fill_blocks(&mut self, blocks: &mut [[u64; BLOCK_SIZE]]) {
        if blocks.is_empty() {
            return;
        }

        let pcg_inc_lo = self.pcg_inc_lo;
        let pcg_inc_hi = self.pcg_inc_hi;
        let i_mixed = pcg_inc_hi + pcg_inc_lo;
        let mut pcg_state_lo = self.pcg_state_lo;
        let mut pcg_state_hi = self.pcg_state_hi;
        let mut tm0 = self.tm0;
        let mut tm1 = self.tm1;
        let mut mwc_state = self.mwc_state;
        let mut mwc_carry = self.mwc_carry;
        let mut xoshiro256 = self.xoshiro256;

        const PCG_OUTPUT_MULTIPLIERS: Simd64 = Simd::from_array([
            0xd6e8feb86659fd93,
            0x881cf9e71fbdd5b9,
            0xbf58476d1ce4e5b9,
            0x94d049bb133111eb,
        ]);

        for block in blocks {
            // Kick off the highest latency operations (multipliers) early
            // a_low * b (where b is pcg_mult)
            let (pcg_p1_lo, pcg_p1_hi) = Self::simd_mulsmall(pcg_state_lo, Self::PCG_MULT_LO);
            let (pcg_p2_lo, pcg_p2_hi) = Self::simd_mulsmall(pcg_state_lo, Self::PCG_MULT_HI);
            // a_high * b = a_high * b_lo + a_high * b_hi * 2^32
            let pcg_a_high_b = simd_wrapping_mul(pcg_state_hi, Self::PCG_MULTIPLIERS);

            // p2 * 2^32 = p2_hi * 2^96 + p2_lo * 2^32
            let pcg_p2_shifted_lo = pcg_p2_lo << Simd64::splat(32);
            let pcg_p2_shifted_hi = (pcg_p2_hi << Simd64::splat(32)) | (pcg_p2_lo >> Simd64::splat(32));

            // low sum = p1_lo + p2_shifted_lo
            let (pcg_prod_lo, pcg_carry) = Self::add128_with_carry(pcg_p1_lo, pcg_p2_shifted_lo, Simd64::splat(0));
            let pcg_a_low_b_hi = pcg_p1_hi + pcg_p2_shifted_hi - pcg_carry;

            let pcg_prod_hi = pcg_a_low_b_hi + pcg_a_high_b;

            // Finish PCG state transition
            let (pcg_next_state_lo, pcg_carry) =
                Self::add128_with_carry(pcg_prod_lo, pcg_inc_lo, Simd::splat(0));
            let (pcg_next_state_hi, _) =
                Self::add128_with_carry(pcg_prod_hi, pcg_inc_hi, pcg_carry);
            pcg_state_lo = pcg_next_state_lo;
            pcg_state_hi = pcg_next_state_hi;
            let pcg_x = pcg_state_hi ^ pcg_state_lo;
            let pcg_m = simd_wrapping_mul(pcg_x ^ (pcg_x >> 31), PCG_OUTPUT_MULTIPLIERS);
            let (mwc_kx_lo, mwc_kx_hi) = Self::simd_mulsmall(mwc_state, Self::MWC_MULTIPLIER_COMPLEMENTS);
            let pcg_rot = pcg_x >> 59;
            let pcg_output = (pcg_m >> pcg_rot) | (pcg_m << (Simd64::splat(64) - pcg_rot));

            let mwc_borrow = mwc_carry.simd_lt(mwc_kx_lo).to_simd().cast::<u64>();
            let mwc_next_state = mwc_carry - mwc_kx_lo;
            mwc_carry = (mwc_state - mwc_kx_hi) + mwc_borrow;
            mwc_state = mwc_next_state;

            // TinyMT Step 0: Mask and initial XOR
            let tm0_masked = tm0 & Simd::splat(TINYMT64_LANE_MASK);
            let mut tm_x = tm0_masked ^ tm1;
            let tm_y = tm0_masked + tm1;

            // TinyMT Step 1: First shift
            tm_x ^= tm_x << Simd::splat(12);

            // Generate scalar xoshiro256** output
            let xoshiro_out = xoshiro256[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);

            // TinyMT Step 2: Second shift
            tm_x ^= tm_x >> Simd::splat(32);

            Self::advance_xoshiro(&mut xoshiro256);

            // TinyMT Step 3: Third shift
            tm_x ^= tm_x << Simd::splat(32);

            // TinyMT Step 4: Fourth shift
            tm_x ^= tm_x << Simd::splat(11);

            // TinyMT Step 5: Final output and transition prep
            let tm_out =
                tm_y ^ ((tm_y & Simd::splat(1)).wrapping_neg() & Simd::splat(Self::TINYMT_TMAT));
            let tm_mask = (tm_x & Simd::splat(1)).wrapping_neg();
            tm0 = tm1 ^ (tm_mask & Simd::splat(Self::TINYMT_MAT1));
            tm1 = tm_x ^ (tm_mask & Simd::splat(Self::TINYMT_MAT2));
            let tm_secondary_out = tm0 - tm1;

            let (x, y, z) = TripleMixSimdCore::<R>::mix(
                mwc_state,
                pcg_output,
                tm_out,
                mwc_carry,
                i_mixed,
                pcg_state_lo,
                tm_secondary_out,
                xoshiro_out,
            );

            x.copy_to_slice(&mut block[0..4]);
            y.copy_to_slice(&mut block[4..8]);
            z.copy_to_slice(&mut block[8..12]);
        }

        self.pcg_state_lo = pcg_state_lo;
        self.pcg_state_hi = pcg_state_hi;
        self.tm0 = tm0;
        self.tm1 = tm1;
        self.mwc_state = mwc_state;
        self.mwc_carry = mwc_carry;
        self.xoshiro256 = xoshiro256;
    }

    #[inline(always)]
    pub(crate) fn advance_xoshiro(xoshiro256: &mut [u64; 4]) {
        let t = xoshiro256[1] << 17;
        xoshiro256[2] ^= xoshiro256[0];
        xoshiro256[3] ^= xoshiro256[1];
        xoshiro256[1] ^= xoshiro256[2];
        xoshiro256[0] ^= xoshiro256[3];

        xoshiro256[2] ^= t;

        xoshiro256[3] = xoshiro256[3].rotate_left(45);
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn mix(
        x0: Simd64,
        x1: Simd64,
        x2: Simd64,
        x3: Simd64,
        x4: Simd64,
        x5: Simd64,
        x6: Simd64,
        scalar1: u64,
    ) -> (Simd64, Simd64, Simd64) {
        // Convert inputs to u32x8 (portable)
        let xi = [
            R::simd64_as_simd32(x0),
            R::simd64_as_simd32(x1),
            R::simd64_as_simd32(x2),
            R::simd64_as_simd32(x3),
            R::simd64_as_simd32(x4),
            R::simd64_as_simd32(x5),
            R::simd64_as_simd32(x6),
        ];

        // Rotation helper
        #[inline(always)]
        fn rotl32(x: Simd32, k: u32) -> Simd32 {
            (x << Simd32::splat(k)) | (x >> Simd32::splat(32 - k))
        }

        #[allow(clippy::too_many_arguments)]
        #[inline(always)]
        fn round3<R: Reproducibility>(
            mut a: Simd32,
            mut b: Simd32,
            mut c: Simd32,
            x: &[Simd32; 7],
            shift1: u32,
            shift2: u32,
            shift3: u32,
        ) -> (Simd32, Simd32, Simd32) {
            // --- First nonlinear layer ---
            let (m0_lo, m0_hi, m1_lo, m1_hi) = TripleMixSimdCore::<R>::mul_lo_hi_triad(a, b, c);

            a ^= b.rotate_elements_left::<1>();
            b += c.rotate_elements_right::<2>();
            c ^= a.rotate_elements_left::<3>();

            // --- Input injection ---
            a += x[0];
            b ^= x[1];
            c += x[2];

            a ^= m1_hi + b.rotate_elements_right::<2>();
            b ^= m0_lo ^ c.rotate_elements_left::<3>();
            c ^= m0_hi + a.rotate_elements_right::<1>();

            a ^= x[3];
            b += x[4];
            c ^= x[5];

            // --- Rotate ---
            a = rotl32(a, shift1);
            b ^= x[6];
            c ^= rotl32(b, shift3);
            b += m1_lo + a.rotate_elements_right::<4>();
            a += rotl32(c, shift2);

            (a, b, c)
        }
        let scalar_hi = (scalar1 >> 32) as u32;
        let scalar_lo = scalar1 as u32;
        let mut a = Simd32::splat(0x243f6a88);
        let scalar_mix_1 = Simd32::from_array([0, scalar_lo, scalar_hi, 0, scalar_hi, 0, scalar_lo, 0]);
        let mut b = Simd32::splat(0x9e3779b9);
        let scalar_mix_2 = Simd32::from_array([scalar_hi, 0, 0, scalar_hi, 0, scalar_lo, 0, scalar_lo]);
        a ^= scalar_mix_1;
        let mut c = Simd32::splat(0xb7e15162);
        b += scalar_mix_2;
        c += scalar_mix_1;
        (a, b, c) = round3::<R>(a, b, c, &xi, 7, 25, 11);
        (b, c, a) = round3::<R>(
            b,
            c,
            a,
            &[xi[3], xi[4], xi[5], xi[6], xi[0], xi[1], xi[2]],
            5,
            17,
            9,
        );
        (c, a, b) = round3::<R>(
            c,
            a,
            b,
            &[xi[5], xi[2], xi[6], xi[1], xi[4], xi[0], xi[3]],
            3,
            13,
            23
        );

        // --- Strong final cross-lane avalanche ---
        a ^= b.rotate_elements_right::<2>();
        b += c.rotate_elements_left::<3>();
        c += a.rotate_elements_right::<1>();

        // Convert back to u64x4 by casting and packing
        (
            R::simd32_as_simd64(a),
            R::simd32_as_simd64(b),
            R::simd32_as_simd64(c),
        )
    }
}

/// SIMD multiply: uses AVX2 mullo (in-register) or portable * (scalarized).
#[inline(always)]
pub(crate) fn simd_wrapping_mul(a: Simd64, b: Simd64) -> Simd64 {
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
    ))]
    {
        avx2::wrapping_mul(a, b)
    }
    #[cfg(not(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        not(all(target_feature = "avx512dq", target_feature = "avx512vl"))
    )))]
    {
        a * b
    }
}

pub(crate) const TINYMT64_LANE_MASK: u64 = 0x7fff_ffff_ffff_ffff_u64;
pub(crate) const SIMD_WIDTH: usize = 4;
pub(crate) const MIX_OUTPUTS: usize = 3;

pub(crate) type Simd64 = Simd<u64, SIMD_WIDTH>;
pub(crate) type Simd32 = Simd<u32, { SIMD_WIDTH * 2 }>;

    impl<R: Reproducibility> Generator for TripleMixSimdCore<R> {
        type Output = [u64; BLOCK_SIZE];

        #[inline(always)]
        fn generate(&mut self, output: &mut Self::Output) {
            self.fill_blocks(from_mut(output))
        }
    }

#[cfg(test)]
mod tests {
    use crate::generate::{MIX_OUTPUTS, SIMD_WIDTH, Simd64};
    use crate::reproducibility::{DefaultReproducibility, NotReproducible, Reproducibility};
    use crate::{BLOCK_SIZE, TripleMixPrng, TripleMixSimdCore, rng};
    use bytemuck::cast_slice_mut;
    use core::simd::Simd;
    use core::simd::cmp::SimdPartialEq;
    use core::simd::num::SimdUint;
    use fsum::FSum;
    use gf2::{BitMatrix, BitStore, BitVector};
    use hypors::chi_square::goodness_of_fit;
    use itertools::Itertools;
    #[cfg(not(miri))]
    use proptest::{prelude::any, prop_assert, proptest};
    use rand::RngExt;
    use rand_core::{Rng, SeedableRng};
    use statrs::distribution::{Binomial, Discrete, DiscreteCDF};

    const MIX_INPUTS: usize = 7;
    const MIX_INPUT_U64S: usize = SIMD_WIDTH * MIX_INPUTS + 1;

    struct MixMatrixStats {
        total_weight: usize,
        min_row_weight: usize,
        min_col_weight: usize,
        rank: usize,
    }

    const AVALANCHE_MATRIX_ROWS: usize = 8 * size_of::<Simd64>() * MIX_OUTPUTS;
    const AVALANCHE_MATRIX_COLS: usize = 8 * size_of::<u64>() * MIX_INPUT_U64S;

    fn evaluate_mix_matrix(mix_input: [u64; MIX_INPUT_U64S]) -> MixMatrixStats {
        let (base_out0, base_out1, base_out2) = mix_from_flat_array(mix_input);
        let mut xor_matrix = BitMatrix::<u64>::zeros(AVALANCHE_MATRIX_ROWS, AVALANCHE_MATRIX_COLS);
        let mut i = 0;
        for variable_idx in 0..(MIX_INPUTS + 1) {
            for lane_idx in 0..SIMD_WIDTH {
                if variable_idx == MIX_INPUTS && lane_idx > 0 {
                    break;
                }
                for bit_idx in 0..64 {
                    let mut modified_input = mix_input.clone();
                    modified_input[variable_idx * 4 + lane_idx] ^= 1u64 << bit_idx;
                    let (mod_out0, mod_out1, mod_out2) = mix_from_flat_array(modified_input);
                    let (out_xor_0, out_xor_1, out_xor_2) = (
                        mod_out0 ^ base_out0,
                        mod_out1 ^ base_out1,
                        mod_out2 ^ base_out2,
                    );
                    let mut j = 0;
                    for out_lane_idx in 0..SIMD_WIDTH {
                        for out_bit_idx in 0..64 {
                            xor_matrix.set(j, i, (out_xor_0[out_lane_idx] >> out_bit_idx) & 1 != 0);
                            j += 1;
                        }
                        for out_bit_idx in 0..64 {
                            xor_matrix.set(j, i, (out_xor_1[out_lane_idx] >> out_bit_idx) & 1 != 0);
                            j += 1;
                        }
                        for out_bit_idx in 0..64 {
                            xor_matrix.set(j, i, (out_xor_2[out_lane_idx] >> out_bit_idx) & 1 != 0);
                            j += 1;
                        }
                    }
                    i += 1;
                }
            }
        }
        let mut input_matrix = BitMatrix::<u64>::zeros(MIX_INPUT_U64S, 64);
        for (index, input) in mix_input.iter().copied().enumerate() {
            input_matrix.set_row(index, &BitVector::from_unsigned(input));
        }
        let rank = input_matrix.to_echelon_form().count_ones();
        #[cfg(not(miri))]
        assert_eq!(
            xor_matrix.clone().to_echelon_form().count_ones(),
            AVALANCHE_MATRIX_ROWS
        );
        let row_weights = (0..AVALANCHE_MATRIX_ROWS)
            .map(|row| xor_matrix.row(row).count_ones())
            .collect::<Vec<_>>();
        let min_row_weight = row_weights.iter().copied().min().unwrap();
        let max_row_weight = row_weights.iter().copied().max().unwrap();
        let col_weights = (0..AVALANCHE_MATRIX_COLS)
            .map(|col| xor_matrix.col(col).count_ones())
            .collect::<Vec<_>>();
        let min_col_weight = col_weights.iter().copied().min().unwrap();
        let max_col_weight = col_weights.iter().copied().max().unwrap();
        println!("min_row_weight={min_row_weight}, max_row_weight={max_row_weight}");
        println!("Row weights:");
        for row_chunk in row_weights.chunks_exact(64) {
            println!("{:>4?} = {:>6}", row_chunk, row_chunk.iter().sum::<usize>());
        }
        println!("min_col_weight={min_col_weight}, max_col_weight={max_col_weight}");
        println!("Column weights:");
        for col_chunk in col_weights.chunks_exact(64) {
            println!("{:>4?} = {:>6}", col_chunk, col_chunk.iter().sum::<usize>());
        }
        let total_weight = row_weights.into_iter().sum::<usize>();
        println!("Total weight: {total_weight}");
        MixMatrixStats {
            total_weight,
            min_row_weight,
            min_col_weight,
            rank
        }
    }

    fn mix_from_flat_array(
        mix_input: [u64; MIX_INPUT_U64S],
    ) -> (Simd64, Simd64, Simd64) {
        let input_simds = [
            Simd::from_array(mix_input[0..4].try_into().unwrap()),
            Simd::from_array(mix_input[4..8].try_into().unwrap()),
            Simd::from_array(mix_input[8..12].try_into().unwrap()),
            Simd::from_array(mix_input[12..16].try_into().unwrap()),
            Simd::from_array(mix_input[16..20].try_into().unwrap()),
            Simd::from_array(mix_input[20..24].try_into().unwrap()),
            Simd::from_array(mix_input[24..28].try_into().unwrap()),
        ];
        let (base_out0, base_out1, base_out2) = TripleMixSimdCore::<DefaultReproducibility>::mix(
            input_simds[0],
            input_simds[1],
            input_simds[2],
            input_simds[3],
            input_simds[4],
            input_simds[5],
            input_simds[6],
            mix_input[28],
        );
        (base_out0, base_out1, base_out2)
    }

    struct SecondDerivativeStats {
        min: u64,
        max: u64,
        mean: f64,
        stdev: f64,
    }

    fn evaluate_second_order_derivatives(
        mix_input: [u64; MIX_INPUT_U64S],
    ) -> SecondDerivativeStats {
        let (base_out0, base_out1, base_out_2) = mix_from_flat_array(mix_input);
        let mut weights = Vec::new();
        for var_idx_1 in 0..MIX_INPUTS {
            for var_idx_2 in var_idx_1..MIX_INPUTS {
                for lane_idx_1 in 0..SIMD_WIDTH {
                    if var_idx_1 == 8 && lane_idx_1 > 0 {
                        break;
                    }
                    for lane_idx_2 in lane_idx_1..SIMD_WIDTH {
                        if var_idx_2 == 8 && lane_idx_2 > 0 {
                            break;
                        }
                        if lane_idx_1 == lane_idx_2 && var_idx_1 == var_idx_2 && !cfg!(miri) {
                            for bit_idx_1 in 0..63 {
                                for bit_idx_2 in bit_idx_1..64 {
                                    let mut modified_input = mix_input;
                                    modified_input[var_idx_1 * 4 + lane_idx_1] ^=
                                        1 << bit_idx_1 | 1 << bit_idx_2;
                                    let (mod_out0, mod_out1, mod_out2) =
                                        mix_from_flat_array(modified_input);
                                    let (out_xor_0, out_xor_1, out_xor_2) = (
                                        mod_out0 ^ base_out0,
                                        mod_out1 ^ base_out1,
                                        mod_out2 ^ base_out_2,
                                    );
                                    let weight = out_xor_0.count_ones().reduce_sum()
                                        + out_xor_1.count_ones().reduce_sum()
                                        + out_xor_2.count_ones().reduce_sum();
                                    if weight < (96 * MIX_OUTPUTS) as u64 {
                                        println!(
                                            "Low-weight second derivative: {weight} (var_idx_1={var_idx_1}, var_idx_2={var_idx_2}, lane_idx_1={lane_idx_1}, lane_idx_2={lane_idx_2}, bit_idx_1={bit_idx_1}, bit_idx_2={bit_idx_2})"
                                        );
                                    }
                                    weights.push(weight);
                                }
                            }
                        } else {
                            for bit_idx in 0..64 {
                                let mut modified_input = mix_input;
                                modified_input[var_idx_1 * 4 + lane_idx_1] ^= 1 << bit_idx;
                                modified_input[var_idx_2 * 4 + lane_idx_2] ^= 1 << bit_idx;
                                let (mod_out0, mod_out1, mod_out2) =
                                    mix_from_flat_array(modified_input);
                                let (out_xor_0, out_xor_1, out_xor_2) = (
                                    mod_out0 ^ base_out0,
                                    mod_out1 ^ base_out1,
                                    mod_out2 ^ base_out_2,
                                );
                                let weight = out_xor_0.count_ones().reduce_sum()
                                    + out_xor_1.count_ones().reduce_sum()
                                    + out_xor_2.count_ones().reduce_sum();
                                if weight < (96 * MIX_OUTPUTS) as u64 {
                                    println!(
                                        "Low-weight second derivative: {weight} (var_idx_1={var_idx_1}, var_idx_2={var_idx_2}, lane_idx_1={lane_idx_1}, lane_idx_2={lane_idx_2}, bit_idx={bit_idx})"
                                    );
                                }
                                weights.push(weight);
                            }
                        }
                    }
                }
            }
        }
        let sample_size = weights.len();
        let min = weights.iter().copied().min().unwrap();
        let max = weights.iter().copied().max().unwrap();
        let mean = weights.iter().copied().map(u64::from).sum::<u64>() as f64 / sample_size as f64;
        let variance_weight = weights
            .iter()
            .copied()
            .map(|weight| weight as f64 - mean)
            .map(|x| x * x)
            .sum::<f64>()
            / (sample_size - 1) as f64;
        let stdev = variance_weight.sqrt();
        println!("N={sample_size}, min={min}, max={max}, mean={mean}, sd={stdev}");
        SecondDerivativeStats {
            min,
            max,
            mean,
            stdev,
        }
    }

    #[cfg(not(miri))]
    const RANDOM_INPUT_ITERATIONS: usize = 10;
    #[cfg(miri)]
    const RANDOM_INPUT_ITERATIONS: usize = 2;

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_mix_matrix_random_inputs() {
        let mut rng = rng();
        let mut mix_input = [0u64; MIX_INPUT_U64S];
        let sigma = ((AVALANCHE_MATRIX_ROWS * AVALANCHE_MATRIX_COLS) as f64 * 0.25 - 1.0).sqrt();
        let mut total_deviation = 0isize;
        let grand_sigma =
            ((AVALANCHE_MATRIX_ROWS * AVALANCHE_MATRIX_COLS * RANDOM_INPUT_ITERATIONS) as f64 * 0.25 - 1.0)
                .sqrt();
        for _ in 0..RANDOM_INPUT_ITERATIONS {
            rng.fill(&mut mix_input);
            let MixMatrixStats {
                total_weight,
                min_row_weight,
                min_col_weight,
                rank,
            } = evaluate_mix_matrix(mix_input);
            let deviation = 0isize
                .checked_add_unsigned(total_weight)
                .unwrap()
                .checked_sub_unsigned((AVALANCHE_MATRIX_ROWS * AVALANCHE_MATRIX_COLS) / 2)
                .unwrap();
            total_deviation += deviation;
            let z = (deviation as f64) / sigma;
            assert!(
                min_col_weight >= (AVALANCHE_MATRIX_ROWS * 3) / 8,
                "Min column weight {min_col_weight} too low (rank {rank} input)"
            );
            assert!(
                min_row_weight >= (AVALANCHE_MATRIX_COLS * 3) / 8,
                "Min row weight {min_row_weight} too low (rank {rank} input)"
            );
            assert!(z >= -4.0, "Total weight {total_weight} (z={z}) too low (rank {rank} input)");
            assert!(z <= 4.0, "Total weight {total_weight} (z={z}) too high (rank {rank} input)");
        }
        let z = (total_deviation as f64) / grand_sigma;
        assert!(
            z >= -3.0,
            "Total deviation {total_deviation} (z={z}) too low"
        );
        assert!(
            z <= 3.0,
            "Total deviation {total_deviation} (z={z}) too high"
        );
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_second_derivative_random_inputs() {
        let mut rng = rng();
        let mut random_inputs = [0u64; MIX_INPUT_U64S];
        for _ in 0..RANDOM_INPUT_ITERATIONS {
            rng.fill(&mut random_inputs);
            let SecondDerivativeStats {
                min,
                max,
                mean,
                stdev,
            } = evaluate_second_order_derivatives(random_inputs);
            assert!(min >= (MIX_OUTPUTS as u64 * 96), "Min weight {min} too low");
            assert!(
                max <= (MIX_OUTPUTS as u64 * 160),
                "Max weight {max} too high"
            );
            assert!(
                mean >= (MIX_OUTPUTS as f64 * 127.0),
                "Mean weight {mean:.02} too low"
            );
            assert!(
                mean <= (MIX_OUTPUTS as f64 * 129.0),
                "Mean weight {mean:.02} too high"
            );
            assert!(stdev >= 11.0, "Stdev weight {stdev:.02} too low");
            assert!(stdev <= 14.1, "Stdev weight {stdev:.02} too high");
        }
    }

    #[cfg(not(miri))]
    proptest! {
        #[test]
        fn test_mix_matrix_proptest(mix_input: [u64; MIX_INPUT_U64S]) {
            let MixMatrixStats { total_weight, min_row_weight, min_col_weight, rank } =
                evaluate_mix_matrix(mix_input);
            prop_assert!(min_col_weight >= (AVALANCHE_MATRIX_ROWS * 3) / 8);
            prop_assert!(min_row_weight >= (AVALANCHE_MATRIX_COLS * 3) / 8);
            let expected = AVALANCHE_MATRIX_ROWS * AVALANCHE_MATRIX_COLS / 2;
            let deviation = total_weight as isize - expected as isize;
            let sigma = ((AVALANCHE_MATRIX_ROWS * AVALANCHE_MATRIX_COLS) as f64 * 0.25 - 1.0).sqrt();
            let z = (deviation as f64) / sigma;
            prop_assert!(z >= -5.0, "Total weight {total_weight} (z={z}) too low (rank {rank} input)");
            prop_assert!(z <= 5.0, "Total weight {total_weight} (z={z}) too high (rank {rank} input)");
        }

        #[test]
        fn test_second_derivative_proptest(mix_input: [u64; MIX_INPUT_U64S]) {
            let SecondDerivativeStats { min, max, mean, stdev } = evaluate_second_order_derivatives(mix_input);
            assert!(min as usize >= (AVALANCHE_MATRIX_ROWS * 5) / 16, "Min weight {min} too low");
            assert!(max as usize <= (AVALANCHE_MATRIX_ROWS * 11) / 16, "Max weight {max} too high");
            assert!(mean >= 0.49 * AVALANCHE_MATRIX_ROWS as f64, "Mean weight {mean:.02} too low");
            assert!(mean <= 0.51 * AVALANCHE_MATRIX_ROWS as f64, "Mean weight {mean:.02} too high");
            assert!(stdev >= 11.0, "Stdev weight {stdev:.02} too low");
            assert!(stdev <= 14.2, "Stdev weight {stdev:.02} too high");
        }

        #[test]
        fn test_simd_mulsmall_proptest(a in any::<[u64; 4]>(), b in any::<[u32; 4]>()) {
            let a_simd = Simd64::from_array(a);
            let b_u64 = b.map(|x| x as u64);
            let b_simd = Simd64::from_array(b_u64);
            let (lo, hi) = TripleMixSimdCore::<DefaultReproducibility>::simd_mulsmall(a_simd, b_simd);
            let lo_arr = lo.to_array();
            let hi_arr = hi.to_array();

            for i in 0..4 {
                let expected = (a[i] as u128) * (b_u64[i] as u128);
                let actual = (lo_arr[i] as u128) | ((hi_arr[i] as u128) << 64);
                assert_eq!(actual, expected, "simd_mulsmall failed for a={} b={}", a[i], b_u64[i]);
            }
        }
    }

    #[test]
    fn test_byte_frequencies() {
        #[cfg(not(miri))]
        const N: usize = 1 << 28;
        #[cfg(miri)]
        const N: usize = 1 << 12;
        for mut prng in crate::create_rngs::<NotReproducible>() {
            let mut frequencies = [0u32; u8::MAX as usize + 1];
            for _ in 0..N {
                let byte: u8 = prng.random();
                frequencies[byte as usize] += 1;
            }
            let chi_square = goodness_of_fit(
                frequencies.map(f64::from),
                core::iter::repeat_n((N >> 8) as f64, u8::MAX as usize + 1),
                0.005,
            )
            .unwrap();
            println!("{:?}", chi_square);
            assert!(!chi_square.reject_null);
        }
    }

    #[test]
    fn test_u16_frequencies() {
        #[cfg(not(miri))]
        const N: usize = 1 << 28;
        #[cfg(miri)]
        const N: usize = 1 << 12;
        for mut prng in crate::create_rngs::<NotReproducible>() {
            let mut frequencies = vec![0u32; u16::MAX as usize + 1];
            for _ in 0..N {
                let word: u16 = prng.random();
                frequencies[word as usize] += 1;
            }
            let chi_square = goodness_of_fit(
                frequencies.into_iter().map(f64::from),
                core::iter::repeat_n((N as f64)/((1 >> 16) as f64), u16::MAX as usize + 1),
                0.005,
            )
            .unwrap();
            println!("{:?}", chi_square);
            assert!(!chi_square.reject_null);
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_bit_correlations_and_transitions() {
        #[cfg(not(miri))]
        const N: usize = 1 << 22;
        #[cfg(miri)]
        const N: usize = 1 << 11;
        const CHUNK_SIZE: usize = 1 << 11;
        const CHUNK_COUNT: usize = N / CHUNK_SIZE;
        #[cfg(not(miri))]
        const P_THRESHOLD: f64 = 1e-6;
        for mut prng in crate::create_rngs::<NotReproducible>() {
            // Flatten to 2D for better cache locality
            let mut bins = [[0u32; 4]; 64 * 64];
            let mut lagged_bins = [[0u32; 4]; 64 * 64];
            // Process in a cache-friendly order
            let mut chunk = [0u64; CHUNK_SIZE + 1];
            chunk[0] = prng.next_u64();
            for _ in 0..CHUNK_COUNT {
                prng.fill_bytes(cast_slice_mut(&mut chunk[1..]));
                for i in 0..64 {
                    for j in 0..64 {
                        let row_index = j * 64 + i;
                        let nonlagged_row = &mut bins[row_index];
                        let lagged_row = &mut lagged_bins[row_index];
                        for [first, second] in chunk.array_windows().copied() {
                            let double_ith_bit_of_second = ((second >> i) & 1) << 1;
                            let nonlagged_bin =
                                (((second >> j) & 1) | double_ith_bit_of_second) as usize;
                            let lagged_bin =
                                (((first >> j) & 1) | double_ith_bit_of_second) as usize;

                            nonlagged_row[nonlagged_bin] += 1;
                            lagged_row[lagged_bin] += 1;
                        }
                    }
                }
                chunk[0] = chunk[CHUNK_SIZE - 1];
            }

            // Testing phase - convert back to 3D view for readability
            #[cfg(not(miri))]
            for i in 0..64 {
                for j in 0..64 {
                    let idx = j * 64 + i;

                    if j > i {
                        let p = goodness_of_fit(
                            bins[idx].map(f64::from),
                            [N as f64 * 0.25; 4],
                            P_THRESHOLD,
                        )
                        .unwrap()
                        .p_value;
                        assert!(
                            p >= P_THRESHOLD,
                            "Chi-square test failed for bins: ({:?}, p={p:.10}) for i={i},j={j}",
                            bins[idx]
                        );
                    }

                    let p = goodness_of_fit(
                        lagged_bins[idx].map(f64::from),
                        [(N - 1) as f64 * 0.25; 4],
                        P_THRESHOLD,
                    )
                    .unwrap()
                    .p_value;
                    assert!(
                        p >= P_THRESHOLD,
                        "Chi-square test failed for lagged bins: ({:?}, p={p:.10}) for i={i},j={j}",
                        lagged_bins[idx]
                    );
                }
            }

            #[cfg(miri)]
            {
                core::hint::black_box(bins);
            }
        }
    }

    #[test]
    fn test_avalanche_miri_slow() {
        const LOW_AVALANCHE_THRESHOLD: u64 = 28 * BLOCK_SIZE as u64;
        println!("Low-avalanche threshold: {LOW_AVALANCHE_THRESHOLD} bits");
        let mut total_low_avalanche_checks = 0;
        let mut total_checks = 0;
        let bit_flip_distribution = Binomial::new(0.5, (BLOCK_SIZE * 64) as u64).unwrap();
        let low_avalanche_probability = bit_flip_distribution.cdf(LOW_AVALANCHE_THRESHOLD);
        for rng in crate::create_rngs::<NotReproducible>() {
            let core = rng.block_core.core;

            #[cfg(not(miri))]
            const ITERATIONS: usize = 20;
            #[cfg(miri)]
            const ITERATIONS: usize = 2;

            let mut min_flips = u64::MAX;
            let mut max_flips = 0;
            let mut total_flips: u64 = 0;
            let mut count: u64 = 0;
            let mut flips_per_bit = [[[0; 64]; SIMD_WIDTH]; 9];
            let mut core1 = core;
            let mut output1 = [[Simd64::splat(0); MIX_OUTPUTS]; ITERATIONS];
            core1.fill_blocks(cast_slice_mut(&mut output1));
            let mut min_field = 0;
            let mut min_lane = 0;
            let mut min_bit = 0;
            let mut min_iter = 0;
            let mut low_avalanches = 0;
            for (field_idx, flips_per_bit_for_field) in flips_per_bit.iter_mut().enumerate() {
                for (lane_idx, flips_per_bit_for_lane) in
                    flips_per_bit_for_field.iter_mut().enumerate()
                {
                    for (bit_idx, flips_for_bit) in flips_per_bit_for_lane.iter_mut().enumerate() {
                        if field_idx == 4 && bit_idx == 63 {
                            continue;
                        }
                        let mut core2 = core;
                        match field_idx {
                            0 => {
                                let x = core2.pcg_state_lo;
                                let mut arr = x.to_array();
                                arr[lane_idx] ^= 1 << bit_idx;
                                core2.pcg_state_lo = Simd64::from_array(arr);
                            }
                            1 => {
                                let x = core2.pcg_state_hi;
                                let mut arr = x.to_array();
                                arr[lane_idx] ^= 1 << bit_idx;
                                core2.pcg_state_hi = Simd64::from_array(arr);
                            }
                            2 => {
                                let x = core2.pcg_inc_lo;
                                let mut arr = x.to_array();
                                arr[lane_idx] ^= 1 << bit_idx;
                                core2.pcg_inc_lo = Simd64::from_array(arr);
                            }
                            3 => {
                                let x = core2.pcg_inc_hi;
                                let mut arr = x.to_array();
                                arr[lane_idx] ^= 1 << bit_idx;
                                core2.pcg_inc_hi = Simd64::from_array(arr);
                            }
                            4 => {
                                let x = core2.tm0;
                                let mut arr = x.to_array();
                                arr[lane_idx] ^= 1 << bit_idx;
                                core2.tm0 = Simd64::from_array(arr);
                            }
                            5 => {
                                let x = core2.tm1;
                                let mut arr = x.to_array();
                                arr[lane_idx] ^= 1 << bit_idx;
                                core2.tm1 = Simd64::from_array(arr);
                            }
                            6 => {
                                let x = core2.mwc_state;
                                let mut arr = x.to_array();
                                arr[lane_idx] ^= 1 << bit_idx;
                                core2.mwc_state = Simd64::from_array(arr);
                            }
                            7 => {
                                let x = core2.mwc_carry;
                                let mut arr = x.to_array();
                                arr[lane_idx] ^= 1 << bit_idx;
                                core2.mwc_carry = Simd64::from_array(arr);
                            }
                            8 => {
                                core2.xoshiro256[lane_idx] ^= 1 << bit_idx;
                            }
                            _ => unreachable!(),
                        }
                        if !core2.is_valid() {
                            continue;
                        }
                        let mut output2 = [[Simd64::splat(0); MIX_OUTPUTS]; ITERATIONS];
                        core2.fill_blocks(cast_slice_mut(&mut output2));
                        for i in 0..ITERATIONS {
                            if field_idx == 8 && i < 4 {
                                // xoshiro256 takes 4 blocks to propagate
                                continue;
                            }
                            let mut flips = 0;
                            let first_output1 = Simd::splat(output1[i][0][0]);
                            let first_output2 = Simd::splat(output2[i][0][0]);
                            for vec_idx in 0..MIX_OUTPUTS {
                                let xor = output1[i][vec_idx] ^ output2[i][vec_idx];
                                let sub_same = (output1[i][vec_idx] - output2[i][vec_idx])
                                    .simd_eq(first_output1 - first_output2);
                                let xor_same = xor.simd_eq(first_output1 ^ first_output2);
                                for cell in 0..SIMD_WIDTH {
                                    if vec_idx == 0 && cell == 0 {
                                        // This is the baseline for comparisons
                                        continue;
                                    }
                                    assert_eq!(
                                        sub_same.test(cell),
                                        false,
                                        "Field {field_idx}, lane {lane_idx}, bit {bit_idx}, iter {i}: Same difference between cells 0 and {cell} as before flipping"
                                    );
                                    assert_eq!(
                                        xor_same.test(cell),
                                        false,
                                        "Field {field_idx}, lane {lane_idx}, bit {bit_idx}, iter {i}: Same xor between cells 0 and {cell} as before flipping"
                                    );
                                }
                                flips += xor.count_ones().reduce_sum();
                            }
                            total_flips += flips;
                            if flips <= LOW_AVALANCHE_THRESHOLD {
                                low_avalanches += 1;
                            }
                            if flips < min_flips {
                                min_flips = flips;
                                min_iter = i;
                                min_field = field_idx;
                                min_lane = lane_idx;
                                min_bit = bit_idx;
                            }
                            max_flips = max_flips.max(flips);
                            count += 1;
                            *flips_for_bit += flips;
                        }
                    }
                }
            }
            for (field_idx, flips_per_bit_for_field) in flips_per_bit.iter().enumerate() {
                for (lane_idx, flips_per_bit_for_lane) in flips_per_bit_for_field.iter().enumerate()
                {
                    println!(
                        "Field {} lane {}: Flips: {:?}",
                        field_idx, lane_idx, flips_per_bit_for_lane
                    );
                }
            }
            let avg_flips = total_flips as f64 / count as f64;
            println!(
                "Avalanche stats ({} checks): Avg: {:.2}, Min: {}, Max: {}",
                count, avg_flips, min_flips, max_flips
            );

            const DEVIATION: f64 = 0.1;
            assert!(
                avg_flips >= 32.0 * (1.0 - DEVIATION) * (BLOCK_SIZE as f64),
                "Average diffusion too low"
            );
            assert!(
                avg_flips <= 32.0 * (1.0 + DEVIATION) * (BLOCK_SIZE as f64),
                "Average diffusion too high?"
            );

            let low_avalanche_p_value =
                binomial_p_value(low_avalanche_probability, count, low_avalanches);
            println!(
                "Expected {:.4} low-avalanche checks, got {}; p={:.4}",
                count as f64 * low_avalanche_probability,
                low_avalanches,
                low_avalanche_p_value
            );
            assert!(
                low_avalanche_p_value > 0.001,
                "Too many low-avalanche results. Worst offender: Field {min_field} lane {min_lane} bit {min_bit} on iteration {min_iter} with {min_flips} flips."
            );
            assert!(
                min_flips as usize >= 16 * BLOCK_SIZE,
                "Minimum diffusion too low in field {min_field} lane {min_lane} bit {min_bit} on iteration {min_iter}, possible blind spot!"
            );
            total_checks += count;
            total_low_avalanche_checks += low_avalanches;
        }
        let low_avalanche_p_value = binomial_p_value(
            low_avalanche_probability,
            total_checks,
            total_low_avalanche_checks,
        );
        println!(
            "Expected {:.4} low-avalanche checks, got {}; p={:.4}",
            total_checks as f64 * low_avalanche_probability,
            total_low_avalanche_checks,
            low_avalanche_p_value
        );
        assert!(
            low_avalanche_p_value > 0.01,
            "Too many low-avalanche results"
        );
    }

    fn binomial_p_value(probability: f64, trials: u64, successes: u64) -> f64 {
        let low_avalanche_distribution = Binomial::new(probability, trials).unwrap();
        let p_obs = low_avalanche_distribution.pmf(successes);

        // Sum all outcomes whose probability is <= the probability of our observation
        let low_avalanche_p_value: f64 = (0..=trials)
            .map(|k| low_avalanche_distribution.pmf(k))
            .sorted_by(f64::total_cmp)
            .take_while(|&p| p <= p_obs + 1e-12) // + epsilon for float safety
            .sum();

        // Ensure it doesn't exceed 1.0 due to floating point errors
        let low_avalanche_p_value = low_avalanche_p_value.min(1.0);
        low_avalanche_p_value
    }

    mod projection {
        use crate::create_rngs;
        use crate::reproducibility::NotReproducible;
        use bytemuck::cast_slice_mut;
        use rand_core::Rng;

        fn xor_successive(words: &mut [u64]) {
            for i in 0..words.len() - 1 {
                words[i] ^= words[i + 1];
            }
        }

        fn random_projection_kernel() -> [[i8; PROJECTION_BLOCK]; PROJECTION_BLOCK] {
            // Fixed deterministic ±1 kernel
            let mut k = [[0i8; PROJECTION_BLOCK]; PROJECTION_BLOCK];
            let mut x: u64 = 0x12345678abcdef01;
            for row in k.iter_mut() {
                for cell in row.iter_mut() {
                    x ^= x << 13;
                    x ^= x >> 7;
                    x ^= x << 17;
                    *cell = if x & 1 == 0 { 1 } else { -1 };
                }
            }
            k
        }

        fn extract_bitplane(words: &[u64], bit: u32) -> Vec<i8> {
            words
                .iter()
                .map(|w| if ((w >> bit) & 1) != 0 { 1 } else { -1 })
                .collect()
        }

        fn projection_test(data: &[i8]) -> f64 {
            let kernel = random_projection_kernel();
            let mut sum = 0f64;
            let mut sum_sq = 0f64;
            let mut count = 0f64;

            let side = (data.len() as f64).sqrt() as usize;
            for y in 0..side - PROJECTION_BLOCK {
                for x in 0..side - PROJECTION_BLOCK {
                    let mut acc = 0i32;
                    for (ky, kernel_row) in kernel.iter().enumerate() {
                        for (kx, kernel_entry) in kernel_row.iter().copied().enumerate() {
                            let idx = (y + ky) * side + (x + kx);
                            acc += data[idx] as i32 * kernel_entry as i32;
                        }
                    }
                    let val = acc as f64;
                    sum += val;
                    sum_sq += val * val;
                    count += 1.0;
                }
            }

            let mean = sum / count;
            let var = (sum_sq / count) - mean * mean;
            mean.abs() + (var - 64.0).abs() // 64 expected variance for 8x8 ±1
        }
        const PROJECTION_BLOCK: usize = 8; // 8x8 projection
        fn test_bitplane_projection_generic(xor_with_next: bool) {
            #[cfg(not(miri))]
            const N: usize = 1 << 22;
            #[cfg(miri)]
            const N: usize = 1 << 8;
            #[cfg(not(miri))]
            const MAX_SCORE: f64 = 1.0;
            #[cfg(miri)]
            const MAX_SCORE: f64 = 10.0;
            for mut rng in create_rngs::<NotReproducible>() {
                let mut buf = vec![0u64; N];
                rng.fill_bytes(cast_slice_mut(&mut buf));

                if xor_with_next {
                    xor_successive(&mut buf);
                }
                for bit in 0..64 {
                    let plane = extract_bitplane(&buf, bit);
                    let score = projection_test(&plane);

                    assert!(
                        score < MAX_SCORE,
                        "Projection deviation too large for bit {bit}: {}",
                        score
                    );
                }
            }
        }

        #[test]
        fn test_bitplane_projection_xornext_miri_xslow() {
            test_bitplane_projection_generic(true);
        }

        #[test]
        fn test_bitplane_projection_miri_xslow() {
            test_bitplane_projection_generic(false);
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_lane_cross_correlation_bitplane() {
        for mut rng in crate::create_rngs::<NotReproducible>() {
            #[cfg(not(miri))]
            const N: usize = 1 << 27;
            #[cfg(miri)]
            const N: usize = 1 << 10;
            let mut lanes = Simd64::splat(0);
            for target_lane in 1..SIMD_WIDTH {
                let mut sums = [0i64; 64];
                for _ in 0..N {
                    rng.fill_bytes(cast_slice_mut(lanes.as_mut_array()));
                    for (bit, sum) in sums.iter_mut().enumerate() {
                        let a = if (lanes[0] >> bit) & 1 == 1 { 1 } else { -1 };
                        let b = if (lanes[target_lane] >> bit) & 1 == 1 {
                            1
                        } else {
                            -1
                        };

                        *sum += (a * b) as i64;
                    }
                }
                for (bit, sum) in sums.into_iter().enumerate() {
                    let corr = sum as f64 / N as f64;

                    // For the binomial distribution, stddev = sqrt(N * p * (1 - p))
                    // but its range is [0, N]; we've scaled it linearly to have the range [-1, 1]
                    // so sigma = sqrt(N * 0.25) * 2 / N
                    // = 1/sqrt(N)
                    let sigma = 1.0 / (N as f64).sqrt();

                    assert!(
                        corr.abs() < 5.0 * sigma,
                        "Lane bit correlation detected on bit {bit} betweeen lanes 0 and {target_lane}: {corr} (σ={sigma})",
                    );
                }
            }
        }
    }

    fn gf2_rank(mut rows: [u64; 64]) -> u32 {
        let mut rank = 0;
        for col in (0..64).rev() {
            if let Some(pivot) = (rank..64).find(|&r| (rows[r] >> col) & 1 == 1) {
                rows.swap(rank, pivot);
                for r in 0..64 {
                    if r != rank && ((rows[r] >> col) & 1) == 1 {
                        rows[r] ^= rows[rank];
                    }
                }
                rank += 1;
            }
        }
        rank.try_into().unwrap()
    }

    /// False positive rate for this test is about 1.2% per PRNG.
    #[test]
    fn test_lowbit_rank() {
        #[cfg(not(miri))]
        const N: usize = 10000;
        #[cfg(miri)]
        const N: usize = 64;

        for mut rng in crate::create_rngs::<NotReproducible>() {
            let mut rank60_count = 0;

            for _ in 0..N {
                let mut matrix = [0u64; 64];
                rng.fill_bytes(cast_slice_mut(&mut matrix));
                let rank = gf2_rank(matrix);
                assert!(rank >= 60, "Low-bit rank deficiency: {}", rank);
                if rank == 60 {
                    rank60_count += 1;
                    assert!(
                        rank60_count <= 2,
                        "Too many low-bit rank deficiencies (rank 60)"
                    );
                }
            }
        }
    }

    #[test]
    fn test_double_differential() {
        for mut rng in crate::create_rngs::<NotReproducible>() {
            #[cfg(not(miri))]
            const N: usize = 1 << 21;
            #[cfg(miri)]
            const N: usize = 1 << 14;

            let mut x = vec![0u64; N];
            rng.fill_bytes(cast_slice_mut(&mut x));

            // first difference
            for i in 0..N - 1 {
                x[i] ^= x[i + 1];
            }

            // second difference
            for i in 0..N - 2 {
                x[i] ^= x[i + 2];
            }

            // check bit bias
            let ones = x.iter().map(|v| v.count_ones() as u64).sum::<u64>();
            let total_bits = (N as u64) * 64;
            let bias = (ones as f64 / total_bits as f64) - 0.5;

            assert!(bias.abs() < 1e-3, "Differential bias detected: {}", bias);
        }
    }

    #[test]
    fn test_fractional_spectral() {
        for mut rng in crate::create_rngs::<NotReproducible>() {
            #[cfg(not(miri))]
            const N: usize = 1 << 21;
            #[cfg(miri)]
            const N: usize = 1 << 10;

            let mut prev = rng.next_u64();
            let mut min_gap = f64::MAX;

            for _ in 0..N {
                let curr = rng.next_u64();
                let diff = (curr.wrapping_sub(prev) as f64).abs();
                if diff < min_gap {
                    min_gap = diff;
                }
                prev = curr;
            }

            assert!(min_gap > 1.0, "Spectral lattice behavior suspected");
        }
    }

    /// Configuration for matrix construction
    struct MatrixConfig<R: Reproducibility> {
        /// Number of output steps to consider (should be 3 for full 1536-bit output)
        steps: usize,
        /// Base state to use (must be valid)
        base_state: TripleMixSimdCore<R>,
    }

    /// Build transition matrix from state bits to output bits
    fn build_transition_matrix<R: Reproducibility>(
        config: &MatrixConfig<R>,
    ) -> (BitMatrix<u64>, Vec<String>) {
        let state_bits = 9 * SIMD_WIDTH * 64; // 9 fields × 4 lanes × 64 bits = 2304 bits

        let output_bits = config.steps * BLOCK_SIZE * 64; // steps × 8 words × 64 bits

        let mut matrix = BitMatrix::<u64>::zeros(output_bits, state_bits);
        let mut column_labels = Vec::with_capacity(state_bits);

        // Define fields and their accessors
        let fields: &[(
            &str,
            fn(&mut TripleMixSimdCore<R>) -> &mut [u64; 4],
        )] = &[
            (
                "pcg_state_lo",
                |c| c.pcg_state_lo.as_mut_array(),
            ),
            (
                "pcg_state_hi",
                |c| c.pcg_state_hi.as_mut_array(),
            ),
            (
                "pcg_inc_lo",
                |c| c.pcg_inc_lo.as_mut_array(),
            ),
            (
                "pcg_inc_hi",
                |c| c.pcg_inc_hi.as_mut_array(),
            ),
            (
                "tm0",
                |c| c.tm0.as_mut_array(),
            ),
            (
                "tm1",
                |c| c.tm1.as_mut_array(),
            ),
            (
                "mwc_state",
                |c| c.mwc_state.as_mut_array(),
            ),
            (
                "mwc_carry",
                |c| c.mwc_carry.as_mut_array(),
            ),
            (
                "xoshiro256",
                |c| &mut c.xoshiro256,
            ),
        ];

        // Generate base outputs once
        let mut base_outputs = vec![[0u64; BLOCK_SIZE]; config.steps];
        let mut base_core = config.base_state.clone();
        base_core.fill_blocks(&mut base_outputs);

        let mut col_idx = 0;

        for (field_name, mut_field) in fields {

            for lane in 0..SIMD_WIDTH {
                for bit in 0..64 {
                    column_labels.push(format!("{}.lane{}.bit{}", field_name, lane, bit));

                    // Create state with this bit flipped
                    let mut flipped_state = config.base_state.clone();
                    mut_field(&mut flipped_state)[lane] ^= 1 << bit;

                    // Generate outputs from flipped state
                    let mut flipped_outputs = vec![[0u64; BLOCK_SIZE]; config.steps];
                    let mut flipped_core = flipped_state;
                    flipped_core.fill_blocks(&mut flipped_outputs);

                    // Record differences
                    for step in 0..config.steps {
                        for word in 0..BLOCK_SIZE {
                            let diff = base_outputs[step][word] ^ flipped_outputs[step][word];
                            for out_bit in 0..64 {
                                if (diff >> out_bit) & 1 == 1 {
                                    let row = step * BLOCK_SIZE * 64 + word * 64 + out_bit;
                                    matrix.set(row, col_idx, true);
                                }
                            }
                        }
                    }

                    col_idx += 1;
                }
            }
        }

        assert_eq!(
            col_idx, state_bits,
            "Should have exactly {} columns",
            state_bits
        );
        (matrix, column_labels)
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_4_step_matrix_rank_distribution() {
        #[cfg(feature = "no_std")]
        extern crate alloc;

        let mut rng = rng();
        let mut ranks = Vec::new();
        let iterations = 1000;
        #[cfg(miri)]
        let iterations = 1;

        for _ in 0..iterations {
            let base_state = TripleMixPrng::<DefaultReproducibility>::from_rng(&mut rng)
                .block_core
                .core;
            let config = MatrixConfig {
                steps: 4,
                base_state,
            };

            let (mut matrix, _) = build_transition_matrix(&config);
            let echelon = matrix.to_echelon_form();
            let rank = echelon.count_ones();
            ranks.push(rank);
        }
        ranks.sort_unstable();
        // Calculate statistics
        let mean_rank = ranks.iter().sum::<usize>() as f64 / iterations as f64;
        println!("Rank distribution over {} trials:", iterations);
        println!("  Mean: {:.2}", mean_rank);
        if iterations > 1 {
            println!("  Min: {}", ranks.iter().min().unwrap());
            println!("  Max: {}", ranks.iter().max().unwrap());
            let variance = FSum::with_all(ranks.iter().map(|&r| (r as f64 - mean_rank).powi(2)))
                .value()
                / ((iterations - 1) as f64);
            let std_dev = variance.sqrt();
            println!("  Std dev: {:.2}", std_dev);
            assert!(std_dev <= 2.0, "Too much variation: {:.2}", std_dev);
        }
        // Create histogram
        #[cfg(not(feature = "no_std"))]
        let mut hist = std::collections::HashMap::new();
        #[cfg(feature = "no_std")]
        let mut hist = alloc::collections::BTreeMap::new();
        for &rank in &ranks {
            *hist.entry(rank).or_insert(0) += 1;
        }

        let mut hist_vec: Vec<_> = hist.into_iter().collect();
        hist_vec.sort();
        for (rank, count) in hist_vec {
            println!(
                "  Rank {}: {} trials ({:.1}%)",
                rank,
                count,
                100.0 * count as f64 / iterations as f64
            );
        }

        assert!(mean_rank >= 2296.0, "Mean rank too low: {:.2}", mean_rank);
    }
}
