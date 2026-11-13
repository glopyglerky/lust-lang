#![no_std]
#![forbid(unsafe_code)]

extern crate std;

pub struct Mark;

pub struct Retained;
mod retained;

mod child {
    pub fn answer() -> u32 {
        let Mark = 1u32;
        let _: crate::Mark = crate::Mark;
        Mark + 41
    }
}

pub use child::answer;
fn main() {
    assert_eq!(answer(), 42);
    assert_eq!(retained::value(), 0);
    std::println!("{}", answer());
}
