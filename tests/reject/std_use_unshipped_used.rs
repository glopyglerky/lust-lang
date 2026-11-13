// expect: L0102
// The import of a std type lust does not model is accepted, but a use
// of the name rejects.
use std::num::Saturating;

fn main() {
    let b: Option<Saturating<u8>> = None;
    println!("{}", b.is_none());
}
