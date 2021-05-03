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
pub type SFMT607 = paramed::SFMT<607>;
/// SFMT with a state length 1279
pub type SFMT1279 = paramed::SFMT<1279>;
/// SFMT with a state length 2281
pub type SFMT2281 = paramed::SFMT<2281>;
/// SFMT with a state length 4253
pub type SFMT4253 = paramed::SFMT<4253>;
/// SFMT with a state length 11213
pub type SFMT11213 = paramed::SFMT<11213>;
/// SFMT with a state length 19937
pub type SFMT19937 = paramed::SFMT<19937>;
/// SFMT with a state length 44497
pub type SFMT44497 = paramed::SFMT<44497>;
/// SFMT with a state length 86243
pub type SFMT86243 = paramed::SFMT<86243>;
/// SFMT with a state length 132049
pub type SFMT132049 = paramed::SFMT<132049>;
/// SFMT with a state length 216091
pub type SFMT216091 = paramed::SFMT<216091>;

/// Internal implemention of SFMT with `MEXP` parameter.
pub mod paramed {
    use crate::{
        packed::*,
        sfmt::{SfmtMEXP, SfmtParams},
    };
    use rand_core::{impls, Error, RngCore, SeedableRng};

    /// State of SFMT
    ///
    /// This struct implements random number generation through `rand::Rng`.
    /// The MEXP is a parameter that defines a length of state.
    /// MEXP is limted to be a known value, and it is checked at compile time.
    /// MEXP can only be `607,1279,2281,4253,11213,19937,44497,86243,132049,216091`.
    /// ```
    /// # use rand_core::SeedableRng;
    /// let s = sfmt::SFMT19937::seed_from_u64(23);
    /// ```
    #[derive(Clone)]
    pub struct SFMT<const MEXP: usize> {
        /// the 128-bit internal state array
        pub(crate) state: [i32x4; MEXP],
        /// index counter to the 32-bit internal state array
        pub(crate) idx: usize,
    }

    impl<const MEXP: usize> SFMT<MEXP>
    where
        SfmtMEXP<MEXP>: SfmtParams<MEXP>,
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
            SfmtMEXP::<MEXP>::sfmt_gen_rand_all(self);
            self.idx = 0;
        }
    }

    impl<const MEXP: usize> SeedableRng for SFMT<MEXP>
    where
        SfmtMEXP<MEXP>: SfmtParams<MEXP>,
    {
        type Seed = [u8; 4];

        fn from_seed(seed: [u8; 4]) -> Self {
            let mut sfmt = SFMT {
                state: [zero(); MEXP],
                idx: 0,
            };
            let seed = unsafe { *(seed.as_ptr() as *const u32) };
            SfmtMEXP::<MEXP>::sfmt_init_gen_rand(&mut sfmt, seed);
            sfmt
        }
    }

    impl<const MEXP: usize> RngCore for SFMT<MEXP>
    where
        SfmtMEXP<MEXP>: SfmtParams<MEXP>,
    {
        fn next_u32(&mut self) -> u32 {
            if self.idx >= SfmtMEXP::<MEXP>::SFMT_N32 {
                self.gen_all();
            }
            self.pop32()
        }

        fn next_u64(&mut self) -> u64 {
            if self.idx >= SfmtMEXP::<MEXP>::SFMT_N32 - 1 {
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
    fn random() {
        let mut rng = SFMT::seed_from_u64(0);
        for _ in 0..19937 * 20 {
            // Generate many random numbers to test the overwrap
            let r = rng.next_u64();
            if r % 2 == 0 {
                let _r = rng.next_u32();
            } // shift SFMT.idx randomly
        }
    }
}
