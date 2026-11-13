// expect: L0506
// A missing std path in a fn-body `use` (rustc E0432), even as `_`.
fn f() {
    use std::fmt::Nope as _;
}

fn main() {
    f();
    println!("hi");
}
