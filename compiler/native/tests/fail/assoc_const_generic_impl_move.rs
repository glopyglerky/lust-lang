// expect: L0301
// rustc E0382: an associated const of a generic impl is checked once, generically.
struct S {
    x: u32,
}
struct W<T>(T);
impl<T> W<T> {
    const C: u32 = {
        let s = S { x: 1 };
        let a = s;
        let b = s;
        a.x + b.x
    };
}
fn main() {
    println!("{}", W::<u8>::C);
}
