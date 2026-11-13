#![no_std]
#![forbid(unsafe_code)]

mod arithmetic;
pub struct Window<'a> { pub values: &'a [u16] }

impl<'a> Window<'a> {
    pub fn sum(&self) -> u16 { arithmetic::sum(self.values) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pair { pub left: u16, pub right: u16 }

pub enum Outcome<T> { Empty, Value(T) }

pub fn first<const N: usize>(values: &[u16; N]) -> Outcome<u16> {
    match values.first() {
        Some(value) => Outcome::Value(*value),
        None => Outcome::Empty,
    }
}

pub const fn width<const N: usize>() -> usize { N }

pub fn view<'a>(values: &'a [u16]) -> Window<'a> { Window { values } }

pub fn checked(value: u16) -> Result<u16, ()> {
    assert_eq!(core::mem::size_of::<u16>(), 2);
    assert_eq!(&[2usize, 5, 8][..2], &[2, 5]);
    assert_ne!(&[2usize, 5, 8][..1], &[2, 5]);
    if value < 100 { Ok(value) } else { Err(()) }
}

// Local names remain ordinary Rust names; no_std is not a spelling ban.
struct Vec { value: i32 }
mod std { pub fn answer() -> i32 { 42 } }

pub fn shadowed_names() -> i32 {
    let value = Vec { value: std::answer() };
    value.value
}

mod macros {
    macro_rules! println { () => { 42 }; }
    pub(crate) use println;
}

use macros::*;
pub fn local_macro() -> i32 { println!() }
