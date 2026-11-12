#![no_std]
#![forbid(unsafe_code)]

#[path = "modules_no_std/child.rs"]
mod child;

#[cfg(any())]
mod absent;

// A test crate's user main is an ordinary function, not its entry point.
fn main(value: u8) -> u8 {
    value + 1
}

#[test]
fn original_main_is_callable() {
    assert_eq!(main(4), 5);
}
