//!  Rust implementation of [SIMD-oriented Fast Mersenne Twister (SFMT)] using [stable SIMD]
//!
//! [SIMD-oriented Fast Mersenne Twister (SFMT)]: http://www.math.sci.hiroshima-u.ac.jp/~m-mat/MT/SFMT/
//! [stable SIMD]: https://github.com/rust-lang/rfcs/blob/master/text/2325-stable-simd.md
//!
//! ```
//! use rand_core::{RngCore, SeedableRng};
//! let mut rng = sfmt::SFMT19937::seed_from_u64(42);
//! let r = rng.next_u32();
//! println!("random u32 number = {}", r);
//! ```

mod packed;
mod sfmt;
#[cfg(feature = "thread_rng")]
mod thread_rng;

#[cfg(feature = "thread_rng")]
pub use self::thread_rng::{thread_rng, ThreadRng};

/// Fall back to [`SFMT19937`], not be a breaking change.
pub type SFMT = SFMT19937;
/// SFMT with a state length 607
pub type SFMT607 = paramed::SFMT<607, { 607 / 128 + 1 }>;
/// SFMT with a state length 1279
pub type SFMT1279 = paramed::SFMT<1279, { 1279 / 128 + 1 }>;
/// SFMT with a state length 2281
pub type SFMT2281 = paramed::SFMT<2281, { 2281 / 128 + 1 }>;
/// SFMT with a state length 4253
pub type SFMT4253 = paramed::SFMT<4253, { 4253 / 128 + 1 }>;
/// SFMT with a state length 11213
pub type SFMT11213 = paramed::SFMT<11213, { 11213 / 128 + 1 }>;
/// SFMT with a state length 19937
pub type SFMT19937 = paramed::SFMT<19937, { 19937 / 128 + 1 }>;
/// SFMT with a state length 44497
pub type SFMT44497 = paramed::SFMT<44497, { 44497 / 128 + 1 }>;
/// SFMT with a state length 86243.
pub type SFMT86243 = paramed::SFMT<86243, { 86243 / 128 + 1 }>;
/// SFMT with a state length 132049.
pub type SFMT132049 = paramed::SFMT<132049, { 132049 / 128 + 1 }>;
/// SFMT with a state length 216091.
pub type SFMT216091 = paramed::SFMT<216091, { 216091 / 128 + 1 }>;

/// Internal implemention of SFMT with `MEXP` parameter.
pub mod paramed {
    use crate::{
        packed::*,
        sfmt::{SfmtParams, SFMTMEXP},
    };
    use rand_core::{impls, Error, RngCore, SeedableRng};

    /// State of SFMT
    ///
    /// This struct implements random number generation through `rand::Rng`.
    /// The MEXP is a parameter that defines a length of state.
    /// MEXP is limted to be a known value, and it is checked at compile time.
    /// MEXP can only be `607,1279,2281,4253,11213,19937,44497,86243,132049,216091`.
    /// Since there is a limitation to const generics, we also need the `MEXP_N = {MEXP / 128 + 1}`
    /// ```
    /// # use rand_core::SeedableRng;
    /// let s = sfmt::SFMT19937::seed_from_u64(23);
    /// ```
    #[derive(Clone)]
    pub struct SFMT<const MEXP: usize, const MEXP_N: usize> {
        /// the 128-bit internal state array
        pub(crate) state: [i32x4; MEXP_N],
        /// index counter to the 32-bit internal state array
        pub(crate) idx: usize,
    }

    impl<const MEXP: usize, const MEXP_N: usize> SFMT<MEXP, MEXP_N>
    where
        SFMTMEXP<MEXP, MEXP_N>: SfmtParams<MEXP, MEXP_N>,
    {
        fn pop32(&mut self) -> u32 {
            let val = extract(self.state[self.idx / 4], self.idx % 4);
            self.idx += 1;
            val
        }

        fn pop64(&mut self) -> u64 {
            let p = self.state.as_ptr() as *const u32;
            let val = unsafe {
                let p = p.offset(self.idx as isize);
                *(p as *const u64) // reinterpret cast [u32; 2] -> u64
            };
            self.idx += 2;
            val
        }

        fn gen_all(&mut self) {
            SFMTMEXP::<MEXP, MEXP_N>::sfmt_gen_rand_all(self);
            self.idx = 0;
        }
    }

    impl<const MEXP: usize, const MEXP_N: usize> SeedableRng for SFMT<MEXP, MEXP_N>
    where
        SFMTMEXP<MEXP, MEXP_N>: SfmtParams<MEXP, MEXP_N>,
    {
        type Seed = [u8; 4];

        fn from_seed(seed: [u8; 4]) -> Self {
            let mut sfmt = Self {
                state: [zero(); MEXP_N],
                idx: 0,
            };
            let seed = unsafe { *(seed.as_ptr() as *const u32) };
            SFMTMEXP::<MEXP, MEXP_N>::sfmt_init_gen_rand(&mut sfmt, seed);
            sfmt
        }
    }

    impl<const MEXP: usize, const MEXP_N: usize> RngCore for SFMT<MEXP, MEXP_N>
    where
        SFMTMEXP<MEXP, MEXP_N>: SfmtParams<MEXP, MEXP_N>,
    {
        fn next_u32(&mut self) -> u32 {
            if self.idx >= SFMTMEXP::<MEXP, MEXP_N>::SFMT_N32 {
                self.gen_all();
            }
            self.pop32()
        }

        fn next_u64(&mut self) -> u64 {
            if self.idx >= SFMTMEXP::<MEXP, MEXP_N>::SFMT_N32 - 1 {
                // drop last u32 if idx == N32-1
                self.gen_all();
            }
            self.pop64()
        }

        fn fill_bytes(&mut self, dest: &mut [u8]) {
            impls::fill_bytes_via_next(self, dest)
        }

        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
            Ok(self.fill_bytes(dest))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::{RngCore, SeedableRng};

    #[test]
    fn random_607() {
        let mut rng = SFMT607::seed_from_u64(0);
        for _ in 0..607 * 20 {
            // Generate many random numbers to test the overwrap
            let r = rng.next_u64();
            if r % 2 == 0 {
                let _r = rng.next_u32();
            } // shift SFMT.idx randomly
        }
    }
    #[test]
    fn random_19937() {
        let mut rng = SFMT::seed_from_u64(0);
        for _ in 0..19937 * 20 {
            // Generate many random numbers to test the overwrap
            let r = rng.next_u64();
            if r % 2 == 0 {
                let _r = rng.next_u32();
            } // shift SFMT.idx randomly
        }
    }
    #[test]
    fn random_44497() {
        let mut rng = SFMT44497::seed_from_u64(0);
        for _ in 0..44497 * 20 {
            // Generate many random numbers to test the overwrap
            let r = rng.next_u64();
            if r % 2 == 0 {
                let _r = rng.next_u32();
            } // shift SFMT.idx randomly
        }
    }
    #[test]
    fn random_86243() {
        let mut rng = SFMT86243::seed_from_u64(0);
        for _ in 0..86243 * 20 {
            // Generate many random numbers to test the overwrap
            let r = rng.next_u64();
            if r % 2 == 0 {
                let _r = rng.next_u32();
            } // shift SFMT.idx randomly
        }
    }
    #[test]
    fn random_216091() {
        let mut rng = SFMT216091::seed_from_u64(0);
        for _ in 0..216091 * 20 {
            // Generate many random numbers to test the overwrap
            let r = rng.next_u64();
            if r % 2 == 0 {
                let _r = rng.next_u32();
            } // shift SFMT.idx randomly
        }
    }
}
