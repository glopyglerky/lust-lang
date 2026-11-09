// expect: L0104
// rustc E0599: floats are not `Ord`, so there is no `cmp` method.
fn main() {
    println!("{:?}", 1.0f64.cmp(&2.0f64));
}
