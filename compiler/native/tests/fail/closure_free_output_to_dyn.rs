// expect: L0735
// rustc E0308: the same for a coercion to `dyn Fn(&u8) -> &u8`.
static G: u8 = 9;
fn main() {
    let f = |_a: &u8| -> &u8 { &G };
    let b: Box<dyn Fn(&u8) -> &u8> = Box::new(f);
    println!("{}", b(&1));
}
