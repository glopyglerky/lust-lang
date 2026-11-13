// expect: L0507
// `std::iter::*` brings iter::Once and `std::sync::*` brings sync::Once
// (rustc E0659 when used).
use std::iter::*;
use std::sync::*;
fn main() {
    let x: Once<u8> = once(1);
    let _ = x;
}
