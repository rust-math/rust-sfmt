#![feature(test)]

extern crate rand;
extern crate sfmt;
extern crate test;

use test::Bencher;
use rand::*;
use sfmt::SFMT;

macro_rules! def_bench { ($name:ident, $t:ty, $rng:expr) => {
#[bench]
fn $name(b: &mut Bencher) {
    let mut rng = $rng;
    b.iter(|| rng.gen::<$t>());
}
}} // def_bench!

mod gen_f64 {
    use super::*;
    def_bench!(xorshift, f64, XorShiftRng::new_unseeded());
    def_bench!(sfmt, f64, SFMT::new(1234));
}

mod gen_f32 {
    use super::*;
    def_bench!(xorshift, f32, XorShiftRng::new_unseeded());
    def_bench!(sfmt, f32, SFMT::new(1234));
}

mod gen_u64 {
    use super::*;
    def_bench!(xorshift, u64, XorShiftRng::new_unseeded());
    def_bench!(sfmt, u64, SFMT::new(1234));
}

mod gen_u32 {
    use super::*;
    def_bench!(xorshift, u32, XorShiftRng::new_unseeded());
    def_bench!(sfmt, u32, SFMT::new(1234));
}
