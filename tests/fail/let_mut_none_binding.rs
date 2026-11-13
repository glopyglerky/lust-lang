// expect: L0110
// rustc E0530: let bindings cannot shadow unit variants.
fn main() {
    let mut None = 5;
    None += 1;
    println!("{}", None);
}
