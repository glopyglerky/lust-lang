// expect: L0610
// rustc E0282: `None` alone leaves `Option<_>` open.
fn main() {
    None;
    println!("x");
}
