// expect: L0110
// rustc E0308: a closure right under `&` keeps its closure type, and
// `&{closure}` does not coerce to `&fn(u8) -> u8`.
fn main() {
    let f: &fn(u8) -> u8 = &|a| a + 1;
    println!("{}", f(2));
}
