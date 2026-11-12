#![no_std]
#![forbid(unsafe_code)]
extern crate std;

fn main() {
    let mut values = std::vec::Vec::new();
    values.push(42);
    std::println!("{}", values[0]);
}
