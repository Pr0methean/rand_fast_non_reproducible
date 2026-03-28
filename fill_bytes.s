<rand_core::block::BlockRng<rand_triplemix::TripleMixSimdCore<rand_triplemix::reproducibility::NotReproducible>>>::fill_bytes:
		// %userprofile%\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\rand_core-0.10.0\src\block.rs:275
		pub fn fill_bytes(&mut self, dest: &mut [u8]) {
	push rsi
	push rdi
	sub rsp, 360
	vmovdqa xmmword ptr [rsp + 336], xmm15
	vmovdqa xmmword ptr [rsp + 320], xmm14
	vmovdqa xmmword ptr [rsp + 304], xmm13
	vmovdqa xmmword ptr [rsp + 288], xmm12
	vmovdqa xmmword ptr [rsp + 272], xmm11
	vmovdqa xmmword ptr [rsp + 256], xmm10
	vmovdqa xmmword ptr [rsp + 240], xmm9
	vmovdqa xmmword ptr [rsp + 224], xmm8
	vmovdqa xmmword ptr [rsp + 208], xmm7
	vmovdqa xmmword ptr [rsp + 192], xmm6
	mov rax, rdx
	mov rsi, rcx
		// %userprofile%\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\rand_core-0.10.0\src\block.rs:175
		self.results[0].into_usize()
	mov rdi, qword ptr [rcx]
		// %userprofile%\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\rand_core-0.10.0\src\block.rs:279
		if index >= N {
	cmp rdi, 11
	jbe .LBB11_2
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vmovdqa ymm1, ymmword ptr [rsi + 256]
	vmovdqa ymm4, ymmword ptr [rsi + 128]
	vmovdqa ymm3, ymmword ptr [rsi + 160]
	vmovdqa ymm2, ymmword ptr [rsi + 192]
	vmovdqa ymm5, ymmword ptr [rsi + 224]
	vmovdqa ymm6, ymmword ptr [rsi + 320]
	vmovdqa ymm7, ymmword ptr [rsi + 288]
	vpaddq ymm0, ymm6, ymm7
	vmovdqu ymmword ptr [rsp + 96], ymm0
	vmovdqa ymm8, ymmword ptr [rip + __ymm@000000009b3cd8f100000000e5a74d29000000004c957f2d000000001fc65da5]
	vpmuludq ymm9, ymm8, ymm5
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm10, ymm9, 32
	vpsrlq ymm11, ymm5, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm12, ymm11, ymm8
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:84
		unsafe { transmute(simd_add(a.as_i64x4(), b.as_i64x4())) }
	vpaddq ymm10, ymm10, ymm12
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm12, ymm10, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2315
		unsafe { transmute(simd_or(a.as_i32x8(), b.as_i32x8())) }
	vpblendd ymm9, ymm9, ymm12, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:56
		core::intrinsics::simd::$simd_call(
	vmovdqa ymm12, ymmword ptr [rip + __ymm@000000002360ed0500000000a3e79b3d000000008f1c5e95000000002360ed05]
	vpmuludq ymm5, ymm12, ymm5
	vpsllq ymm13, ymm5, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddq ymm13, ymm9, ymm13
	vpaddq ymm0, ymm13, ymm7
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm14, ymm8, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddq ymm6, ymm14, ymm6
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm14, ymm1, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm8, ymm14, ymm8
	vpmuludq ymm1, ymm12, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:84
		unsafe { transmute(simd_add(a.as_i64x4(), b.as_i64x4())) }
	vpaddq ymm1, ymm8, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm1, ymm1, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddq ymm1, ymm6, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm6, ymm10, 32
	vpsrlq ymm5, ymm5, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm8, ymm11, ymm12
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:84
		unsafe { transmute(simd_add(a.as_i64x4(), b.as_i64x4())) }
	vpaddq ymm5, ymm8, ymm5
	vpaddq ymm5, ymm6, ymm5
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddq ymm1, ymm1, ymm5
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\select.rs:68
		core::intrinsics::simd::simd_select(mask, true_values, false_values)
	vpbroadcastq ymm5, qword ptr [rip + __real@8000000000000000]
	vpxor ymm6, ymm9, ymm5
	vpxor ymm8, ymm13, ymm5
	vpcmpgtq ymm6, ymm6, ymm8
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpsubq ymm1, ymm1, ymm6
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\select.rs:68
		core::intrinsics::simd::simd_select(mask, true_values, false_values)
	vpxor ymm6, ymm0, ymm5
	vpcmpgtq ymm6, ymm8, ymm6
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpsubq ymm8, ymm1, ymm6
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vmovdqa ymm1, ymmword ptr [rip + __ymm@0000000000000454000000000000059800000000000015c400000000000002e6]
	vpmuludq ymm6, ymm3, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm9, ymm6, 32
	vpsrlq ymm10, ymm3, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm1, ymm10, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:84
		unsafe { transmute(simd_add(a.as_i64x4(), b.as_i64x4())) }
	vpaddq ymm9, ymm9, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm1, ymm9, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2315
		unsafe { transmute(simd_or(a.as_i32x8(), b.as_i32x8())) }
	vpbroadcastq ymm10, qword ptr [rip + __real@00000000fffffffe]
	vpand ymm6, ymm10, ymm6
	vpor ymm6, ymm1, ymm6
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpsubq ymm1, ymm2, ymm6
	vmovdqu ymmword ptr [rsp + 64], ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm9, ymm9, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpsubq ymm3, ymm3, ymm9
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\simd\cmp\ord.rs:57
		unsafe { Mask::from_simd_unchecked(core::intrinsics::simd::simd_lt(self, other)) }
	vpxor ymm6, ymm6, ymm5
	vpxor ymm2, ymm2, ymm5
	vpcmpgtq ymm2, ymm6, ymm2
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddq ymm7, ymm3, ymm2
	vpbroadcastq ymm3, qword ptr [rip + __real@7fffffffffffffff]
	vpand ymm6, ymm3, ymmword ptr [rsi + 96]
	vpxor ymm3, ymm6, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:56
		core::intrinsics::simd::$simd_call(
	vpsllq ymm5, ymm3, 12
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm3, ymm5, ymm3
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:56
		core::intrinsics::simd::$simd_call(
	vpsrlq ymm5, ymm3, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm3, ymm5, ymm3
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\select.rs:68
		core::intrinsics::simd::simd_select(mask, true_values, false_values)
	vpbroadcastq ymm5, qword ptr [rip + __real@0000000000000001]
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpand ymm5, ymm3, ymm5
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpxor xmm11, xmm11, xmm11
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops\unary.rs:15
		unsafe { core::intrinsics::simd::simd_neg(self) }
	vpsubq ymm5, ymm11, ymm5
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpbroadcastq ymm9, qword ptr [rip + __real@00000000daa51b54]
	vpand ymm9, ymm9, ymm5
	vpxor ymm9, ymm9, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:56
		core::intrinsics::simd::$simd_call(
	vpsllq ymm10, ymm3, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm3, ymm10, ymm3
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:56
		core::intrinsics::simd::$simd_call(
	vpsllq ymm10, ymm3, 11
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm10, ymm10, ymm3
	vpbroadcastq ymm3, qword ptr [rip + __real@fed47fb500000000]
	vpand ymm3, ymm5, ymm3
	vpxor ymm12, ymm10, ymm3
	vpxor ymm1, ymm8, ymm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:56
		core::intrinsics::simd::$simd_call(
	vpsrlq ymm3, ymm1, 31
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm3, ymm3, ymm1
	vmovdqu ymmword ptr [rsp + 160], ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vmovdqa ymm13, ymmword ptr [rip + __ymm@00000000133111eb000000001ce4e5b9000000001fbdd5b9000000006659fd93]
	vpmuludq ymm14, ymm13, ymm3
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm15, ymm3, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm13, ymm15, ymm13
	vpmuludq ymm3, ymm3, ymmword ptr [rip + __ymm@0000000094d049bb00000000bf58476d00000000881cf9e700000000d6e8feb8]
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:84
		unsafe { transmute(simd_add(a.as_i64x4(), b.as_i64x4())) }
	vpaddq ymm3, ymm13, ymm3
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm3, ymm3, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:84
		unsafe { transmute(simd_add(a.as_i64x4(), b.as_i64x4())) }
	vpaddq ymm3, ymm14, ymm3
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:56
		core::intrinsics::simd::$simd_call(
	vpsrlq ymm13, ymm1, 59
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpsubq ymm14, ymm11, ymm13
	vpbroadcastq ymm15, qword ptr [rip + __real@000000000000003f]
	vpand ymm14, ymm14, ymm15
	vpsllvq ymm14, ymm3, ymm14
	vpsrlvq ymm3, ymm3, ymm13
	vpor ymm2, ymm14, ymm3
	vmovdqu ymmword ptr [rsp + 128], ymm2
	vpaddq ymm4, ymm6, ymm4
	vpsllq ymm6, ymm4, 63
	vpcmpgtq ymm6, ymm11, ymm6
	vpbroadcastq ymm11, qword ptr [rip + __real@a853e7ffeffefffe]
	vpand ymm6, ymm11, ymm6
	vpxor ymm5, ymm6, ymm4
		// src\generate.rs:242
		self.pcg_state_lo = pcg_state_lo;
	vmovdqa ymmword ptr [rsi + 224], ymm0
		// src\generate.rs:243
		self.pcg_state_hi = pcg_state_hi;
	vmovdqa ymmword ptr [rsi + 256], ymm8
		// src\generate.rs:244
		self.tm0 = tm0;
	vmovdqa ymmword ptr [rsi + 96], ymm9
		// src\generate.rs:245
		self.tm1 = tm1;
	vmovdqa ymmword ptr [rsi + 128], ymm12
	vmovdqu ymm3, ymmword ptr [rsp + 64]
		// src\generate.rs:246
		self.mwc_state = mwc_state;
	vmovdqa ymmword ptr [rsi + 160], ymm3
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddq ymm6, ymm10, ymm4
		// src\generate.rs:247
		self.mwc_carry = mwc_carry;
	vmovdqa ymmword ptr [rsi + 192], ymm7
	vmovdqa xmm4, xmmword ptr [rsi + 352]
		// src\generate.rs:255
		xoshiro256[3] ^= xoshiro256[1];
	vpxor xmm8, xmm4, xmmword ptr [rsi + 368]
	mov rcx, qword ptr [rsi + 360]
	vpshufd xmm9, xmm8, 78
		// src\generate.rs:257
		xoshiro256[0] ^= xoshiro256[3];
	vpxor xmm0, xmm9, xmm4
	vmovdqa xmmword ptr [rsp + 48], xmm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\num\uint_macros.rs:2533
		intrinsics::wrapping_mul(self, rhs)
	lea r10, [rcx + 4*rcx]
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\intrinsics\mod.rs:2029
		unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
	vpextrq r9, xmm8, 1
		// src\generate.rs:259
		xoshiro256[2] ^= t;
	vmovq rdx, xmm8
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\intrinsics\mod.rs:2029
		unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
	rorx r10, r10, 57
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\num\uint_macros.rs:2533
		intrinsics::wrapping_mul(self, rhs)
	lea r10, [r10 + 8*r10]
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\ptr\mod.rs:551
		unsafe { crate::intrinsics::copy_nonoverlapping(src, dst, count) }
	vmovd xmm8, r10d
		// src\generate.rs:333
		let scalar_hi = (scalar >> 32) as u32;
	shr r10, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\ptr\mod.rs:551
		unsafe { crate::intrinsics::copy_nonoverlapping(src, dst, count) }
	vmovd xmm9, r10d
	vpunpcklqdq xmm10, xmm9, xmm8
	vshufps xmm11, xmm8, xmm9, 65
	vinsertf128 ymm10, ymm11, xmm10, 1
	vpshufd xmm9, xmm9, 20
	vpshufb xmm8, xmm8, xmmword ptr [rip + __xmm@03020100808080800302010080808080]
	vinserti128 ymm8, ymm9, xmm8, 1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vbroadcastss ymm9, dword ptr [rip + __real@243f6a88]
	vxorps ymm12, ymm10, ymm9
	vpbroadcastd ymm9, dword ptr [rip + __real@9e3779b9]
	vpaddd ymm8, ymm8, ymm9
	vpbroadcastd ymm9, dword ptr [rip + __real@b7e15162]
	vpaddd ymm10, ymm10, ymm9
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm9, ymm8, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm11, ymm8, ymm12
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm13, ymm12, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm13, ymm9, ymm13
	vpmuludq ymm14, ymm8, ymm10
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm15, ymm10, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm9, ymm9, ymm15
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm15, ymm13, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm15, ymm11, ymm15, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm11, ymm11, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm13, ymm11, ymm13, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm11, ymm9, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm11, ymm14, ymm11, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm14, ymm14, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm14, ymm14, ymm9, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vmovdqa ymm9, ymmword ptr [rip + __ymm@0000000000000007000000060000000500000004000000030000000200000001]
	vxorps xmm4, xmm4, xmm4
	vpermd ymm4, ymm9, ymm8
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm4, ymm12, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm12, xmm12, xmm12
	vpermq ymm12, ymm10, 147
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm12, ymm8, ymm12
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vmovdqa ymm8, ymmword ptr [rip + __ymm@0000000200000001000000000000000700000006000000050000000400000003]
	vxorps xmm0, xmm0, xmm0
	vpermd ymm0, ymm8, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm0, ymm10, ymm0
	vpxor ymm12, ymm12, ymm2
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm10, xmm10, xmm10
	vpermq ymm10, ymm12, 57
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm10, ymm10, ymm14
	vpaddd ymm4, ymm4, ymm3
	vpxor ymm4, ymm10, ymm4
	vpaddd ymm0, ymm0, ymm5
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vmovdqa ymm1, ymmword ptr [rip + __ymm@0000000400000003000000020000000100000000000000070000000600000005]
	vxorps xmm14, xmm14, xmm14
	vpermd ymm14, ymm1, ymm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm14, ymm15, ymm14
	vpxor ymm12, ymm14, ymm12
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm14, xmm14, xmm14
	vpermd ymm14, ymm9, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm13, ymm14, ymm13
	vpxor ymm4, ymm4, ymm7
	vpsrld ymm14, ymm4, 25
	vpslld ymm4, ymm4, 7
	vpor ymm4, ymm14, ymm4
	vmovdqu ymm2, ymmword ptr [rsp + 96]
	vpaddd ymm12, ymm12, ymm2
	vpxor ymm12, ymm12, ymm6
	vpsrld ymm14, ymm12, 21
	vpslld ymm15, ymm12, 11
	vpor ymm14, ymm15, ymm14
	vmovdqu ymm3, ymmword ptr [rsp + 160]
	vpxor ymm14, ymm14, ymm3
	vpxor ymm0, ymm14, ymm0
	vpxor ymm0, ymm13, ymm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm13, xmm13, xmm13
	vpermq ymm13, ymm4, 78
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm11, ymm12, ymm11
	vpaddd ymm12, ymm11, ymm13
	vpsrld ymm11, ymm0, 7
	vpslld ymm13, ymm0, 25
	vpor ymm11, ymm13, ymm11
	vpaddd ymm4, ymm11, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm11, ymm12, 32
	vpsrlq ymm13, ymm4, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm13, ymm13, ymm11
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm14, ymm0, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm14, ymm14, ymm11
	vpmuludq ymm11, ymm12, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm15, ymm13, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm15, ymm11, ymm15, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm11, ymm11, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm13, ymm11, ymm13, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm1, ymm12, ymm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm11, ymm14, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm11, ymm1, ymm11, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm1, ymm1, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm1, ymm1, ymm14, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm14, xmm14, xmm14
	vpermd ymm14, ymm9, ymm12
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm4, ymm14, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm14, xmm14, xmm14
	vpermq ymm14, ymm0, 147
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm12, ymm14, ymm12
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm14, xmm14, xmm14
	vpermd ymm14, ymm8, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm0, ymm14, ymm0
	vpxor ymm12, ymm12, ymm2
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm14, xmm14, xmm14
	vpermq ymm14, ymm12, 57
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm1, ymm14, ymm1
	vpaddd ymm4, ymm4, ymm7
	vmovdqa ymm10, ymm7
	vpxor ymm1, ymm1, ymm4
	vpxor ymm4, ymm15, ymm12
	vpaddd ymm0, ymm0, ymm3
	vmovdqa ymm2, ymm3
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vmovdqa ymm3, ymmword ptr [rip + __ymm@0000000400000003000000020000000100000000000000070000000600000005]
	vxorps xmm12, xmm12, xmm12
	vpermd ymm12, ymm3, ymm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm4, ymm12, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm12, xmm12, xmm12
	vpermd ymm12, ymm9, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm12, ymm12, ymm13
	vmovdqa ymm3, ymm6
	vpxor ymm1, ymm1, ymm6
	vpsrld ymm13, ymm1, 27
	vpslld ymm1, ymm1, 5
	vpor ymm1, ymm13, ymm1
	vmovdqu ymm6, ymmword ptr [rsp + 64]
	vpaddd ymm4, ymm4, ymm6
	vpxor ymm4, ymm4, ymm5
	vpsrld ymm13, ymm4, 23
	vpslld ymm14, ymm4, 9
	vpor ymm13, ymm14, ymm13
	vmovdqu ymm7, ymmword ptr [rsp + 128]
	vpxor ymm12, ymm12, ymm7
	vpxor ymm0, ymm12, ymm0
	vpxor ymm0, ymm13, ymm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm12, xmm12, xmm12
	vpermq ymm12, ymm1, 78
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm11, ymm12, ymm11
	vpaddd ymm4, ymm11, ymm4
	vpsrld ymm11, ymm0, 15
	vpslld ymm12, ymm0, 17
	vpor ymm11, ymm12, ymm11
	vpaddd ymm1, ymm11, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm11, ymm4, 32
	vpsrlq ymm12, ymm1, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm12, ymm12, ymm11
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm13, ymm0, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm13, ymm13, ymm11
	vpmuludq ymm11, ymm1, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm14, ymm12, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm14, ymm11, ymm14, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm11, ymm11, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm12, ymm11, ymm12, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2221
		transmute(simd_mul(simd_and(a, mask), simd_and(b, mask)))
	vpmuludq ymm15, ymm0, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:2871
		transmute(simd_shl(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsllq ymm11, ymm13, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm11, ymm15, ymm11, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\x86\avx2.rs:3290
		transmute(simd_shr(a.as_u64x4(), u64x4::splat(IMM8 as u64)))
	vpsrlq ymm15, ymm15, 32
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\stdarch\crates\core_arch\src\macros.rs:157
		$crate::intrinsics::simd::simd_shuffle(
	vpblendd ymm13, ymm15, ymm13, 170
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm15, xmm15, xmm15
	vpermd ymm15, ymm9, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm1, ymm15, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm15, xmm15, xmm15
	vpermq ymm15, ymm0, 147
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm4, ymm15, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm15, xmm15, xmm15
	vpermd ymm15, ymm8, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm0, ymm15, ymm0
	vpaddd ymm1, ymm1, ymm3
	vpxor ymm4, ymm4, ymm5
	vpaddd ymm0, ymm0, ymm2
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vpermq ymm5, ymm4, 57
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm5, ymm13, ymm5
	vpxor ymm1, ymm5, ymm1
	vpxor ymm4, ymm14, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vmovdqa ymm2, ymmword ptr [rip + __ymm@0000000400000003000000020000000100000000000000070000000600000005]
	vxorps xmm5, xmm5, xmm5
	vpermd ymm5, ymm2, ymm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm4, ymm5, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm5, xmm5, xmm5
	vpermd ymm5, ymm9, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm5, ymm12, ymm5
	vpxor ymm1, ymm1, ymm6
	vpaddd ymm4, ymm4, ymmword ptr [rsp + 96]
	vpsrld ymm6, ymm1, 29
	vpslld ymm1, ymm1, 3
	vpor ymm1, ymm1, ymm6
	vpxor ymm2, ymm10, ymm4
	vpsrld ymm4, ymm2, 9
	vpslld ymm6, ymm2, 23
	vpor ymm4, ymm6, ymm4
	vpxor ymm3, ymm5, ymm7
	vpxor ymm0, ymm3, ymm0
	vpxor ymm0, ymm0, ymm4
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm3, xmm3, xmm3
	vpermq ymm3, ymm1, 78
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm2, ymm11, ymm2
	vpaddd ymm2, ymm3, ymm2
	vpsrld ymm3, ymm0, 19
	vpslld ymm4, ymm0, 13
	vpor ymm3, ymm4, ymm3
	vpaddd ymm1, ymm3, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm3, xmm3, xmm3
	vpermq ymm3, ymm2, 147
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpxor ymm1, ymm1, ymm3
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vxorps xmm3, xmm3, xmm3
	vpermd ymm3, ymm8, ymm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm2, ymm3, ymm2
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\swizzle.rs:88
		core::intrinsics::simd::simd_shuffle(
	vmovdqa ymm3, ymmword ptr [rip + __ymm@0000000600000005000000040000000300000002000000010000000000000007]
	vpermd ymm3, ymm3, ymm1
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\portable-simd\crates\core_simd\src\ops.rs:38
		unsafe { core::intrinsics::simd::$simd_call($lhs, $rhs) }
	vpaddd ymm0, ymm3, ymm0
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\ptr\mod.rs:551
		unsafe { crate::intrinsics::copy_nonoverlapping(src, dst, count) }
	vmovdqa ymmword ptr [rsi], ymm1
	vmovdqa ymmword ptr [rsi + 32], ymm2
	vmovdqa ymmword ptr [rsi + 64], ymm0
	vmovaps xmm0, xmmword ptr [rsp + 48]
		// src\generate.rs:248
		self.xoshiro256 = xoshiro256;
	vmovaps xmmword ptr [rsi + 352], xmm0
		// src\generate.rs:253
		let t = xoshiro256[1] << 17;
	shl rcx, 17
		// src\generate.rs:259
		xoshiro256[2] ^= t;
	xor rdx, rcx
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\intrinsics\mod.rs:2029
		unsafe { unchecked_funnel_shl(x, x, shift % (mem::size_of::<T>() as u32 * 8)) }
	rorx rcx, r9, 19
		// src\generate.rs:248
		self.xoshiro256 = xoshiro256;
	mov qword ptr [rsi + 368], rdx
	mov qword ptr [rsi + 376], rcx
	xor edi, edi
.LBB11_2:
		// %userprofile%\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\rand_core-0.10.0\src\block.rs:299
		dest_rem.copy_from_slice(&src.to_le_bytes().as_ref()[..n]);
	mov rcx, qword ptr [rsi + 8*rdi]
	mov qword ptr [rsp + 40], rcx
	lea rdx, [rsp + 40]
		// %userprofile%\.rustup\toolchains\nightly-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library\core\src\ptr\mod.rs:551
		unsafe { crate::intrinsics::copy_nonoverlapping(src, dst, count) }
	mov rcx, rax
	vzeroupper
	call memcpy
		// %userprofile%\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\rand_core-0.10.0\src\block.rs:300
		index += 1;
	inc rdi
		// %userprofile%\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\rand_core-0.10.0\src\block.rs:181
		self.results[0] = W::from_usize(index);
	mov qword ptr [rsi], rdi
	vmovaps xmm6, xmmword ptr [rsp + 192]
	vmovaps xmm7, xmmword ptr [rsp + 208]
	vmovaps xmm8, xmmword ptr [rsp + 224]
	vmovaps xmm9, xmmword ptr [rsp + 240]
	vmovaps xmm10, xmmword ptr [rsp + 256]
	vmovaps xmm11, xmmword ptr [rsp + 272]
	vmovaps xmm12, xmmword ptr [rsp + 288]
	vmovaps xmm13, xmmword ptr [rsp + 304]
	vmovaps xmm14, xmmword ptr [rsp + 320]
	vmovaps xmm15, xmmword ptr [rsp + 336]
		// %userprofile%\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\rand_core-0.10.0\src\block.rs:307
		}
	add rsp, 360
	pop rdi
	pop rsi
	ret
