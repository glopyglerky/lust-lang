#![no_std]
#![forbid(unsafe_code)]

extern crate std;

pub struct Parcel { pub value: u8 }

pub struct Retained;
mod retained;

mod child {
    fn inspect<T>(value: crate::Parcel) -> u8 { value.value }

    pub fn answer() -> u8 {
        inspect::<()>(crate::Parcel { value: 42 })
    }
}

pub use child::answer;
fn main() {
    assert_eq!(answer(), 42);
    assert_eq!(retained::value(), 0);
    std::println!("{}", answer());
}
