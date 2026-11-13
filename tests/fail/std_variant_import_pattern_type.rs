// expect: L0110
// An imported std variant in a pattern is the variant, not a binding.
use std::cmp::Ordering::Less;
fn main() {
    let x = 5;
    match x {
        Less => println!("a"),
        _ => println!("b"),
    }
}
