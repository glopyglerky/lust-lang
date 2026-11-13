#![no_std]
#![forbid(unsafe_code)]

pub enum Fault { Code(u8) }

impl Fault {
    pub fn code(&self) -> u8 {
        match self { Self::Code(value) => *value }
    }
}

pub struct Bundle<T = u8, const N: usize = 2> { pub values: [T; N] }

mod child;
pub use child::answer;
