// expect: L0610
// rustc E0282: the `_` half of a tuple pattern still needs a type.
fn main() {
    let (a, _) = (1, None);
    println!("{a}");
}
