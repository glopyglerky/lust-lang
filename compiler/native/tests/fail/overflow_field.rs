// expect: L0540
struct S {
    x: u8,
}
fn main() {
    let s = S { x: 255 };
    println!("{}", s.x + 1);
}
