// expect: L0602
// rustc E0562: `impl Trait` is not allowed in fn pointer parameters.
fn f(_x: fn(impl Copy)) {}
fn main() {
    println!("ok");
}
