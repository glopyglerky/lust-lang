// expect: L0506
// rustc E0432: `std::os::unix::fs` has no `NoSuchThing`; the
// partial platform module must not turn unknown names into
// compat imports either (fx-stdos).
use std::os::unix::fs::NoSuchThing;

fn main() {
    println!("ok");
}
