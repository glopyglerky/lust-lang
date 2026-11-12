#![no_std]
#![forbid(unsafe_code)]

const Vec: usize = 2;

pub struct Width<const N: usize>;

impl<const N: usize> Width<N> {
    pub fn value() -> usize { N }
}

pub fn width() -> usize { Width::<Vec>::value() }
