// expect: L0641
// Valid Rust: Box is fundamental, so `impl Display for Box<My>` is legal
// and `{}` runs it. lust's fmt glue calls written impls of structs and
// enums only (it would format the pointee), so the impl rejects.
use std::fmt;

struct My;

impl fmt::Display for Box<My> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "boxed")
    }
}

fn main() {
    println!("{}", Box::new(My));
    let s = Box::new(My).to_string();
    println!("{s}");
}
