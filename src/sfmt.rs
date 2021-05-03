//! Rust re-implementation of SFMT

use super::*;
use crate::packed::*;

/// Parameters used in sfmt.
pub trait SfmtParams<const MEXP: usize, const MEXP_N: usize>: Sized {
    const SFMT_MEXP: usize = MEXP;
    const SFMT_N: usize = MEXP_N; //Self::SFMT_MEXP / 128 + 1;
    const SFMT_N32: usize = Self::SFMT_N * 4;

    const SFMT_POS1: usize;
    const SFMT_SL1: i32;
    const SFMT_SL2: i32;
    const SFMT_SR1: i32;
    const SFMT_SR2: i32;
    const SFMT_MSK1: i32;
    const SFMT_MSK2: i32;
    const SFMT_MSK3: i32;
    const SFMT_MSK4: i32;
    const SFMT_PARITY1: u32;
    const SFMT_PARITY2: u32;
    const SFMT_PARITY3: u32;
    const SFMT_PARITY4: u32;

    fn mm_recursion(a: i32x4, b: i32x4, c: i32x4, d: i32x4) -> i32x4 {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        unsafe {
            let mask = new(
                Self::SFMT_MSK1,
                Self::SFMT_MSK2,
                Self::SFMT_MSK3,
                Self::SFMT_MSK4,
            );
            let y = _mm_srli_epi32(b, Self::SFMT_SR1);
            let z = _mm_srli_si128(c, Self::SFMT_SR2);
            let v = _mm_slli_epi32(d, Self::SFMT_SL1);
            let z = _mm_xor_si128(z, a);
            let z = _mm_xor_si128(z, v);
            let x = _mm_slli_si128(a, Self::SFMT_SL2);
            let y = _mm_and_si128(y, mask);
            let z = _mm_xor_si128(z, x);
            _mm_xor_si128(z, y)
        }
    }

    fn sfmt_gen_rand_all(sfmt: &mut paramed::SFMT<MEXP, MEXP_N>) {
        let st = &mut sfmt.state;
        let mut r1 = st[Self::SFMT_N - 2];
        let mut r2 = st[Self::SFMT_N - 1];
        for i in 0..(Self::SFMT_N - Self::SFMT_POS1) {
            st[i] = Self::mm_recursion(st[i], st[i + Self::SFMT_POS1], r1, r2);
            r1 = r2;
            r2 = st[i];
        }
        for i in (Self::SFMT_N - Self::SFMT_POS1)..Self::SFMT_N {
            st[i] = Self::mm_recursion(st[i], st[i + Self::SFMT_POS1 - Self::SFMT_N], r1, r2);
            r1 = r2;
            r2 = st[i];
        }
    }

    fn period_certification(sfmt: &mut paramed::SFMT<MEXP, MEXP_N>) {
        let mut inner = 0_u32;
        let st = &mut sfmt.state[0];
        let parity = [
            Self::SFMT_PARITY1,
            Self::SFMT_PARITY2,
            Self::SFMT_PARITY3,
            Self::SFMT_PARITY4,
        ];
        for i in 0..4 {
            inner ^= extract(*st, i) & parity[i];
        }
        for i in [16, 8, 4, 2, 1].iter() {
            inner ^= inner >> i;
        }
        inner &= 1;
        if inner == 1 {
            return;
        }
        for i in 0..4 {
            let mut work = 1_u32;
            for _ in 0..32 {
                if (work & parity[i]) != 0 {
                    let val = extract(*st, i) ^ work;
                    insert(st, val as i32, i);
                    return;
                }
                work = work << 1;
            }
        }
    }

    fn iterate(pre: i32, i: i32) -> i32 {
        use std::num::Wrapping;
        let pre = Wrapping(pre as u32);
        let i = Wrapping(i as u32);
        (Wrapping(1812433253) * (pre ^ (pre >> 30)) + i).0 as i32
    }

    fn map(a: i32, idx: i32) -> (i32x4, i32) {
        let b = Self::iterate(a, 4 * idx + 1);
        let c = Self::iterate(b, 4 * idx + 2);
        let d = Self::iterate(c, 4 * idx + 3);
        let a2 = Self::iterate(d, 4 * idx + 4);
        (new(a, b, c, d), a2)
    }

    fn sfmt_init_gen_rand(sfmt: &mut paramed::SFMT<MEXP, MEXP_N>, seed: u32) {
        let mut pre = seed as i32;
        for (idx, v) in sfmt.state.iter_mut().enumerate() {
            let (v_, pre_) = Self::map(pre, idx as i32);
            *v = v_;
            pre = pre_;
        }
        sfmt.idx = Self::SFMT_N32;
        Self::period_certification(sfmt);
    }
}
/// Wrapper for `MEXP` parameter.
pub struct SFMTMEXP<const MEXP: usize, const MEXP_N: usize>;

macro_rules! parms_impl {
    ($mexp : expr, $n : expr, $pos1 : expr, $sl1 : expr, $sl2 : expr, $sr1 : expr, $sr2 : expr,
        $msk1 : expr, $msk2 : expr, $msk3 : expr, $msk4 : expr,
        $parity1 : expr, $parity2 : expr, $parity3 : expr, $parity4 : expr) => {
        impl SfmtParams<$mexp, $n> for SFMTMEXP<$mexp, $n> {
            const SFMT_POS1: usize = $pos1;
            const SFMT_SL1: i32 = $sl1;
            const SFMT_SL2: i32 = $sl2;
            const SFMT_SR1: i32 = $sr1;
            const SFMT_SR2: i32 = $sr2;
            const SFMT_MSK1: i32 = $msk1 as i32;
            const SFMT_MSK2: i32 = $msk2 as i32;
            const SFMT_MSK3: i32 = $msk3 as i32;
            const SFMT_MSK4: i32 = $msk4 as i32;
            const SFMT_PARITY1: u32 = $parity1;
            const SFMT_PARITY2: u32 = $parity2;
            const SFMT_PARITY3: u32 = $parity3;
            const SFMT_PARITY4: u32 = $parity4;
        }
    };
}

parms_impl!(
    607,
    { 607 / 128 + 1 },
    2,
    15,
    3,
    13,
    3,
    0xfdff_37ff_u32,
    0xef7f_3f7d_u32,
    0xff77_7b7d_u32,
    0x7ff7_fb2f_u32,
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x5986_f054
);
parms_impl!(
    1279,
    { 1279 / 128 + 1 },
    7,
    14,
    3,
    5,
    1,
    0xf7fe_fffd_u32,
    0x7fef_cfff_u32,
    0xaff3_ef3f_u32,
    0xb5ff_ff7f_u32,
    0x0000_0001_u32,
    0x0000_0000_u32,
    0x0000_0000_u32,
    0x2000_0000_u32
);
parms_impl!(
    2281,
    { 2281 / 128 + 1 },
    12,
    19,
    1,
    5,
    1,
    0xbff7_ffbf_u32,
    0xfdff_fffe_u32,
    0xf7ffe_f7f_u32,
    0xf2f7_cbbf_u32,
    0x0000_0001_u32,
    0x0000_0000_u32,
    0x0000_0000_u32,
    0x41df_a600_u32
);
parms_impl!(
    4253,
    { 4253 / 128 + 1 },
    17,
    20,
    1,
    7,
    1,
    0x9f7b_ffff_u32,
    0x9fff_ff5f_u32,
    0x3eff_fffb_u32,
    0xffff_f7bb_u32,
    0xa800_0001_u32,
    0xaf53_90a3_u32,
    0xb740_b3f8_u32,
    0x6c11_486d_u32
);
parms_impl!(
    11213,
    { 11213 / 128 + 1 },
    68,
    14,
    3,
    7,
    3,
    0xefff_f7fb_u32,
    0xffff_ffef_u32,
    0xdfdf_bfff_u32,
    0x7fff_dbfd_u32,
    0x0000_0001_u32,
    0x0000_0000_u32,
    0xb740_b3f8_u32,
    0x6c11_486d_u32
);
parms_impl!(
    19937,
    { 19937 / 128 + 1 },
    122,
    18,
    1,
    11,
    1,
    0xdfff_ffef_u32,
    0xddfe_cb7f_u32,
    0xbffa_ffff_u32,
    0xbfff_fff6_u32,
    0x0000_0001_u32,
    0x0000_0000_u32,
    0x0000_0000_u32,
    0x13c9_e684_u32
);
parms_impl!(
    44497,
    { 44497 / 128 + 1 },
    330,
    5,
    3,
    9,
    3,
    0xefff_fffb_u32,
    0xdfbe_bfff_u32,
    0xbfbf_7bef_u32,
    0x9ffd_7bff_u32,
    0x0000_0001_u32,
    0x0000_0000_u32,
    0xa3ac_4000_u32,
    0xecc1_327a_u32
);
parms_impl!(
    86243,
    { 86243 / 128 + 1 },
    366,
    6,
    7,
    19,
    1,
    0xfdbf_bff7_u32,
    0xfd77_efff_u32,
    0xfd77_efff_u32,
    0xbf9f_f3ff_u32,
    0x0000_0001_u32,
    0x0000_0000_u32,
    0x0000_0000_u32,
    0x3952_8d85_u32
);
parms_impl!(
    132049,
    { 132049 / 128 + 1 },
    110,
    19,
    1,
    21,
    1,
    0xffff_bb5f_u32,
    0xfb6e_bf95_u32,
    0xfffe_fffa_u32,
    0xcff7_7fff_u32,
    0x0000_0001_u32,
    0x0000_0000_u32,
    0xcb52_0000_u32,
    0xc7e9_1c7d_u32
);
parms_impl!(
    216091,
    { 216091 / 128 + 1 },
    627,
    11,
    3,
    10,
    1,
    0xbff7_bff7_u32,
    0xbfff_ffff_u32,
    0xbfff_fa7f_u32,
    0xffdd_fbfb_u32,
    0xf800_0001_u32,
    0x89e8_0709_u32,
    0x3bd2_b64b_u32,
    0x0c64_b1e4_u32
);

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::SeedableRng;
    use std::{fs, io, io::BufRead};

    fn split(a: i32x4) -> (u32, u32, u32, u32) {
        (extract(a, 0), extract(a, 1), extract(a, 2), extract(a, 3))
    }

    fn read_answer(filename: &str) -> Result<Vec<i32x4>, io::Error> {
        let f = io::BufReader::new(fs::File::open(filename)?);
        Ok(f.lines()
            .map(|line| {
                let vals: Vec<_> = line
                    .unwrap()
                    .split(" ")
                    .map(|s| s.trim().parse().unwrap())
                    .collect();
                new(vals[0], vals[1], vals[2], vals[3])
            })
            .collect())
    }

    #[test]
    fn test_init() {
        let seed: u32 = 1234;
        let seed = unsafe { *(&seed as *const u32 as *const [u8; 4]) };
        let sfmt = paramed::SFMT::<19937, { 19937 / 128 + 1 }>::from_seed(seed);
        let ans = read_answer("check/init1234.txt").unwrap();
        for (v, a) in sfmt.state.iter().zip(ans.iter()) {
            assert_eq!(split(*v), split(*a));
        }
    }

    #[test]
    fn test_mm_recursion_19937() {
        let a = new(1, 2, 3, 4);
        let z = SFMTMEXP::<19937, { 19937 / 128 + 1 }>::mm_recursion(a, a, a, a);
        let zc = new(33816833, 50856450, 67896067, 1049604); // calculated by C code
        assert_eq!(split(z), split(zc));

        let b = new(431, 232, 83, 14);
        let c = new(213, 22, 93, 234);
        let d = new(112, 882, 23, 124);
        let z = SFMTMEXP::<19937, { 19937 / 128 + 1 }>::mm_recursion(a, b, c, d);
        let zc = new(398459137, 1355284994, -363068669, 32506884); // calculated by C code
        assert_eq!(split(z), split(zc));
    }
}
