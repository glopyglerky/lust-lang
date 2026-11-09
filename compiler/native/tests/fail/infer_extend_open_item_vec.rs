// expect: L0610
// rustc E0283: `Vec<u8>: Extend<u8>` and `Extend<&u8>` both apply to `vec![]`.
fn main() {
    let mut v = vec![1u8];
    v.extend(vec![]);
    println!("{v:?}");
}
