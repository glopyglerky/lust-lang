// expect: L0102
// rustc E0433: `NonZero` is not in the 2024 prelude; it needs `use std::num::NonZero;`.
fn main() {
    let n = NonZero::new(5u32);
    println!("{:?}", n.map(|x| x.get()));
}
