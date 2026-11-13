// Where clauses over concrete types only on impl methods hold for every
// caller, so they need no implication from the trait (rustc accepts them).
// The scalar impls of Default, Hash, Ord, Display, From and Into must be
// there when lust proves them (they were not, in collect: L0110).
use std::fmt::Display;
use std::hash::Hash;

pub trait Val {
    fn val(&self) -> u32;
}

impl<T: Copy> Val for T {
    fn val(&self) -> u32
    where
        i64: Clone + Default,
        u8: Hash + Ord + Display,
        i64: From<u8>,
        u32: Into<u64>,
    {
        7
    }
}

struct W<T>(T);

impl<T> W<T> {
    fn get(&self) -> u32
    where
        char: From<u8>,
        f64: From<u32> + Default,
    {
        8
    }
}

impl Val for W<String> {
    fn val(&self) -> u32
    where
        String: Default + Clone + PartialEq,
        u64: From<bool>,
    {
        9
    }
}

fn main() {
    println!("{} {} {}", 5i32.val(), W(1u8).get(), W(String::new()).val());
}
