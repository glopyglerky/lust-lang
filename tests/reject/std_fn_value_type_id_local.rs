// expect: L0702
// Valid Rust (prints `true`): two uses of `Vec::<u8>::new` share one fn
// item type. lust eta-expands a std fn value to a closure, one type per
// use, so a TypeId of it is a reject.
use std::any::Any;
fn main() {
    let a = Vec::<u8>::new;
    let b = Vec::<u8>::new;
    println!("{}", a.type_id() == b.type_id());
}
