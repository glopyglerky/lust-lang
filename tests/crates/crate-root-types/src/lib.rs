#![no_std]
#![forbid(unsafe_code)]

pub enum Fault { Missing, Code(u8) }

impl Fault {
    pub fn missing() -> Self { Self::Missing }
    pub fn code(&self) -> u8 {
        match self {
            Self::Missing => 0,
            Self::Code(value) => *value,
        }
    }
}

pub struct Bundle<T = u8, const N: usize = 2> { pub values: [T; N] }

impl<T, const N: usize> Bundle<T, N> {
    pub fn width(&self) -> usize { N }
}

pub mod resolution;
