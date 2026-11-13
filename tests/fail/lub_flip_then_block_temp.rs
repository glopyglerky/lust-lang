// expect: L0306
// rustc E0716: with no expected type, rustc's LUB coerces the then
// block `{ &&G }` to `&u8` after the block, when its tail temporary
// `&G` is already dropped.
static G: u8 = 9;
fn g<'a>(v: &'a Vec<u8>) -> Option<&'a u8> {
    let r: Option<&'a u8> = Some(1u8).map(|k| if k > 0 { &&G } else { &v[0] });
    r
}
fn main() {
    let v = vec![1u8, 2];
    println!("{:?}", g(&v));
}
