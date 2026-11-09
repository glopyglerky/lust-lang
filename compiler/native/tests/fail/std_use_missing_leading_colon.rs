// expect: L0506
// A missing std path with a leading `::` (rustc E0432).
use ::std::nope::X;

fn main() {
    println!("hi");
}
