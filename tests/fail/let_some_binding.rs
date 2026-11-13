// expect: L0110
// rustc E0530: let bindings cannot shadow tuple variants.
fn main() {
    let Some = 5;
    println!("{}", Some);
}
