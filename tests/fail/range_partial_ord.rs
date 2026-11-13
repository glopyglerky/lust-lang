// expect: L0112
// rustc E0369: the range types are not PartialOrd.
fn main() {
    println!("{}", (0..1) < (0..2));
}
