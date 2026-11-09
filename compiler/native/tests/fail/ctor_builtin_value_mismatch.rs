// expect: L0110
// rustc E0308: `f` is `Some::<u16>` after its first call.
fn main() {
    let f = Some;
    let a = f(1u16);
    let b: Option<u8> = f(2);
    println!("{a:?} {b:?}");
}
