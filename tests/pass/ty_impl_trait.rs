// `impl Trait` in argument position is a generic param with the bound;
// the std scalars implement Display.
fn show(x: impl std::fmt::Display) {
    println!("{}", x);
}

fn main() {
    show(3);
}
